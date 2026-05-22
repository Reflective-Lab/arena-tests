//! Atelier showcase: counterparty KYC convergence.
//!
//! Demonstrates the atomic unit of the Mosaic moat: a counterparty's
//! identity is looked up (Embassy GLEIF), screened against sanctions
//! (Embassy OFAC-SLS), and routed to a typed onboarding decision —
//! with every step producing a typed `Observation<T>` that an LLM-only
//! competitor cannot fabricate.
//!
//! ## REAL-by-default declaration
//!
//! Per `~/dev/reflective/stack/mosaic-extensions/kb/Standards/Real-by-Default Connections.md`:
//!
//! - **GLEIF identity lookup** — `CONTRACT-SHAPE` today. Embassy
//!   `gleif` ships only `StubGleifProvider`; a live HTTP provider
//!   against `https://api.gleif.org/api/v1/lei-records` is the
//!   documented next step. (Gap: G1 / Embassy-stubs-only.)
//! - **OFAC SDN screening** — `CONTRACT-SHAPE` today. Embassy
//!   `ofac-sls` ships only `StubOfacSlsProvider`; a live provider
//!   over the OFAC SDN data feed is the next step. (Gap: G1.)
//! - **Decision logic** — `LOCAL REAL`. The "deny on sanctions hit"
//!   rule is plain Rust over the typed `SanctionsHit` payload.
//! - **Causal record** — `LOCAL REAL`. The scenario builds the typed
//!   evidence chain in memory and prints it. Full Mnemos
//!   `agentic::causal` write awaits the Mnemos client wiring (gap:
//!   G3 / Mnemos agentic memory dark).
//! - **Soter SMT proof of non-bypass** — `DEFERRED`. Cedar policy at
//!   `policies/no-sanctioned-onboarding.cedar` is included as the
//!   target. Wiring it through `arbiter::CedarAnalysisSuggestor` +
//!   the vendored CVC5 (see `cedar-smt-analysis` for the pattern)
//!   requires extending `arbiter::ContextIn` to carry
//!   `sanctions_hit_present` — tracked as a follow-up.
//!
//! Until G1 closes (a real Embassy provider lands), running this
//! scenario without `--mock-ok` exits non-zero. With `--mock-ok` it
//! runs end-to-end against the stubs and labels everything honestly.
//!
//! ## Why a generic substitute fails
//!
//! An LLM-only counterparty-screening prompt produces a *claim* that
//! the model checked the list. It cannot produce a signed
//! `Observation<SanctionsHit>` with `request_hash`, `match_type`,
//! `match_score`, source-list name, and replay envelope. The audit
//! trail of "did we miss a hit?" reduces to "we asked the model and
//! it said no" — which is exactly the failure mode regulators reject.

use std::process::ExitCode;

use clap::Parser;
use embassy_gleif::{GleifProvider, GleifRequest, Lei, LiveGleifProvider, StubGleifProvider};
use embassy_ofac_sls::{
    LiveOfacSlsProvider, OfacSlsProvider, OfacSlsRequest, StubOfacSlsProvider,
};
use embassy_pack::{CallContext, SanctionsSubject};

#[derive(Parser, Debug)]
#[command(
    name = "counterparty-kyc-convergence",
    about = "Counterparty KYC convergence — typed identity + sanctions evidence",
    long_about = "REAL connections are the default. Embassy ports ship stub-only \
today; this scenario refuses to run without --mock-ok until a live \
provider lands. See \
~/dev/reflective/stack/mosaic-extensions/kb/Standards/Real-by-Default \
Connections.md for the doctrine."
)]
struct Cli {
    /// Counterparty legal name to screen.
    #[arg(long, default_value = "BLOCKED Holdings AB")]
    counterparty: String,

    /// LEI of the counterparty (mod-97-valid).
    #[arg(long, default_value = "529900T8BM49AURSDO55")]
    lei: String,

    /// Run against the deterministic stub Embassy providers instead of
    /// live network calls. Useful for offline / CI runs where the
    /// outbound network to api.gleif.org or treasury.gov is unreachable.
    /// Default is REAL LIVE for both Embassy legs.
    #[arg(long)]
    mock_ok: bool,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    print_banner(&cli);

    // Both Embassy legs are now live: GLEIF identity via
    // LiveGleifProvider (api.gleif.org), OFAC screening via
    // LiveOfacSlsProvider (downloads the canonical SDN.CSV from
    // treasury.gov). --mock-ok is no longer required; it is kept as
    // an explicit opt-out for offline / CI scenarios that cannot
    // reach the network.
    print_mode_table(cli.mock_ok);

    match run_scenario(&cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("scenario failed: {err}");
            ExitCode::FAILURE
        }
    }
}

fn print_banner(cli: &Cli) {
    println!("══════════════════════════════════════════════════════════════════════");
    println!("Counterparty KYC convergence");
    println!("──────────────────────────────────────────────────────────────────────");
    println!("  counterparty: {}", cli.counterparty);
    println!("  lei:          {}", cli.lei);
    println!(
        "  mode:         {}",
        if cli.mock_ok {
            "MOCK-OK (Embassy stub providers; offline-safe)"
        } else {
            "REAL LIVE (api.gleif.org + treasury.gov SDN.CSV)"
        }
    );
    println!("══════════════════════════════════════════════════════════════════════");
}

