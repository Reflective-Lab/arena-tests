//! Shared intent fixtures for cross-repo formation tests.
//!
//! The cases stay dependency-free so other workspaces can reuse them without
//! inheriting arena's test dependency graph.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntentCase {
    pub id: &'static str,
    pub outcome: &'static str,
    pub constraints: &'static [&'static str],
    pub forbidden_actions: &'static [&'static str],
    pub context_json: &'static str,
    pub expected_problem_class: &'static str,
    pub expected_template_id: &'static str,
}

pub const CASES: &[IntentCase] = &[
    IntentCase {
        id: "vendor-due-diligence",
        outcome: "vet and verify the shortlisted vendor before approval",
        constraints: &["budget:50000", "vendor_due_diligence_required"],
        forbidden_actions: &["approve_unverified_vendor"],
        context_json: r#"{"entity":"vendor:acme","amount_usd":42000,"jurisdiction":"EU"}"#,
        expected_problem_class: "diligence",
        expected_template_id: "organism-diligence",
    },
    IntentCase {
        id: "competitive-research",
        outcome: "research the competitive landscape for the Q3 launch",
        constraints: &["public_sources_only"],
        forbidden_actions: &[],
        context_json: r#"{"topic":"ai-crm-market","region":"nordics"}"#,
        expected_problem_class: "research",
        expected_template_id: "organism-research",
    },
    IntentCase {
        id: "candidate-evaluation",
        outcome: "evaluate and rank candidate proposals against compliance score",
        constraints: &["weighted_rubric_required"],
        forbidden_actions: &["pick_without_rubric"],
        context_json: r#"{"candidates":["alpha","bravo","charlie"],"rubric":"compliance"}"#,
        expected_problem_class: "evaluation",
        expected_template_id: "organism-evaluation",
    },
    IntentCase {
        id: "migration-planning",
        outcome: "plan the data migration rollout and dependency schedule",
        constraints: &["no_weekend_cutover", "rollback_plan_required"],
        forbidden_actions: &["drop_legacy_system_before_validation"],
        context_json: r#"{"systems":["crm","billing"],"deadline":"2026-09-30"}"#,
        expected_problem_class: "planning",
        expected_template_id: "organism-planning",
    },
    IntentCase {
        id: "vendor-approval-decision",
        outcome: "decide which vendor proposal to approve under policy",
        constraints: &["approval_policy_required", "budget:50000"],
        forbidden_actions: &["approve_without_policy_gate"],
        context_json: r#"{"options":["vendor-a","vendor-b"],"budget_usd":50000}"#,
        expected_problem_class: "decision",
        expected_template_id: "organism-decision",
    },
    // `template_id_for` documents Incident as routing to
    // `organism-decision`. The organism-decision template's metadata
    // carries the `incident` keyword so FormationGuru's keyword-driven
    // path matches naturally — no special-case control flow.
    IntentCase {
        id: "production-incident",
        outcome: "respond to the production incident and stabilize the system",
        constraints: &["incident_response_runbook", "p1_severity"],
        forbidden_actions: &["close_without_root_cause"],
        context_json: r#"{"system":"payments","severity":"p1","detected_at":"2026-05-18T03:14:00Z"}"#,
        expected_problem_class: "incident",
        expected_template_id: "organism-decision",
    },
    // `template_id_for` documents Strategy as routing to
    // `organism-research`. The organism-research template's metadata
    // carries the `strategy` keyword for the same reason.
    IntentCase {
        id: "market-strategy",
        outcome: "set our 3-year strategy and define the long-term vision for the market",
        constraints: &["board_review_required", "horizon_3_years"],
        forbidden_actions: &["commit_without_board_review"],
        context_json: r#"{"market":"enterprise-ai","horizon_years":3}"#,
        expected_problem_class: "strategy",
        expected_template_id: "organism-research",
    },
];

#[must_use]
pub const fn cases() -> &'static [IntentCase] {
    CASES
}
