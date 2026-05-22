# counterparty-kyc-convergence (arena test)

Cross-extension arena test demonstrating the atomic unit of the Mosaic moat: a counterparty's identity is looked up through an Embassy port, screened against sanctions through another Embassy port, and routed to a typed onboarding decision — with every step producing a typed `Observation<T>` that carries `request_hash`, `vendor`, `match_type`, `match_score`, and replay envelope.

This crate lives in **`arena-tests/`**, not in **`atelier-showcase/`**, per the v1.1.0 atelier policy ([`atelier-showcase/kb/Planning/MILESTONES.md`](../../../atelier-showcase/kb/Planning/MILESTONES.md)): *any scenario that needs `Stub*` / `Mock*` / `Fake*` / recorded HTTP / canned provider data moves to `arena-tests` or stays unlanded until live wiring exists.* It is the [Phase C deliverable](../../../mosaic-extensions/kb/Standards/Real-by-Default%20Connections.md) of the REAL-by-default doctrine sweep — the first cross-extension binary that **refuses to run on stubs without explicit operator consent**.

When real Embassy providers for GLEIF and OFAC land, this binary becomes the template for a `REAL LIVE` scenario in `atelier-showcase/scenarios/counterparty-kyc-convergence/`.

## Customer outcome

> "Do not onboard a sanctioned counterparty — and produce the signed evidence chain that proves we didn't."

## Why generic substitutes fail

A single LLM-only counterparty-screening prompt can produce a plausible claim that the model checked the sanctions list. It cannot produce a signed `Observation<SanctionsHit>` with `request_hash`, `match_type`, `match_score`, source-list name, and replay envelope. When a regulator asks "how did you know this name wasn't on OFAC's SDN at the time you onboarded?", "the AI said it wasn't" is not an answer; "here is the typed observation with content hash X retrieved from OFAC's published feed at timestamp Y" is.

The composed stack produces the second answer. An LLM-only competitor cannot.

## REAL-by-default declaration

Per `~/dev/reflective/stack/mosaic-extensions/kb/Standards/Real-by-Default Connections.md`:

| Subsystem | Mode today | Notes |
|---|---|---|
| GLEIF identity lookup | **REAL LIVE** | `LiveGleifProvider` calling `https://api.gleif.org/api/v1/lei-records/{lei}` (CC0, no auth). Verified against Apple Inc.'s LEI `HWUPKR0MPOU8FGXBT394`. |
| OFAC SDN screening | **REAL LIVE** | `LiveOfacSlsProvider` downloading the canonical SDN.CSV from `treasury.gov` (no auth required). Verified: `GAZPROM` → fuzzy match on `GAZPROMBANK JOINT STOCK COMPANY`. |
| EU FSF screening | **REAL LIVE** | `LiveEuSanctionsProvider` defaulting to OpenSanctions' mirror of the EU Financial Sanctions Files (CC-BY 4.0, no auth). Canonical EU-Login endpoint configurable. |
| US Commerce CSL screening | **REAL LIVE** | `LiveCommerceCslProvider` defaulting to OpenSanctions' mirror of the US Trade CSL (CC-BY 4.0, no auth). Canonical trade.gov endpoint configurable (api_key required there). |
| Decision logic | **LOCAL REAL** | Plain Rust over typed `SanctionsHit` payload. |
| Causal evidence chain | **LOCAL REAL** | In-memory chain printed at end of run. Full Mnemos `agentic::causal` write awaits the Mnemos client wiring. Gap: G3 / Mnemos agentic memory dark. |
| Soter SMT proof of non-bypass | **DEFERRED** | `policies/no-sanctioned-onboarding.cedar` included as target. Wiring requires extending `arbiter::ContextIn` to carry `sanctions_hit_present`; cedar-smt-analysis is the pattern. |

**Running** (from `~/dev/reflective/stack/arena-tests`):

- `cargo run -p arena-counterparty-kyc-convergence` — **exits code 2**. There is no live Embassy provider yet; running against stubs without consent is the theatre the doctrine prohibits.
- `cargo run -p arena-counterparty-kyc-convergence -- --mock-ok` — runs end-to-end against `StubGleifProvider` + `StubOfacSlsProvider`. Output clearly labels every step as CONTRACT-SHAPE.
- Trigger the "blocked" path: `--counterparty "BLOCKED Holdings AB"` (the stub returns a synthetic hit on names containing "BLOCKED").
- Trigger the "clean" path: `--counterparty "Volvo AB"`.

## Mosaic functions pulled

- `embassy_pack::CallContext`, `Observation<T>`, `content_hash`, `SanctionsSubject`, `SanctionsHit`, `MatchType`
- `embassy_gleif::{GleifProvider, GleifRequest, GleifResponse, StubGleifProvider, types::Lei, types::LegalEntity}`
- `embassy_ofac_sls::{OfacSlsProvider, OfacSlsRequest, OfacSlsResponse, StubOfacSlsProvider}`

References:
- [Capability Matrix → Embassy](../../../mosaic-extensions/kb/Capability%20Matrix.md#embassy-embassy-ports--named-source-observation)
- [Capability Matrix → Soter (deferred)](../../../mosaic-extensions/kb/Capability%20Matrix.md#soter-soter-smt--searched-evidence-via-smt)

## Pressure-test target

This scenario was built explicitly to surface gaps in the wiring. The findings as of 2026-05-22:

1. **No live Embassy provider for either GLEIF or OFAC-SLS.** The scenario refuses to run REAL today because of this. Tracked in the Real-by-Default Connections standard's "Current known violations" list and as G1 in shoal-meta's portfolio-stretch mandate. Next: build `HttpGleifProvider` against `api.gleif.org` (free public API; no auth required) and a real OFAC SDN provider against the published feed (`https://www.treasury.gov/ofac/downloads/sdn.csv`).
2. **`arbiter::ContextIn` does not carry sanctions evidence.** Cedar policy cannot evaluate `context.sanctions_hit_present` without extending the struct. Next: add the field + `CedarAnalysisQuery::OnboardingNoSanctionedAdmitted` to `arbiter`, then wire the Soter SMT path here.
3. **Mnemos `agentic::causal` is not yet pulled by any scenario or app.** The causal chain in this scenario is printed in-memory; the next move is to land an MnemosClient and write the chain as a real `agentic::causal` fact. G3 in portfolio-stretch.

When the first two gaps close, the scenario flips from `cargo run` → exits-non-zero to `cargo run` → produces a signed evidence chain plus a Soter `unsat` report proving no sanctioned counterparty is admissible.

## Falsifiable LLM-only baseline

The README of this scenario must include a side-by-side comparison once a real Embassy provider lands. Until then, document the expected baseline failure:

- Prompt an LLM with "is `BLOCKED Holdings AB` on OFAC's SDN?". Record the answer.
- Compare with the typed `SanctionsHit` from the live provider.
- The LLM cannot produce a `content_hash`, cannot prove it consulted the canonical list at a specific time, and cannot be replayed deterministically.

That gap is the moat in one demo.
