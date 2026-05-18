//! Cross-extension smoke tests for the first Arbiter high-risk claim portfolio.
//!
//! These are not exhaustive Arbiter tests; Arbiter owns those locally. Arena
//! verifies that product-side assemblies can rely on the public Arbiter surface
//! for more than the single expense CVC5 exemplar.

use arbiter::{
    ContextIn, DataClassificationGateSuggestor, DecideRequest, EXPENSE_APPROVAL_POLICY,
    EXPENSE_NON_FINANCE_HIGH_VALUE_COMMIT_CLAIM_POLICY, FLOW_GOVERNANCE_POLICY, PolicyEngine,
    PolicyOutcome, PrincipalIn, ResourceIn, VENDOR_SELECTION_POLICY,
};
use converge_core::{AuthorityLevel, FlowAction, FlowPhase};
use converge_kernel::{Budget, ContextKey, ContextState, Engine};
use converge_pack::{DomainId, GateId, PolicyVersionId, ResourceKind};

fn budget() -> Budget {
    Budget {
        max_cycles: 3,
        max_facts: 25,
    }
}

fn request(
    domain: &str,
    resource_type: &str,
    action: FlowAction,
    amount: i64,
    human_approval_present: bool,
    gates_passed: &[&str],
    required_gates_met: bool,
) -> DecideRequest {
    DecideRequest {
        principal: PrincipalIn {
            id: format!("agent:{domain}:supervisor").into(),
            authority: AuthorityLevel::Supervisory,
            domains: vec![DomainId::new(domain)],
            policy_version: Some(PolicyVersionId::new("arena_v1")),
        },
        resource: ResourceIn {
            id: format!("{resource_type}:arena-001").into(),
            resource_type: Some(ResourceKind::new(resource_type)),
            phase: Some(FlowPhase::Commitment),
            gates_passed: Some(gates_passed.iter().copied().map(GateId::new).collect()),
        },
        action,
        context: Some(ContextIn {
            commitment_type: Some(resource_type.into()),
            amount: Some(amount),
            human_approval_present: Some(human_approval_present),
            required_gates_met: Some(required_gates_met),
        }),
        delegation_b64: None,
    }
}

#[test]
fn expense_claim_exemplar_is_reviewable_and_rejected() {
    assert!(
        !EXPENSE_NON_FINANCE_HIGH_VALUE_COMMIT_CLAIM_POLICY
            .trim()
            .is_empty(),
        "claim policy should be a reviewable public artifact"
    );

    let decision = PolicyEngine::from_policy_str(EXPENSE_APPROVAL_POLICY)
        .expect("expense policy should parse")
        .evaluate(&request(
            "operations",
            "expense",
            FlowAction::Commit,
            5_001,
            true,
            &["receipt", "manager_approval"],
            true,
        ))
        .expect("expense claim fixture should evaluate");

    assert_eq!(decision.outcome, PolicyOutcome::Reject);
}

#[test]
fn hitl_denial_stays_reject_when_approval_still_denied() {
    let decision = PolicyEngine::from_policy_str(EXPENSE_APPROVAL_POLICY)
        .expect("expense policy should parse")
        .evaluate(&request(
            "operations",
            "expense",
            FlowAction::Commit,
            5_001,
            false,
            &["receipt", "manager_approval"],
            true,
        ))
        .expect("HITL strictness fixture should evaluate");

    assert_eq!(
        decision.outcome,
        PolicyOutcome::Reject,
        "non-finance should not escalate when the approved version would still be denied"
    );
}

#[test]
fn vendor_commit_requires_due_diligence() {
    let decision = PolicyEngine::from_policy_str(VENDOR_SELECTION_POLICY)
        .expect("vendor policy should parse")
        .evaluate(&request(
            "procurement",
            "spend",
            FlowAction::Commit,
            15_000,
            true,
            &["competitive_review"],
            true,
        ))
        .expect("vendor due-diligence fixture should evaluate");

    assert_eq!(decision.outcome, PolicyOutcome::Reject);
}

#[test]
fn flow_phase_commit_requires_required_gates() {
    let decision = PolicyEngine::from_policy_str(FLOW_GOVERNANCE_POLICY)
        .expect("flow governance policy should parse")
        .evaluate(&request(
            "finance",
            "invoice",
            FlowAction::Commit,
            12_500,
            true,
            &["customer_validated"],
            true,
        ))
        .expect("flow gate fixture should evaluate");

    assert_eq!(decision.outcome, PolicyOutcome::Reject);
}

#[tokio::test]
async fn data_classification_blocks_pii_before_external_move() {
    let mut engine = Engine::with_budget(budget());
    engine.register_suggestor(DataClassificationGateSuggestor::default_patterns(
        ContextKey::Strategies,
    ));

    let mut context = ContextState::new();
    context
        .add_input(
            ContextKey::Strategies,
            "external-email-plan",
            "Send onboarding summary to jane.doe@example.com",
        )
        .expect("PII fixture should stage");

    let result = engine.run(context).await.expect("engine should run");
    assert!(result.converged);

    let constraints = result.context.get(ContextKey::Constraints);
    assert!(
        constraints
            .iter()
            .any(|fact| fact.id() == "pii-detected-external-email-plan"),
        "PII fixture should create a blocking data-classification constraint"
    );
}
