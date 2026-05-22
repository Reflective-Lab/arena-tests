// Cedar policy: do not onboard a sanctioned counterparty.
//
// Status: DEFERRED. Wiring this through Arbiter's CedarAnalysisSuggestor
// + the vendored CVC5 (see `scenarios/cedar-smt-analysis` for the
// pattern) requires extending `arbiter::ContextIn` to carry
// `sanctions_hit_present` — tracked as a follow-up to this scenario.
//
// The intent of this file is to be the *target* for Soter SMT analysis
// to prove "no input shape admits an onboarding decision when a
// sanctions hit is present." Until the ContextIn extension lands, this
// policy is read for shape but not evaluated by the running scenario.

permit (
    principal,
    action == Onboarding::Action::"onboard",
    resource
)
when { context.sanctions_hit_present == false };

forbid (
    principal,
    action == Onboarding::Action::"onboard",
    resource
)
when { context.sanctions_hit_present == true };
