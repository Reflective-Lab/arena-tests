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
use embassy_gleif::{GleifProvider, GleifRequest, Lei, LiveGleifProvider};
use embassy_ofac_sls::{OfacSlsProvider, OfacSlsRequest, StubOfacSlsProvider};
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

    /// Accept stub Embassy providers. Without this flag the scenario
    /// exits non-zero because Embassy ports do not yet ship live
    /// providers — running them silently against stubs would be the
    /// theatre this stack prohibits.
    #[arg(long)]
    mock_ok: bool,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    print_banner(&cli);

    // The OFAC SDN port still ships stub-only. GLEIF now has a live
    // provider (`LiveGleifProvider`), so by default the binary calls
    // the real GLEIF API for identity but cannot screen for sanctions
    // without operator consent to use the stub OFAC provider. The
    // refusal stays until OFAC has a live provider too.
    if !cli.mock_ok {
        eprintln!();
        eprintln!("ERROR: REAL-by-default refused.");
        eprintln!();
        eprintln!(
            "  GLEIF identity lookup now runs LIVE via LiveGleifProvider\n  \
            (api.gleif.org, no auth required). OFAC SDN screening still\n  \
            ships stub-only (StubOfacSlsProvider); a live provider over\n  \
            the published OFAC SDN feed is the next move to close G1."
        );
        eprintln!();
        eprintln!(
            "  Gap reference: Embassy-stubs-only in\n  \
            ~/dev/reflective/stack/mosaic-extensions/kb/Standards/Real-by-Default \
Connections.md\n  \
            G1 in ~/dev/reflective/marquee-apps/shoal-meta/kb/portfolio-stretch.md"
        );
        eprintln!();
        eprintln!(
            "  Re-run with `--mock-ok` to proceed with LIVE GLEIF + STUB OFAC.\n  \
            Output labels each step's mode explicitly so the audit trail is\n  \
            honest about which evidence is live and which is contract-shape."
        );
        return ExitCode::from(2);
    }

    print_mode_table();

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
            "MOCK-OK (Embassy stubs accepted)"
        } else {
            "REAL (will refuse to run on stubs)"
        }
    );
    println!("══════════════════════════════════════════════════════════════════════");
}

fn print_mode_table() {
    println!();
    println!("Subsystem resource declaration:");
    println!("  GLEIF identity   : REAL LIVE       (LiveGleifProvider → api.gleif.org)");
    println!("  OFAC screening   : CONTRACT-SHAPE  (StubOfacSlsProvider; live provider pending)");
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

    // ───────────────── Step 1: GLEIF identity lookup (LIVE) ──────────
    println!("── Step 1: identity lookup via Embassy GLEIF (LIVE api.gleif.org) ────");
    let lei = Lei::parse(&cli.lei).map_err(|e| format!("invalid LEI: {e}"))?;
    let gleif_provider = LiveGleifProvider::new();
    let gleif_request = GleifRequest::Lookup { lei: lei.clone() };
    let gleif_response = gleif_provider.fetch(&gleif_request, &ctx).await?;
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
    println!("── Step 2: sanctions screening via Embassy OFAC-SLS ──────────────────");
    let subject = SanctionsSubject::parse(&cli.counterparty)
        .map_err(|e| format!("invalid subject: {e}"))?;
    let ofac_provider = StubOfacSlsProvider;
    let ofac_request = OfacSlsRequest::Screen { subject };
    let ofac_response = ofac_provider.screen(&ofac_request, &ctx).await?;
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
    println!(
        "Resource declaration: {} (Embassy stubs accepted via --mock-ok). \n\
         A LIVE run requires a real Embassy provider — see Cargo.toml header.",
        if hit.is_some() {
            "DENY on stub OFAC hit"
        } else {
            "ALLOW on stub clean screen"
        }
    );
    println!("══════════════════════════════════════════════════════════════════════");

    Ok(())
}
