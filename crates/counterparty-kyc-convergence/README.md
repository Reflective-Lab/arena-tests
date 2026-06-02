# counterparty-kyc-convergence

**Five canonical European + US government data sources, one binary, signed typed evidence.** The arena demo for what the Mosaic moat actually produces.

## Quick demo

```bash
cd ~/dev/reflective/arena-tests
cargo run -p arena-counterparty-kyc-convergence -- --smoke-test
```

Hits all five live Embassy providers against a fixed Apple Inc. reference and prints a tight summary:

```
══════════════════════════════════════════════════════════════════════
Mosaic Embassy smoke test — five canonical providers, one binary
──────────────────────────────────────────────────────────────────────
Reference: Apple Inc. (LEI HWUPKR0MPOU8FGXBT394)
Mode:      REAL LIVE (no stubs; network required)
══════════════════════════════════════════════════════════════════════

  ✓ GLEIF         live_gleif         → identity: Apple Inc.
  ✓ OFAC SDN      live_ofac_sls      → 0 hits
  ✓ EU FSF        live_eu_sanctions  → 0 hits
  ✓ Commerce CSL  live_commerce_csl  → 0 hits
  ✓ TED           live_ted           → 0 EU procurement notices (not an EU public buyer)

──────────────────────────────────────────────────────────────────────
Decision:        ALLOW
Identity:        verified (GLEIF returned 1 record)
Sanctions:       0 of 3 lists hit (OFAC SDN, EU FSF, Commerce CSL)
Enrichment:      0 EU procurement notices (not an EU public buyer)
Audit replay:    every call carries request_hash + vendor identity
Providers OK:    5/5 live (no stubs)
Wall time:       ~5s
══════════════════════════════════════════════════════════════════════
```

No env vars, no credentials, no setup. Network access is the only requirement.

## What the demo proves

Five canonical upstreams hit in one invocation:

| Source | Provider | Endpoint | Auth |
|---|---|---|---|
| **GLEIF** (Global LEI registry) | `live_gleif` | `api.gleif.org/api/v1/lei-records/{lei}` | None (CC0) |
| **OFAC SDN** (US Treasury sanctions) | `live_ofac_sls` | `sanctionslistservice.ofac.treas.gov` CSV feed | None |
| **EU FSF** (EU Consolidated Financial Sanctions) | `live_eu_sanctions` | OpenSanctions mirror (CC-BY 4.0) | None |
| **US Commerce CSL** (BIS Denied / Entity List) | `live_commerce_csl` | OpenSanctions mirror (CC-BY 4.0) | None |
| **TED** (EU Tenders Electronic Daily) | `live_ted` | `api.ted.europa.eu/v3/notices/search` (cursor-paginated) | None |

Every call returns a typed `Observation<T>` with `request_hash` for replay, `vendor` for source attribution, and a content envelope a downstream auditor can re-derive from the canonical feed.

## Customer outcome

> *"Do not onboard a sanctioned counterparty — and produce the signed evidence chain that proves we didn't."*

The decision flips ALLOW → DENY only on **typed sanctions hits**, not on LLM judgment. TED procurement history is non-binding enrichment context, not a decision driver. Identity verification is a typed `LegalEntity` from the canonical CC0 GLEIF registry.

## Why generic substitutes fail

An LLM-only counterparty-screening prompt produces a *claim* that the model checked the lists. It cannot produce:

- A `request_hash` that an auditor can re-derive from the request payload.
- A `vendor: "live_ofac_sls"` that points at the canonical US Treasury feed.
- A `MatchType::{Exact, Fuzzy, Alias}` with a `match_score` from a deterministic name-match algorithm.
- A signed observation that says "we asked OFAC at timestamp X and got this exact response."

When a regulator asks *"how did you know this name wasn't on OFAC's SDN at the time you onboarded?"*, **"the AI said it wasn't"** is not an answer. **"Here is the typed observation with content hash X retrieved from OFAC's published feed at timestamp Y"** is. The composed stack produces the second answer. An LLM-only competitor structurally cannot.

## Try the other paths

**Sanctions hit** — uses `GAZPROM` to trigger hits across all three lists:

```bash
cargo run -p arena-counterparty-kyc-convergence -- \
    --counterparty 'GAZPROM' --lei HWUPKR0MPOU8FGXBT394
```

Returns 100+ fuzzy matches across OFAC SDN, EU FSF, and US Commerce CSL — every match a typed `Observation<SanctionsHit>` with `match_score: 0.80`, `list_program`, and replay envelope. Decision flips to `DENY`.

**TED procurement history** — uses `Trafikkontoret` (Stockholm/Göteborg municipal traffic admin) to surface real EU public procurement activity:

```bash
cargo run -p arena-counterparty-kyc-convergence -- \
    --counterparty 'Trafikkontoret' --lei HWUPKR0MPOU8FGXBT394
```

Returns 5 real Swedish municipal contract notices fetched cursor-paginated from `api.ted.europa.eu/v3`.

**Offline / CI** — falls back to deterministic Embassy stubs:

```bash
cargo run -p arena-counterparty-kyc-convergence -- --mock-ok \
    --counterparty 'BLOCKED Holdings AB' --lei HWUPKR0MPOU8FGXBT394
```

