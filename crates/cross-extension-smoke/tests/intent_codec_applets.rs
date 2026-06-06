//! Applet-shaped Intent Codec checks.
//!
//! These tests keep the "thin applet, not big app" contract observable across
//! the stack. The fixtures intentionally stay in `arena-intent-cases` so
//! Atelier, Helm, and future app repos can reuse the same business intent
//! cases without inheriting this smoke suite.

use arena_intent_cases::{AppletIntentCase, applet_cases};
use serde::Deserialize;

const APPLET_MANIFESTS: &[&str] = &[
    include_str!("../../../../KB/02-product/applets/activate-subscription.intent.json"),
    include_str!("../../../../KB/02-product/applets/refill-prepaid-ai-credits.intent.json"),
];

#[derive(Debug, Deserialize)]
struct AppletManifest {
    manifest_version: String,
    job_name: String,
    primary_job_key: String,
    status: String,
    trigger: String,
    current_workaround: String,
    functional_need: ManifestFunctionalNeed,
    emotional_need: ManifestEmotionalNeed,
    relational_need: ManifestRelationalNeed,
    failure_modes: Vec<String>,
    authority: ManifestAuthority,
    evidence_contract: ManifestEvidenceContract,
    runtime_needs: Vec<String>,
    commercial_needs: Vec<String>,
    projection: ManifestProjection,
    non_goals: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ManifestFunctionalNeed {
    outcome: String,
    inputs: Vec<String>,
    output: String,
    constraints: Vec<String>,
    success_signal: String,
}

#[derive(Debug, Deserialize)]
struct ManifestEmotionalNeed {
    operator_anxiety: String,
    desired_confidence: String,
    tolerance: String,
}

#[derive(Debug, Deserialize)]
struct ManifestRelationalNeed {
    dependent_parties: Vec<String>,
    trust_obligation: String,
    handoff_created: String,
}

#[derive(Debug, Deserialize)]
struct ManifestAuthority {
    requester: String,
    approvers: Vec<String>,
    allowed_actions: Vec<String>,
    forbidden_actions: Vec<String>,
    approval_points: Vec<String>,
    reversibility: String,
    expiry: String,
    audit_visibility: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ManifestEvidenceContract {
    required_sources: Vec<ManifestEvidenceSource>,
    disallowed_sources: Vec<String>,
    confidence_floor: String,
    conflict_policy: String,
    sensitive_fields: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ManifestEvidenceSource {
    source: String,
    freshness: String,
    authority: String,
}

#[derive(Debug, Deserialize)]
struct ManifestProjection {
    operator_view: String,
    customer_or_partner_view: String,
}

fn parsed_manifests() -> Vec<AppletManifest> {
    APPLET_MANIFESTS
        .iter()
        .map(|manifest| serde_json::from_str(manifest).expect("applet manifest parses as JSON"))
        .collect()
}

fn case_for_key(key: &str) -> &'static AppletIntentCase {
    applet_cases()
        .iter()
        .find(|case| case.truth_key == key)
        .expect("manifest key has reusable arena case")
}

fn assert_non_empty(label: &str, value: &str, case: &AppletIntentCase) {
    assert!(
        !value.trim().is_empty(),
        "{} should define {}",
        case.id,
        label
    );
}

#[test]
fn applet_intents_have_complete_jtbd_triads() {
    for case in applet_cases() {
        assert_non_empty("functional JTBD", case.jtbd.functional, case);
        assert_non_empty("emotional JTBD", case.jtbd.emotional, case);
        assert_non_empty("relational JTBD", case.jtbd.relational, case);
        assert_non_empty("success signal", case.success_signal, case);
        assert!(
            !case.current_workaround.trim().is_empty(),
            "{} should name the manual or SaaS workaround it replaces",
            case.id
        );
    }
}

#[test]
fn applet_intents_encode_authority_before_execution() {
    for case in applet_cases() {
        assert_non_empty("requester", case.authority.requester, case);
        assert!(
            !case.authority.approvers.is_empty(),
            "{} should name approvers or policies",
            case.id
        );
        assert!(
            !case.authority.allowed_actions.is_empty(),
            "{} should bound allowed actions",
            case.id
        );
        assert!(
            !case.authority.forbidden_actions.is_empty(),
            "{} should block unsafe actions",
            case.id
        );
        assert!(
            !case.authority.approval_points.is_empty(),
            "{} should name Helm approval or pause points",
            case.id
        );
        assert!(
            matches!(
                case.authority.reversibility,
                "reversible" | "partially_reversible" | "irreversible"
            ),
            "{} should use a bounded reversibility value",
            case.id
        );
    }
}

#[test]
fn applet_intents_separate_runtime_and_commerce_boundaries() {
    for case in applet_cases() {
        assert!(
            !case.runtime_needs.is_empty(),
            "{} should name Runtime Runway concerns",
            case.id
        );
        assert!(
            !case.commercial_needs.is_empty(),
            "{} should name Commerce Rails concerns",
            case.id
        );
        assert!(
            case.commercial_needs
                .iter()
                .any(|need| need.contains("outside_applet")),
            "{} should keep provider reconciliation outside the applet",
            case.id
        );
        assert!(
            case.non_goals
                .iter()
                .any(|goal| goal.contains("provider_verification")),
            "{} should reject provider verification as an applet-owned concern",
            case.id
        );
    }
}

#[test]
fn applet_intents_require_primary_evidence() {
    for case in applet_cases() {
        assert!(
            !case.evidence.is_empty(),
            "{} should require evidence before execution",
            case.id
        );
        assert!(
            case.evidence.iter().any(|item| item.authority == "primary"),
            "{} should have at least one primary evidence source",
            case.id
        );
        assert!(
            case.evidence
                .iter()
                .all(|item| !item.source.trim().is_empty() && !item.freshness.trim().is_empty()),
            "{} evidence sources should name source and freshness",
            case.id
        );
    }
}

#[test]
fn applet_intents_name_one_truth_and_small_projection() {
    for case in applet_cases() {
        assert_non_empty("truth key", case.truth_key, case);
        assert_non_empty("job name", case.job_name, case);
        assert_non_empty("trigger", case.trigger, case);
        assert_non_empty("projection", case.projection, case);
        assert!(
            !case.inputs.is_empty(),
            "{} should define the input envelope",
            case.id
        );
        assert!(
            !case.failure_modes.is_empty(),
            "{} should name failure modes",
            case.id
        );
    }
}

#[test]
fn machine_readable_manifests_match_reusable_applet_cases() {
    let manifests = parsed_manifests();
    assert_eq!(
        manifests.len(),
        applet_cases().len(),
        "every reusable applet case should have a manifest fixture"
    );

    for manifest in manifests {
        let case = case_for_key(&manifest.primary_job_key);
        assert_eq!(manifest.manifest_version, "intent-codec-applet.v1");
        assert_eq!(manifest.status, "code-backed");
        assert_eq!(manifest.job_name, case.job_name);
        assert_eq!(manifest.trigger, case.trigger);
        assert_eq!(manifest.functional_need.outcome, case.jtbd.functional);
        assert_eq!(
            manifest.functional_need.inputs,
            case.inputs
                .iter()
                .map(|input| (*input).to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            manifest.authority.reversibility,
            case.authority.reversibility
        );
        assert!(
            !manifest.current_workaround.trim().is_empty(),
            "{} manifest should name the replaced workaround",
            manifest.primary_job_key
        );
        assert!(
            !manifest.functional_need.output.trim().is_empty()
                && !manifest.functional_need.constraints.is_empty()
                && !manifest.functional_need.success_signal.trim().is_empty(),
            "{} manifest should keep the functional job executable",
            manifest.primary_job_key
        );
        assert!(
            !manifest.emotional_need.operator_anxiety.trim().is_empty()
                && !manifest.emotional_need.desired_confidence.trim().is_empty()
                && !manifest.emotional_need.tolerance.trim().is_empty(),
            "{} manifest should preserve the emotional JTBD lane",
            manifest.primary_job_key
        );
        assert!(
            !manifest.relational_need.dependent_parties.is_empty()
                && !manifest.relational_need.trust_obligation.trim().is_empty()
                && !manifest.relational_need.handoff_created.trim().is_empty(),
            "{} manifest should preserve the relational JTBD lane",
            manifest.primary_job_key
        );
        assert!(
            !manifest.failure_modes.is_empty()
                && !manifest.authority.requester.trim().is_empty()
                && !manifest.authority.approvers.is_empty()
                && !manifest.authority.allowed_actions.is_empty()
                && !manifest.authority.forbidden_actions.is_empty()
                && !manifest.authority.approval_points.is_empty()
                && !manifest.authority.expiry.trim().is_empty()
                && !manifest.authority.audit_visibility.is_empty(),
            "{} manifest should encode authority before execution",
            manifest.primary_job_key
        );
        assert!(
            manifest
                .evidence_contract
                .required_sources
                .iter()
                .any(|source| source.authority == "primary"),
            "{} manifest should require primary evidence",
            manifest.primary_job_key
        );
        assert!(
            manifest
                .evidence_contract
                .required_sources
                .iter()
                .all(|source| {
                    !source.source.trim().is_empty() && !source.freshness.trim().is_empty()
                })
                && !manifest.evidence_contract.disallowed_sources.is_empty()
                && !manifest
                    .evidence_contract
                    .confidence_floor
                    .trim()
                    .is_empty()
                && manifest.evidence_contract.conflict_policy == "stop"
                && !manifest.evidence_contract.sensitive_fields.is_empty(),
            "{} manifest should make evidence boundaries machine-readable",
            manifest.primary_job_key
        );
        assert!(
            !manifest.runtime_needs.is_empty()
                && !manifest.commercial_needs.is_empty()
                && !manifest.projection.operator_view.trim().is_empty()
                && !manifest
                    .projection
                    .customer_or_partner_view
                    .trim()
                    .is_empty()
                && !manifest.non_goals.is_empty(),
            "{} manifest should stay applet-sized",
            manifest.primary_job_key
        );
    }
}