fn print_mode_table(mock_ok: bool) {
    let (gleif_label, ofac_label) = if mock_ok {
        (
            "CONTRACT-SHAPE  (StubGleifProvider; offline)",
            "CONTRACT-SHAPE  (StubOfacSlsProvider; offline)",
        )
    } else {
        (
            "REAL LIVE       (LiveGleifProvider → api.gleif.org)",
            "REAL LIVE       (LiveOfacSlsProvider → treasury.gov SDN.CSV)",
        )
    };
    println!();
    println!("Subsystem resource declaration:");
    println!("  GLEIF identity   : {gleif_label}");
    println!("  OFAC screening   : {ofac_label}");
    println!("  Decision logic   : LOCAL REAL");
    println!("  Causal record    : LOCAL REAL      (in-memory; Mnemos client pending)");
    println!("  SMT non-bypass   : DEFERRED        (see policies/no-sanctioned-onboarding.cedar)");
    println!();
}

async fn run_scenario(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = CallContext::default();
    let causal_chain = std::cell::RefCell::new(Vec::<String>::new());
    let record_step = |step: &str| {
        let n = causal_chain.borrow().len() + 1;
        causal_chain.borrow_mut().push(format!("[{:>3}] {step}", n));
    };

    // ───────────────── Step 1: GLEIF identity lookup ─────────────────
    let gleif_header = if cli.mock_ok {
        "── Step 1: identity lookup via Embassy GLEIF (STUB; offline) ─────────"
    } else {
        "── Step 1: identity lookup via Embassy GLEIF (LIVE api.gleif.org) ────"
    };
    println!("{gleif_header}");
    let lei = Lei::parse(&cli.lei).map_err(|e| format!("invalid LEI: {e}"))?;
    let gleif_request = GleifRequest::Lookup { lei: lei.clone() };
    let gleif_response: embassy_gleif::GleifResponse = if cli.mock_ok {
        StubGleifProvider.fetch(&gleif_request, &ctx).await?
    } else {
        LiveGleifProvider::new().fetch(&gleif_request, &ctx).await?
    };
    if gleif_response.records.is_empty() {
        eprintln!("  no entity found for LEI {}", lei.as_str());
        record_step("identity: no GLEIF record");
    } else {
        let obs = &gleif_response.records[0];
        println!("  request_hash:  {}", obs.request_hash);
        println!("  observation:   {}", obs.observation_id);
        println!("  vendor:        {}", obs.vendor);
        println!("  legal_name:    {}", obs.content.legal_name);
        record_step(&format!(
            "identity: GLEIF record {} for LEI {}",
            obs.content.legal_name,
            lei.as_str()
        ));
    }

    // ───────────────── Step 2: OFAC SDN screening ─────────────────
    println!();
    let ofac_header = if cli.mock_ok {
        "── Step 2: sanctions screening via Embassy OFAC-SLS (STUB; offline) ──"
    } else {
        "── Step 2: sanctions screening via Embassy OFAC-SLS (LIVE SDN.CSV) ───"
    };
    println!("{ofac_header}");
    let subject = SanctionsSubject::parse(&cli.counterparty)
        .map_err(|e| format!("invalid subject: {e}"))?;
    let ofac_request = OfacSlsRequest::Screen { subject };
    let ofac_response: embassy_ofac_sls::OfacSlsResponse = if cli.mock_ok {
        StubOfacSlsProvider.screen(&ofac_request, &ctx).await?
    } else {
        LiveOfacSlsProvider::new().screen(&ofac_request, &ctx).await?
    };
    let hit = ofac_response.records.first();
    if let Some(obs) = hit {
        println!("  HIT");
        println!("  request_hash:  {}", obs.request_hash);
        println!("  observation:   {}", obs.observation_id);
        println!("  list_name:     {}", obs.content.list_name);
        println!("  match_type:    {:?}", obs.content.match_type);
        println!("  match_score:   {:.2}", obs.content.match_score);
        record_step(&format!(
            "screening: OFAC {:?} match on '{}' (score {:.2})",
            obs.content.match_type, obs.content.subject_name, obs.content.match_score
        ));
    } else {
        println!("  CLEAN  (no records returned)");
        record_step("screening: OFAC clean (zero records)");
    }

    // ───────────────── Step 3: typed decision ─────────────────
    println!();
    println!("── Step 3: typed onboarding decision ─────────────────────────────────");
    let decision = if hit.is_some() { "DENY" } else { "ALLOW" };
    let reason = if hit.is_some() {
        "OFAC sanctions hit present"
    } else {
        "no sanctions hits across queried lists"
    };
    println!("  decision: {decision}");
    println!("  reason:   {reason}");
    record_step(&format!("decision: {decision} ({reason})"));

    // ───────────────── Step 4: causal chain (in-memory Mnemos shape) ──
    println!();
    println!("── Step 4: causal evidence chain ────────────────────────────────────");
    for line in causal_chain.borrow().iter() {
        println!("  {line}");
    }
    println!();
    println!("══════════════════════════════════════════════════════════════════════");
    let mode_tag = if cli.mock_ok {
        "Embassy stubs (offline)"
    } else {
        "Embassy LIVE (api.gleif.org + treasury.gov SDN.CSV)"
    };
    let outcome_tag = if hit.is_some() {
        "DENY on OFAC hit"
    } else {
        "ALLOW on clean OFAC screen"
    };
    println!("Resource declaration: {outcome_tag} ({mode_tag}).");
    println!("══════════════════════════════════════════════════════════════════════");

    Ok(())
}
