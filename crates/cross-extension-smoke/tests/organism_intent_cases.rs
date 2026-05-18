//! Shared intent-case validation for Organism's front-half routing.
//!
//! Arena owns the reusable case table; other repos can import
//! `arena-intent-cases` and run richer participant/formation assertions
//! against the same business intents.

use arena_intent_cases::{IntentCase, cases};
use chrono::{TimeZone, Utc};
use converge_kernel::ContextState;
use converge_kernel::admission::{AdmissionActor, AdmissionActorKind, AdmissionSource};
use converge_kernel::formation::SuggestorCapability;
use organism_intent::problem::classify;
use organism_pack::{ForbiddenAction, IntentPacket};
use organism_runtime::{Runtime, standard_formation_catalog};

fn expiry() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2027, 1, 15, 12, 0, 0)
        .single()
        .expect("fixed expiry should be valid")
}

fn intent(case: &IntentCase) -> IntentPacket {
    let mut intent = IntentPacket::new(case.outcome, expiry());
    intent.constraints = case.constraints.iter().map(|c| (*c).to_string()).collect();
    intent.forbidden = case
        .forbidden_actions
        .iter()
        .map(|action| ForbiddenAction {
            action: (*action).to_string(),
            reason: "arena fixture forbidden action".to_string(),
        })
        .collect();
    intent.context =
        serde_json::from_str(case.context_json).expect("intent fixture context must be JSON");
    intent
}

fn host_capabilities() -> Vec<SuggestorCapability> {
    vec![
        SuggestorCapability::LlmReasoning,
        SuggestorCapability::KnowledgeRetrieval,
        SuggestorCapability::Analytics,
        SuggestorCapability::Optimization,
        SuggestorCapability::PolicyEnforcement,
        SuggestorCapability::HumanInTheLoop,
        SuggestorCapability::ExperienceLearning,
    ]
}

#[test]
fn shared_intents_classify_to_expected_problem_classes() {
    for case in cases() {
        let classification = classify(&intent(case));
        assert_eq!(
            classification.class.as_str(),
            case.expected_problem_class,
            "{} should classify as {}",
            case.id,
            case.expected_problem_class
        );
    }
}

#[test]
fn shared_intents_select_expected_standard_formations() {
    let runtime = Runtime::new();
    let catalog = standard_formation_catalog();
    let caps = host_capabilities();

    for case in cases() {
        let selection = runtime
            .select_formation(&intent(case), &catalog, &caps)
            .unwrap_or_else(|err| panic!("{} should select a formation: {err}", case.id));

        assert_eq!(
            selection.primary.id(),
            case.expected_template_id,
            "{} should select {}",
            case.id,
            case.expected_template_id
        );
    }
}

#[test]
fn shared_intents_pass_runtime_admission() {
    let runtime = Runtime::new();
    let actor = AdmissionActor::new("arena-intent-cases", AdmissionActorKind::System)
        .expect("test actor should be valid");
    let source = AdmissionSource::new("arena-tests").expect("test source should be valid");

    for case in cases() {
        let mut context = ContextState::new();
        let receipt = runtime
            .admit_intent(&intent(case), actor.clone(), source.clone(), &mut context)
            .unwrap_or_else(|err| panic!("{} should pass admission: {err}", case.id));

        assert_eq!(
            receipt.key(),
            converge_kernel::ContextKey::Seeds,
            "{} should admit under Seeds",
            case.id
        );
    }
}

#[test]
fn shared_intent_expiry_is_stable_future_date() {
    let fixed_now = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
        .single()
        .expect("fixed now should be valid");
    for case in cases() {
        assert!(
            !intent(case).is_expired(fixed_now),
            "{} should not depend on wall-clock flakiness",
            case.id
        );
    }
}
