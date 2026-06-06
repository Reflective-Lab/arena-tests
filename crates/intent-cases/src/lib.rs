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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JtbdTriad {
    pub functional: &'static str,
    pub emotional: &'static str,
    pub relational: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvidenceRequirement {
    pub source: &'static str,
    pub freshness: &'static str,
    pub authority: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthorityEnvelope {
    pub requester: &'static str,
    pub approvers: &'static [&'static str],
    pub allowed_actions: &'static [&'static str],
    pub forbidden_actions: &'static [&'static str],
    pub approval_points: &'static [&'static str],
    pub reversibility: &'static str,
    pub expiry: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppletIntentCase {
    pub id: &'static str,
    pub truth_key: &'static str,
    pub job_name: &'static str,
    pub trigger: &'static str,
    pub current_workaround: &'static str,
    pub inputs: &'static [&'static str],
    pub jtbd: JtbdTriad,
    pub success_signal: &'static str,
    pub failure_modes: &'static [&'static str],
    pub authority: AuthorityEnvelope,
    pub evidence: &'static [EvidenceRequirement],
    pub runtime_needs: &'static [&'static str],
    pub commercial_needs: &'static [&'static str],
    pub projection: &'static str,
    pub non_goals: &'static [&'static str],
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

pub const APPLET_CASES: &[AppletIntentCase] = &[
    AppletIntentCase {
        id: "activate-subscription",
        truth_key: "activate-subscription",
        job_name: "Activate paid subscription",
        trigger: "subscription_activation_requested",
        current_workaround: "Billing or RevOps manually checks payment, plan, entitlements, and opening balance before enabling customer access.",
        inputs: &[
            "organization_id",
            "subscription_id",
            "catalog_item_id",
            "payment_confirmed",
        ],
        jtbd: JtbdTriad {
            functional: "Turn an agreed commercial plan into active subscription, entitlement, and auditable opening financial state.",
            emotional: "Avoid customer embarrassment from paid-but-locked access or premature access before terms are trustworthy.",
            relational: "Customer admin, support, finance, RevOps, and partner owner need the same explainable activation state.",
        },
        success_signal: "Subscription activates with projected entitlement state and no manual approval required.",
        failure_modes: &[
            "grant_entitlement_without_payment_confirmation",
            "activate_wrong_organization_subscription",
            "store_provider_id_as_canonical_entitlement",
            "hide_activation_exception_in_app_state",
        ],
        authority: AuthorityEnvelope {
            requester: "commerce/runtime subscription activation envelope",
            approvers: &["commerce policy", "billing operator for blocked activation"],
            allowed_actions: &[
                "activate_subscription",
                "derive_entitlements",
                "open_approval_workflow",
                "project_activation_receipt",
            ],
            forbidden_actions: &[
                "grant_without_payment_confirmation",
                "own_provider_reconciliation",
                "bypass_helm_approval",
            ],
            approval_points: &["payment_missing_or_false", "non_standard_plan_terms"],
            reversibility: "partially_reversible",
            expiry: "payment_or_activation_event_replay_window",
        },
        evidence: &[
            EvidenceRequirement {
                source: "Commerce Rails verified subscription contract",
                freshness: "current_at_activation_time",
                authority: "primary",
            },
            EvidenceRequirement {
                source: "runtime or commerce payment confirmation envelope",
                freshness: "within_replay_window",
                authority: "primary",
            },
            EvidenceRequirement {
                source: "catalog plan definition",
                freshness: "current_at_activation_time",
                authority: "primary",
            },
        ],
        runtime_needs: &[
            "normalized_event_ingress",
            "provider_secret_handling_outside_applet",
            "activation_telemetry",
            "durable_workflow_references",
        ],
        commercial_needs: &[
            "subscription_lifecycle_state",
            "catalog_plan_resolution",
            "entitlement_grant",
            "opening_ledger_context",
            "provider_reconciliation_outside_applet",
        ],
        projection: "operator sees subscription, plan, activation state, entitlements, workflow or approval IDs, and stop reason",
        non_goals: &[
            "billing_dashboard",
            "provider_verification",
            "canonical_entitlement_store",
            "unrelated_billing_jobs",
        ],
    },
    AppletIntentCase {
        id: "refill-prepaid-ai-credits",
        truth_key: "refill-prepaid-ai-credits",
        job_name: "Refill prepaid AI credits",
        trigger: "prepaid_top_up_settled",
        current_workaround: "A billing operator manually confirms payment and updates prepaid usage balance.",
        inputs: &[
            "organization_id",
            "subscription_id",
            "amount_minor",
            "currency_code",
            "payment_reference",
            "payment_status",
        ],
        jtbd: JtbdTriad {
            functional: "Apply a settled top-up to prepaid AI credit balances with financial traceability.",
            emotional: "Avoid service interruption from stale balance while preventing credit grants for risky or unsettled payment.",
            relational: "Customer admin, finance, support, runtime metering, and partner owner need an explainable balance change.",
        },
        success_signal: "Confirmed top-up appears as a ledger-backed credit grant and entitlement balance increases for the correct account.",
        failure_modes: &[
            "grant_credit_without_confirmed_payment",
            "increase_wrong_subscription_balance",
            "lose_payment_to_ledger_traceability",
            "hide_risk_review_in_app_state",
        ],
        authority: AuthorityEnvelope {
            requester: "commerce/runtime prepaid top-up envelope",
            approvers: &["commerce policy", "billing operator for risky top-up"],
            allowed_actions: &[
                "grant_prepaid_credit",
                "append_ledger_entry",
                "open_approval_workflow",
                "project_credit_receipt",
            ],
            forbidden_actions: &[
                "grant_pending_payment",
                "own_provider_reconciliation",
                "bypass_risk_review",
            ],
            approval_points: &["pending_payment", "unusual_top_up_size_or_risk_signal"],
            reversibility: "partially_reversible",
            expiry: "payment_event_replay_window",
        },
        evidence: &[
            EvidenceRequirement {
                source: "Commerce Rails verified top-up event",
                freshness: "within_replay_window",
                authority: "primary",
            },
            EvidenceRequirement {
                source: "active subscription commercial commitment",
                freshness: "current_at_top_up_time",
                authority: "primary",
            },
            EvidenceRequirement {
                source: "ledger credit grant receipt",
                freshness: "created_during_truth_execution",
                authority: "primary",
            },
        ],
        runtime_needs: &[
            "normalized_top_up_ingress",
            "provider_secret_handling_outside_applet",
            "balance_change_telemetry",
            "durable_ledger_reference",
        ],
        commercial_needs: &[
            "payment_settlement_state",
            "subscription_commitment",
            "credit_entitlement_balance",
            "ledger_credit_grant",
            "provider_reconciliation_outside_applet",
        ],
        projection: "operator sees payment status, grant amount, subscription, credit entitlement, ledger entry, and stop reason",
        non_goals: &[
            "usage_metering_engine",
            "provider_verification",
            "canonical_credit_store",
            "subscription_activation",
        ],
    },
];

#[must_use]
pub const fn cases() -> &'static [IntentCase] {
    CASES
}

#[must_use]
pub const fn applet_cases() -> &'static [AppletIntentCase] {
    APPLET_CASES
}