Output explicitly labels every step as `CONTRACT-SHAPE` so the audit trail stays honest. The decision logic, causal chain, and provider contracts are identical; only the source data is synthetic.

## REAL-by-default declaration

Per [`~/dev/reflective/mosaic-extensions/kb/Standards/Real-by-Default Connections.md`](../../../stack/mosaic-extensions/kb/Standards/Real-by-Default%20Connections.md):

| Subsystem | Mode | Notes |
|---|---|---|
| GLEIF identity | **REAL LIVE** | `api.gleif.org` (CC0). |
| OFAC SDN screening | **REAL LIVE** | Canonical Treasury feed. |
| EU FSF screening | **REAL LIVE** | OpenSanctions mirror; canonical EU-Login endpoint configurable. |
| US Commerce CSL | **REAL LIVE** | OpenSanctions mirror; canonical trade.gov endpoint configurable with api_key. |
| TED procurement | **REAL LIVE** | `api.ted.europa.eu/v3`, cursor-paginated via `manifold::pagination`. |
| Decision logic | **LOCAL REAL** | Plain Rust over typed `SanctionsHit`. |
| Causal evidence chain | **LOCAL REAL** | In-memory; Mnemos `agentic::causal` write is a future addition (gap G3). |
| Soter SMT non-bypass proof | **DEFERRED** | `policies/no-sanctioned-onboarding.cedar` included as target; wiring requires extending `arbiter::ContextIn` with sanctions evidence. |

Default invocation is REAL LIVE. `--mock-ok` is an explicit opt-out for offline / CI runs.

## Mosaic functions exercised

- `embassy_pack::{CallContext, Observation<T>, content_hash, SanctionsSubject, SanctionsHit, MatchType, SubjectType}`
- `embassy_gleif::{LiveGleifProvider, GleifRequest, Lei, LegalEntity}`
- `embassy_ofac_sls::{LiveOfacSlsProvider, OfacSlsRequest}`
- `embassy_eu_sanctions::{LiveEuSanctionsProvider, EuSanctionsRequest}`
- `embassy_commerce_csl::{LiveCommerceCslProvider, CommerceCslRequest}`
- `embassy_ted::{LiveTedProvider, TedRequest::{Lookup, SearchByCountry, SearchByBuyerName}, ProcurementNotice}`
- `manifold::HttpFetchProvider`, `WebFetchRequest::with_body` (POST), `manifold::xml` (SOAP via vies pattern), `manifold::pagination::paginate` (TED cursor walk)

See [Capability Matrix → Embassy](../../../stack/mosaic-extensions/kb/Capability%20Matrix.md#embassy--named-source-observation) for the full surface.

## Why this crate lives in arena-tests, not atelier-showcase

Per the atelier v1.1.0 policy ([`atelier-showcase/kb/Planning/MILESTONES.md`](../../../atelier-showcase/kb/Planning/MILESTONES.md)), scenarios with `Stub*` providers in any decision path live in `arena-tests`. This crate offers `--mock-ok` as an explicit opt-out, which keeps it in arena-tests even though REAL LIVE is the default — that flag exists only for offline CI runs.

When TED, EU FSF, and Commerce CSL all have first-party canonical-source-only paths (no `--mock-ok` fallback needed), this crate becomes the template for a `REAL LIVE` showcase scenario in `atelier-showcase/scenarios/counterparty-kyc-convergence/`.

## Pressure-test target

This scenario was built to surface gaps in cross-extension wiring. Findings to date (driving follow-on commits on the always-at-the-edge policy):

1. **TED endpoint drift** — documentation said `/v3.0/notices/search`; the real path is `/v3/`. The pagination cursor only appears when the request body includes `"paginationMode": "ITERATION"`. Fixed via session-time API probing; the live test now uses strict assertions against real data.
2. **Manifold needed POST + XML support for SOAP** (vies). Landed `WebFetchRequest::with_body`, `WebFetchMethod::Post`, and `manifold::xml` extract helpers.
3. **Manifold pagination** (TED). Landed `manifold::pagination::paginate` with closure-driven strategies and `PaginationError::MaxPagesReached` as a typed signal rather than silent truncation.
4. **Mnemos `agentic::causal` is not yet pulled by any scenario or app.** The causal chain in this scenario is printed in-memory; the next move is to land an MnemosClient and write the chain as a real `agentic::causal` fact. (G3 in `shoal-meta/kb/portfolio-stretch.md`.)
5. **`arbiter::ContextIn` does not carry sanctions evidence.** Cedar policy at `policies/no-sanctioned-onboarding.cedar` is the target; wiring requires extending `ContextIn` so a SMT non-bypass proof can run.

## Falsifiable LLM-only baseline

Ask any LLM: *"Is `GAZPROM` currently on OFAC's SDN list?"* Record the answer.

Then run:

```bash
cargo run -p arena-counterparty-kyc-convergence -- \
    --counterparty 'GAZPROM' --lei HWUPKR0MPOU8FGXBT394
```

The LLM's answer is a claim. The binary's answer is a hundred-plus `Observation<SanctionsHit>` records with `match_type`, `match_score`, `list_name: "OFAC SDN"`, `vendor: "live_ofac_sls"`, and a `request_hash` that an auditor can re-derive from the request payload. The LLM cannot produce any of those artifacts; the regulator cannot accept an answer that lacks them.

That gap is the moat in one demo.
