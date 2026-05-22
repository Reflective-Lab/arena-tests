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
use embassy_commerce_csl::{
    CommerceCslProvider, CommerceCslRequest, LiveCommerceCslProvider, StubCommerceCslProvider,
};
use embassy_eu_sanctions::{
    EuSanctionsProvider, EuSanctionsRequest, LiveEuSanctionsProvider, StubEuSanctionsProvider,
};
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
    let labels = if mock_ok {
        [
            "CONTRACT-SHAPE  (StubGleifProvider; offline)",
            "CONTRACT-SHAPE  (StubOfacSlsProvider; offline)",
            "CONTRACT-SHAPE  (StubEuSanctionsProvider; offline)",
            "CONTRACT-SHAPE  (StubCommerceCslProvider; offline)",
        ]
    } else {
        [
            "REAL LIVE       (LiveGleifProvider → api.gleif.org)",
            "REAL LIVE       (LiveOfacSlsProvider → treasury.gov SDN.CSV)",
            "REAL LIVE       (LiveEuSanctionsProvider → OpenSanctions mirror of EU FSF)",
            "REAL LIVE       (LiveCommerceCslProvider → OpenSanctions mirror of US Trade CSL)",
        ]
    };
    println!();
    println!("Subsystem resource declaration:");
    println!("  GLEIF identity   : {}", labels[0]);
    println!("  OFAC screening   : {}", labels[1]);
    println!("  EU sanctions     : {}", labels[2]);
    println!("  Commerce CSL     : {}", labels[3]);
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

    // ───────────────── Step 2: sanctions screening (3 sources) ───────
    println!();
    let subject = SanctionsSubject::parse(&cli.counterparty)
        .map_err(|e| format!("invalid subject: {e}"))?;

    // ----- 2a: OFAC -----
    let ofac_header = if cli.mock_ok {
        "── Step 2a: OFAC-SLS (STUB; offline) ─────────────────────────────────"
    } else {
        "── Step 2a: OFAC-SLS (LIVE SDN.CSV; treasury.gov) ────────────────────"
    };
    println!("{ofac_header}");
    let ofac_request = OfacSlsRequest::Screen {
        subject: subject.clone(),
    };
    let ofac_response: embassy_ofac_sls::OfacSlsResponse = if cli.mock_ok {
        StubOfacSlsProvider.screen(&ofac_request, &ctx).await?
    } else {
        LiveOfacSlsProvider::new().screen(&ofac_request, &ctx).await?
    };
    let ofac_hit = report_sanctions_hits(
        "OFAC",
        &ofac_response.records.iter().map(|o| &o.content).collect::<Vec<_>>(),
        &mut |s| record_step(s),
    );

    // ----- 2b: EU sanctions -----
    println!();
    let eu_header = if cli.mock_ok {
        "── Step 2b: EU FSF (STUB; offline) ───────────────────────────────────"
    } else {
        "── Step 2b: EU FSF (LIVE; OpenSanctions mirror of EU consolidated) ───"
    };
    println!("{eu_header}");
    let eu_request = EuSanctionsRequest::Screen {
        subject: subject.clone(),
    };
    let eu_response: embassy_eu_sanctions::EuSanctionsResponse = if cli.mock_ok {
        StubEuSanctionsProvider.screen(&eu_request, &ctx).await?
    } else {
        LiveEuSanctionsProvider::new().screen(&eu_request, &ctx).await?
    };
    let eu_hit = report_sanctions_hits(
        "EU",
        &eu_response.records.iter().map(|o| &o.content).collect::<Vec<_>>(),
        &mut |s| record_step(s),
    );

    // ----- 2c: US Commerce CSL -----
    println!();
    let csl_header = if cli.mock_ok {
        "── Step 2c: US Commerce CSL (STUB; offline) ──────────────────────────"
    } else {
        "── Step 2c: US Commerce CSL (LIVE; OpenSanctions mirror of US Trade CSL) ─"
    };
    println!("{csl_header}");
    let csl_request = CommerceCslRequest::Screen { subject };
    let csl_response: embassy_commerce_csl::CommerceCslResponse = if cli.mock_ok {
        StubCommerceCslProvider.screen(&csl_request, &ctx).await?
    } else {
        LiveCommerceCslProvider::new()
            .screen(&csl_request, &ctx)
            .await?
    };
    let csl_hit = report_sanctions_hits(
        "Commerce CSL",
        &csl_response
            .records
            .iter()
            .map(|o| &o.content)
            .collect::<Vec<_>>(),
        &mut |s| record_step(s),
    );

    // ───────────────── Step 3: typed decision ─────────────────────────
    println!();
    println!("── Step 3: typed onboarding decision ─────────────────────────────────");
    let hit = ofac_hit || eu_hit || csl_hit;
    let decision = if hit { "DENY" } else { "ALLOW" };
    let hit_sources: Vec<&str> = [
        ("OFAC", ofac_hit),
        ("EU", eu_hit),
        ("Commerce CSL", csl_hit),
    ]
    .iter()
    .filter_map(|(name, hit)| hit.then_some(*name))
    .collect();
    let reason = if hit {
        format!("sanctions hit on {}", hit_sources.join(", "))
    } else {
        "no sanctions hits across OFAC / EU / Commerce".to_string()
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
        "Embassy LIVE (api.gleif.org + treasury.gov + EU FSF mirror + US Trade CSL mirror)"
    };
    let outcome_tag = if hit {
        format!("DENY on sanctions hit ({})", hit_sources.join(", "))
    } else {
        "ALLOW on clean screen across all three sanctions sources".to_string()
    };
    println!("Resource declaration: {outcome_tag} ({mode_tag}).");
    println!("══════════════════════════════════════════════════════════════════════");

    Ok(())
}

/// Pretty-print sanctions hits for one source and accumulate causal-
/// chain entries. Returns `true` if any hit was reported. Keeps the
/// reporting consistent across OFAC / EU / Commerce so the audit
/// output reads uniformly regardless of source.
fn report_sanctions_hits(
    source_label: &str,
    hits: &[&embassy_pack::SanctionsHit],
    record_step: &mut dyn FnMut(&str),
) -> bool {
    if hits.is_empty() {
        println!("  CLEAN  (no records returned)");
        record_step(&format!("screening: {source_label} clean (zero records)"));
        return false;
    }
    for hit in hits {
        println!("  HIT");
        println!("  list_name:     {}", hit.list_name);
        println!("  match_type:    {:?}", hit.match_type);
        println!("  match_score:   {:.2}", hit.match_score);
        if let Some(program) = &hit.list_program {
            println!("  program:       {program}");
        }
        println!("  subject_name:  {}", hit.subject_name);
        record_step(&format!(
            "screening: {} {:?} match on '{}' (score {:.2})",
            source_label, hit.match_type, hit.subject_name, hit.match_score
        ));
    }
    true
}
