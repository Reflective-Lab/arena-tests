//! Cross-stack fuzzy control smoke test.
//!
//! This intentionally avoids app-specific code. The test proves the reusable
//! path: Converge runs an Organism suggestor, Organism delegates fuzzy math to
//! Prism, and Helm can carry the resulting trace as operator-control context.
//!
//! The Organism adapter used here calls Prism's Mamdani-style
//! `FuzzyInferenceEngine`: crisp inputs become fuzzy memberships; `AND` uses
//! min, `OR` uses max, `NOT` uses `1 - membership`, and multiple rules for the
//! same consequent are aggregated by max. Prism also exposes Sugeno and
//! Tsukamoto engines, plus Mamdani defuzzification helpers, but this operator
//! control example deliberately keeps the output linguistic and auditable.

use std::collections::BTreeMap;

use converge_kernel::{ContextState, Engine};
use converge_pack::{Context, ContextKey, FactPayload};
use organism_planning::{
    FuzzyConsequent, FuzzyExpression, FuzzyInferenceSuggestor, FuzzyInferenceTrace, FuzzyRule,
    FuzzySet, FuzzySuggestorError, LinguisticVariable, MembershipFunction,
};
use prio_agent_ops::{
    AdapterReceiptStatus, EvidenceReadinessStatus, FuzzyDefuzzifiedScore, FuzzyMembership,
    FuzzyReadinessTrace, FuzzyRuleActivation, JobEvidenceStatus, JobReadinessPacket,
    JobReadinessPacketInput, JobVerdict,
};
use prism::fuzzy::{
    DefuzzMethod, Domain, FuzzyInferenceOutput as PrismFuzzyInferenceOutput, MembershipDegree,
    defuzzify_mamdani,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct OperatorControlPayload {
    trace: FuzzyInferenceTrace,
}

#[derive(Debug, Clone)]
struct OperatorControlRulebook {
    variables: Vec<LinguisticVariable>,
    rules: Vec<FuzzyRule>,
}

#[derive(Debug, Clone)]
struct OperatorControlEvaluation {
    trace: FuzzyInferenceTrace,
    defuzzified_pressure: f64,
}

impl FactPayload for OperatorControlPayload {
    const FAMILY: &'static str = "arena.operator_control.fuzzy";
    const VERSION: u16 = 1;
}

const AMBIGUITY: f64 = 0.72;
const STAKEHOLDER_ALIGNMENT: f64 = 0.35;
const EVIDENCE_COMPLETENESS: f64 = 0.42;
const AUTHORITY_CLARITY: f64 = 0.55;
const TIME_PRESSURE: f64 = 0.82;
const REVERSIBILITY: f64 = 0.50;

#[derive(Debug, Clone, Copy)]
struct GovernedDecisionInputs {
    ambiguity: f64,
    stakeholder_alignment: f64,
    evidence_completeness: f64,
    authority_clarity: f64,
    time_pressure: f64,
    reversibility: f64,
}

impl GovernedDecisionInputs {
    const BASELINE: Self = Self {
        ambiguity: AMBIGUITY,
        stakeholder_alignment: STAKEHOLDER_ALIGNMENT,
        evidence_completeness: EVIDENCE_COMPLETENESS,
        authority_clarity: AUTHORITY_CLARITY,
        time_pressure: TIME_PRESSURE,
        reversibility: REVERSIBILITY,
    };

    const CLEAR: Self = Self {
        ambiguity: 0.15,
        stakeholder_alignment: 0.90,
        evidence_completeness: 0.92,
        authority_clarity: 0.95,
        time_pressure: 0.20,
        reversibility: 0.95,
    };
}

#[tokio::test]
async fn prism_fuzzy_trace_flows_through_organism_into_helm_operator_packet() {
    let evaluation = run_operator_control_evaluation().await;
    let trace = &evaluation.trace;

    assert_eq!(trace.total_rules, 4);
    assert_eq!(trace.activated_rules.len(), 4);
    assert_close(trace.input_memberships["ambiguity"]["high"], 0.8);
    assert_close(
        trace.input_memberships["stakeholder_alignment"]["low"],
        0.875,
    );
    assert_close(
        trace.input_memberships["evidence_completeness"]["weak"],
        0.7,
    );
    assert_close(
        trace.input_memberships["authority_clarity"]["unclear"],
        0.625,
    );
    assert_close(trace.input_memberships["time_pressure"]["urgent"], 0.8);
    assert_close(trace.input_memberships["reversibility"]["high"], 0.125);
    assert_close(
        trace.memberships["operator_review_pressure.operator_control_required"],
        0.8,
    );
    assert_close(
        trace.memberships["operator_review_pressure.heightened_monitoring"],
        0.7,
    );
    assert_close(
        rule_strength(
            trace,
            "high-ambiguity-low-alignment-requires-operator-control",
        ),
        0.8,
    );
    assert_close(
        rule_strength(trace, "weak-evidence-or-unclear-authority-needs-monitoring"),
        0.7,
    );
    assert_close(
        rule_strength(trace, "urgent-hard-to-reverse-requires-operator-control"),
        0.8,
    );
    assert_close(
        rule_strength(
            trace,
            "weak-evidence-under-time-pressure-requires-operator-control",
        ),
        0.7,
    );
    assert_close(evaluation.defuzzified_pressure, 0.661_066_876_083_455_9);

    let packet =
        operator_control_packet(trace, evaluation.defuzzified_pressure, "decision:arena-001");

    assert!(!packet.authorizes_domain_action);
    let helm_trace = packet
        .fuzzy_trace
        .as_ref()
        .expect("packet should carry fuzzy trace");
    assert_eq!(helm_trace.variable_key, "operator_review_pressure");
    assert_eq!(helm_trace.observed_value_basis_points, 8_000);
    assert_eq!(
        helm_membership(helm_trace, "operator_control_required"),
        8_000
    );
    assert_eq!(helm_membership(helm_trace, "heightened_monitoring"), 7_000);
    assert_eq!(
        helm_rule_strength(
            helm_trace,
            "high-ambiguity-low-alignment-requires-operator-control"
        ),
        8_000
    );
    assert_eq!(
        helm_rule_strength(
            helm_trace,
            "weak-evidence-or-unclear-authority-needs-monitoring"
        ),
        7_000
    );
    assert_eq!(
        helm_trace
            .defuzzified_score
            .as_ref()
            .expect("typed defuzzified score")
            .score_basis_points,
        6_611
    );

    let serialized = serde_json::to_string(&packet).expect("packet should serialize");
    assert!(serialized.contains("\"fuzzy_trace\""));
    assert!(
        !serialized.contains("ambiguity=0.72"),
        "Helm packet should carry distilled fuzzy trace, not raw signal text"
    );
}

#[tokio::test]
async fn clear_inputs_do_not_create_operator_control_pressure() {
    let evaluation = run_operator_control_evaluation_for(GovernedDecisionInputs::CLEAR).await;
    let trace = &evaluation.trace;

    assert_eq!(
        trace.activated_rules.len(),
        0,
        "clear, aligned, evidenced, reversible decisions should not trigger fuzzy escalation"
    );
    assert_eq!(
        basis_points(trace_membership(trace, "operator_control_required")),
        0
    );
    assert_eq!(
        basis_points(trace_membership(trace, "heightened_monitoring")),
        0
    );
    assert_close(evaluation.defuzzified_pressure, 0.0);

    let packet = operator_control_packet(
        trace,
        evaluation.defuzzified_pressure,
        "decision:arena-clear-001",
    );
    assert_eq!(packet.verdict, Some(JobVerdict::Satisfied));
    assert_eq!(
        packet.evidence_status[0].status,
        EvidenceReadinessStatus::Present
    );
    assert_eq!(
        packet
            .fuzzy_trace
            .as_ref()
            .and_then(|trace| trace.defuzzified_score.as_ref())
            .expect("typed defuzzified score")
            .score_basis_points,
        0
    );
    assert!(
        packet
            .operator_actions
            .iter()
            .any(|action| action.contains("no fuzzy operator-control escalation"))
    );
}

#[tokio::test]
async fn weak_evidence_alone_requests_monitoring_not_operator_override() {
    let evaluation = run_operator_control_evaluation_for(GovernedDecisionInputs {
        evidence_completeness: EVIDENCE_COMPLETENESS,
        ..GovernedDecisionInputs::CLEAR
    })
    .await;
    let trace = &evaluation.trace;

    assert_close(trace_membership(trace, "heightened_monitoring"), 0.7);
    assert_eq!(
        basis_points(trace_membership(trace, "operator_control_required")),
        0,
        "weak evidence should not become required operator control without another pressure"
    );
    assert_close(
        rule_strength(trace, "weak-evidence-or-unclear-authority-needs-monitoring"),
        0.7,
    );
    assert_close(evaluation.defuzzified_pressure, 0.55);

    let packet = operator_control_packet(
        trace,
        evaluation.defuzzified_pressure,
        "decision:arena-monitor-001",
    );
    assert_eq!(packet.verdict, Some(JobVerdict::Satisfied));
    assert_eq!(
        helm_membership(
            packet.fuzzy_trace.as_ref().expect("fuzzy trace"),
            "heightened_monitoring"
        ),
        7_000
    );
    assert_eq!(
        packet
            .fuzzy_trace
            .as_ref()
            .and_then(|trace| trace.defuzzified_score.as_ref())
            .expect("typed defuzzified score")
            .score_basis_points,
        5_500
    );
    assert!(
        packet
            .operator_actions
            .iter()
            .any(|action| action.contains("monitor"))
    );
}

#[tokio::test]
async fn combining_weak_evidence_with_time_pressure_changes_the_decision() {
    let evaluation = run_operator_control_evaluation_for(GovernedDecisionInputs {
        evidence_completeness: EVIDENCE_COMPLETENESS,
        time_pressure: TIME_PRESSURE,
        ..GovernedDecisionInputs::CLEAR
    })
    .await;
    let trace = &evaluation.trace;

    assert_close(
        rule_strength(
            trace,
            "weak-evidence-under-time-pressure-requires-operator-control",
        ),
        0.7,
    );
    assert_close(trace_membership(trace, "operator_control_required"), 0.7);
    assert_close(evaluation.defuzzified_pressure, 0.653_411_614_749_675_4);

    let packet = operator_control_packet(
        trace,
        evaluation.defuzzified_pressure,
        "decision:arena-escalate-001",
    );
    assert_eq!(packet.verdict, Some(JobVerdict::Blocked));
    assert_eq!(
        packet.evidence_status[0].status,
        EvidenceReadinessStatus::Concern
    );
    assert_eq!(
        packet
            .fuzzy_trace
            .as_ref()
            .and_then(|trace| trace.defuzzified_score.as_ref())
            .expect("typed defuzzified score")
            .score_basis_points,
        6_534
    );
}

#[tokio::test]
async fn educational_walkthrough_of_prism_organism_helm_fuzzy_contract() {
    println!("Step 1: stage six crisp signals in Converge context.");
    println!("  ambiguity=0.72: the decision target is underspecified.");
    println!("  stakeholder_alignment=0.35: the humans are not aligned.");
    println!("  evidence_completeness=0.42: evidence exists but is thin.");
    println!("  authority_clarity=0.55: ownership is only partly clear.");
    println!("  time_pressure=0.82: timing is becoming urgent.");
    println!("  reversibility=0.50: the move is only partly reversible.");

    println!("Step 2: Organism registers a normal Suggestor, not a side-channel.");
    println!("  The Suggestor adapter owns loop participation.");
    println!("  Prism owns membership functions and rule activation math.");
    let evaluation = run_operator_control_evaluation().await;
    let trace = &evaluation.trace;

    println!("Step 3: Prism evaluates the fuzzy memberships.");
    println!("  ambiguity.high uses a right shoulder from 0.4 to 0.8.");
    println!("  stakeholder_alignment.low uses a left shoulder from 0.3 to 0.7.");
    println!("  evidence_completeness.weak uses a left shoulder from 0.3 to 0.7.");
    println!("  authority_clarity.unclear uses a left shoulder from 0.4 to 0.8.");
    println!("  time_pressure.urgent uses a right shoulder from 0.5 to 0.9.");
    println!("  reversibility.high uses a right shoulder from 0.45 to 0.85.");
    let expected_ambiguity_high = (AMBIGUITY - 0.4) / (0.8 - 0.4);
    let expected_alignment_low = (0.7 - STAKEHOLDER_ALIGNMENT) / (0.7 - 0.3);
    let expected_evidence_weak = (0.7 - EVIDENCE_COMPLETENESS) / (0.7 - 0.3);
    let expected_authority_unclear = (0.8 - AUTHORITY_CLARITY) / (0.8 - 0.4);
    let expected_time_urgent = (TIME_PRESSURE - 0.5) / (0.9 - 0.5);
    let expected_reversibility_high = (REVERSIBILITY - 0.45) / (0.85 - 0.45);
    assert_close(
        trace.input_memberships["ambiguity"]["high"],
        expected_ambiguity_high,
    );
    assert_close(
        trace.input_memberships["stakeholder_alignment"]["low"],
        expected_alignment_low,
    );
    assert_close(
        trace.input_memberships["evidence_completeness"]["weak"],
        expected_evidence_weak,
    );
    assert_close(
        trace.input_memberships["authority_clarity"]["unclear"],
        expected_authority_unclear,
    );
    assert_close(
        trace.input_memberships["time_pressure"]["urgent"],
        expected_time_urgent,
    );
    assert_close(
        trace.input_memberships["reversibility"]["high"],
        expected_reversibility_high,
    );
    println!(
        "  ambiguity.high={expected_ambiguity_high:.3}; \
         stakeholder_alignment.low={expected_alignment_low:.3}; \
         evidence_completeness.weak={expected_evidence_weak:.3}; \
         authority_clarity.unclear={expected_authority_unclear:.3}; \
         time_pressure.urgent={expected_time_urgent:.3}; \
         reversibility.high={expected_reversibility_high:.3}"
    );

    println!("Step 4: Prism applies four fuzzy rules.");
    println!("  Mamdani AND=min, OR=max, NOT=1-x, same-consequent aggregation=max.");
    let alignment_rule_strength = expected_ambiguity_high.min(expected_alignment_low);
    let evidence_rule_strength = expected_evidence_weak.max(expected_authority_unclear);
    let hard_to_reverse = 1.0 - expected_reversibility_high;
    let time_rule_strength = expected_time_urgent.min(hard_to_reverse);
    let weak_evidence_time_rule_strength = expected_evidence_weak.min(expected_time_urgent);
    let required_strength = alignment_rule_strength
        .max(time_rule_strength)
        .max(weak_evidence_time_rule_strength);
    assert_close(
        trace.memberships["operator_review_pressure.operator_control_required"],
        required_strength,
    );
    assert_close(
        trace.memberships["operator_review_pressure.heightened_monitoring"],
        evidence_rule_strength,
    );
    println!("  alignment rule=min(0.800, 0.875)=0.800.");
    println!("  evidence/authority rule=max(0.700, 0.625)=0.700 monitoring.");
    println!("  time rule=min(0.800, NOT 0.125)=0.800.");
    println!("  weak-evidence/time rule=min(0.700, 0.800)=0.700 required.");
    println!("  operator_control_required=max(0.800, 0.800, 0.700)=0.800.");
    println!("  defuzzified centroid pressure is a blended scalar beside the trace.");
    assert_close(evaluation.defuzzified_pressure, 0.661_066_876_083_455_9);
    println!(
        "  centroid(operator_review_pressure)={:.3}",
        evaluation.defuzzified_pressure
    );

    println!("Step 5: Helm receives distilled readiness context.");
    println!("  Helm does not own the fuzzy math and does not authorize domain action.");
    println!("  It carries the trace plus the typed blended scalar as operator-control evidence.");
    let packet = operator_control_packet(
        trace,
        evaluation.defuzzified_pressure,
        "decision:arena-educational-001",
    );

    let expected_basis_points = basis_points(required_strength);
    let helm_trace = packet
        .fuzzy_trace
        .as_ref()
        .expect("Helm packet should carry fuzzy trace");
    assert_eq!(
        helm_trace.observed_value_basis_points,
        expected_basis_points
    );
    assert_eq!(
        helm_membership(helm_trace, "operator_control_required"),
        expected_basis_points
    );
    assert_eq!(
        helm_membership(helm_trace, "heightened_monitoring"),
        basis_points(evidence_rule_strength)
    );
    assert_eq!(
        helm_rule_strength(
            helm_trace,
            "urgent-hard-to-reverse-requires-operator-control"
        ),
        expected_basis_points
    );
    assert!(!packet.authorizes_domain_action);

    println!("Step 6: the serialized packet is safe to show in operator control.");
    println!("  It includes fuzzy_trace with 8000 bps for required operator control.");
    println!("  It also includes the typed defuzzified centroid score.");
    println!("  It does not include raw app transcript or raw signal text.");
    let serialized = serde_json::to_string_pretty(&packet).expect("packet should serialize");
    assert!(serialized.contains("\"fuzzy_trace\""));
    assert!(serialized.contains("\"score_basis_points\": 8000"));
    assert_eq!(
        helm_trace
            .defuzzified_score
            .as_ref()
            .expect("typed defuzzified score")
            .score_basis_points,
        6_611
    );
    assert!(!serialized.contains("ambiguity=0.72"));
    assert!(!serialized.contains("stakeholder_alignment=0.35"));
}

async fn run_operator_control_evaluation() -> OperatorControlEvaluation {
    run_operator_control_evaluation_for(GovernedDecisionInputs::BASELINE).await
}

async fn run_operator_control_evaluation_for(
    inputs: GovernedDecisionInputs,
) -> OperatorControlEvaluation {
    let rulebook = operator_control_rulebook();
    let mut engine = Engine::new();
    engine.register_suggestor(operator_control_suggestor_from_rulebook(rulebook.clone()));

    let result = engine
        .run(governed_decision_context(inputs))
        .await
        .expect("engine should run");

    assert!(result.converged);
    let trace = result
        .context
        .get(ContextKey::Evaluations)
        .iter()
        .find(|fact| fact.id().as_str() == "arena.fuzzy.operator-control-pressure")
        .expect("operator-control fuzzy proposal should be promoted")
        .require_payload::<OperatorControlPayload>()
        .expect("typed fuzzy payload should round-trip through Converge")
        .trace
        .clone();
    let defuzzified_pressure = defuzzified_pressure(&trace, &rulebook.variables);

    OperatorControlEvaluation {
        trace,
        defuzzified_pressure,
    }
}

fn governed_decision_context(inputs: GovernedDecisionInputs) -> ContextState {
    let mut context = ContextState::new();
    add_signal(
        &mut context,
        "ambiguity-signal",
        "ambiguity",
        inputs.ambiguity,
    );
    add_signal(
        &mut context,
        "alignment-signal",
        "stakeholder_alignment",
        inputs.stakeholder_alignment,
    );
    add_signal(
        &mut context,
        "evidence-signal",
        "evidence_completeness",
        inputs.evidence_completeness,
    );
    add_signal(
        &mut context,
        "authority-signal",
        "authority_clarity",
        inputs.authority_clarity,
    );
    add_signal(
        &mut context,
        "time-pressure-signal",
        "time_pressure",
        inputs.time_pressure,
    );
    add_signal(
        &mut context,
        "reversibility-signal",
        "reversibility",
        inputs.reversibility,
    );
    context
}

fn add_signal(context: &mut ContextState, id: &str, key: &str, value: f64) {
    context
        .add_input(ContextKey::Signals, id, format!("{key}={value:.2}"))
        .expect("signal should stage");
}

fn operator_control_packet(
    trace: &FuzzyInferenceTrace,
    defuzzified_pressure: f64,
    subject_ref: &str,
) -> JobReadinessPacket {
    let required_bps = basis_points(trace_membership(trace, "operator_control_required"));
    let monitoring_bps = basis_points(trace_membership(trace, "heightened_monitoring"));
    let required = required_bps >= 7_000;
    let monitoring = monitoring_bps >= 5_000;
    let verdict = if required {
        JobVerdict::Blocked
    } else {
        JobVerdict::Satisfied
    };
    let evidence_status = if required {
        EvidenceReadinessStatus::Concern
    } else {
        EvidenceReadinessStatus::Present
    };
    let operator_actions = if required {
        vec![
            "review ambiguity, evidence weakness, timing, and authority before continuing"
                .to_string(),
        ]
    } else if monitoring {
        vec!["monitor fuzzy readiness during execution; no operator-control block".to_string()]
    } else {
        vec!["no fuzzy operator-control escalation".to_string()]
    };

    JobReadinessPacket::new(JobReadinessPacketInput {
        package_id: "truth-package:arena-guided-decision-v1".to_string(),
        truth_version: "arena-guided-decision@v1".to_string(),
        domain_hint: "arena.guided-decision".to_string(),
        job_key: "guided-decision-readiness".to_string(),
        subject_ref: subject_ref.to_string(),
        adapter_receipt_id: format!("receipt:organism-fuzzy-pressure:{subject_ref}"),
        adapter_status: AdapterReceiptStatus::Succeeded,
        verdict: Some(verdict),
        authorizes_domain_action: false,
        evidence_status: vec![JobEvidenceStatus {
            clause_id: "clause:operator-control-required".to_string(),
            clause_key: "operator_control_required".to_string(),
            label: "operator control required before domain action".to_string(),
            status: evidence_status,
            fact_ids: vec!["arena.fuzzy.operator-control-pressure".to_string()],
            evidence_refs: vec!["evidence:arena.fuzzy.operator-control-pressure".to_string()],
            trace_links: vec!["trace:organism-planning/prism-fuzzy".to_string()],
            concern_record_ids: Vec::new(),
        }],
        fuzzy_trace: Some(helm_fuzzy_trace(trace, defuzzified_pressure)),
        verifier_forbidden_actions: vec![
            "do not authorize domain action from fuzzy readiness".to_string(),
            "do not replace operator review with inferred pressure".to_string(),
        ],
        operator_actions,
    })
    .expect("Helm packet should accept the Prism-backed fuzzy trace")
}

fn operator_control_suggestor_from_rulebook(
    rulebook: OperatorControlRulebook,
) -> FuzzyInferenceSuggestor<OperatorControlPayload> {
    FuzzyInferenceSuggestor::new(
        "operator-control-pressure",
        rulebook.variables,
        rulebook.rules,
        |ctx| {
            Ok(BTreeMap::from([
                ("ambiguity".to_string(), numeric_signal(ctx, "ambiguity")?),
                (
                    "stakeholder_alignment".to_string(),
                    numeric_signal(ctx, "stakeholder_alignment")?,
                ),
                (
                    "evidence_completeness".to_string(),
                    numeric_signal(ctx, "evidence_completeness")?,
                ),
                (
                    "authority_clarity".to_string(),
                    numeric_signal(ctx, "authority_clarity")?,
                ),
                (
                    "time_pressure".to_string(),
                    numeric_signal(ctx, "time_pressure")?,
                ),
                (
                    "reversibility".to_string(),
                    numeric_signal(ctx, "reversibility")?,
                ),
            ]))
        },
        |trace| {
            Ok(OperatorControlPayload {
                trace: trace.clone(),
            })
        },
        |_| "arena.fuzzy.operator-control-pressure".to_string(),
    )
    .with_proposal_id_prefix("arena.fuzzy.")
}

fn operator_control_rulebook() -> OperatorControlRulebook {
    let variables = vec![
        LinguisticVariable {
            name: "ambiguity".to_string(),
            sets: vec![FuzzySet {
                name: "high".to_string(),
                function: MembershipFunction::RightShoulder {
                    start: 0.4,
                    end: 0.8,
                },
            }],
        },
        LinguisticVariable {
            name: "stakeholder_alignment".to_string(),
            sets: vec![FuzzySet {
                name: "low".to_string(),
                function: MembershipFunction::LeftShoulder {
                    start: 0.3,
                    end: 0.7,
                },
            }],
        },
        LinguisticVariable {
            name: "evidence_completeness".to_string(),
            sets: vec![FuzzySet {
                name: "weak".to_string(),
                function: MembershipFunction::LeftShoulder {
                    start: 0.3,
                    end: 0.7,
                },
            }],
        },
        LinguisticVariable {
            name: "authority_clarity".to_string(),
            sets: vec![FuzzySet {
                name: "unclear".to_string(),
                function: MembershipFunction::LeftShoulder {
                    start: 0.4,
                    end: 0.8,
                },
            }],
        },
        LinguisticVariable {
            name: "time_pressure".to_string(),
            sets: vec![FuzzySet {
                name: "urgent".to_string(),
                function: MembershipFunction::RightShoulder {
                    start: 0.5,
                    end: 0.9,
                },
            }],
        },
        LinguisticVariable {
            name: "reversibility".to_string(),
            sets: vec![FuzzySet {
                name: "high".to_string(),
                function: MembershipFunction::RightShoulder {
                    start: 0.45,
                    end: 0.85,
                },
            }],
        },
        LinguisticVariable {
            name: "operator_review_pressure".to_string(),
            // The Mamdani trace path below consumes linguistic consequent
            // strengths directly. These output membership functions become
            // mathematically active if a caller later uses Prism's
            // `defuzzify_mamdani` helper to collapse the linguistic output
            // into one crisp score.
            sets: vec![
                FuzzySet {
                    name: "heightened_monitoring".to_string(),
                    function: MembershipFunction::Triangular {
                        min: 0.2,
                        peak: 0.55,
                        max: 0.9,
                    },
                },
                FuzzySet {
                    name: "operator_control_required".to_string(),
                    function: MembershipFunction::RightShoulder {
                        start: 0.5,
                        end: 1.0,
                    },
                },
            ],
        },
    ];
    let rules = vec![
        FuzzyRule {
            id: Some("high-ambiguity-low-alignment-requires-operator-control".to_string()),
            when: FuzzyExpression::And {
                terms: vec![
                    FuzzyExpression::Is {
                        variable: "ambiguity".to_string(),
                        set: "high".to_string(),
                    },
                    FuzzyExpression::Is {
                        variable: "stakeholder_alignment".to_string(),
                        set: "low".to_string(),
                    },
                ],
            },
            then: FuzzyConsequent {
                variable: "operator_review_pressure".to_string(),
                set: "operator_control_required".to_string(),
            },
            weight: None,
        },
        FuzzyRule {
            id: Some("weak-evidence-or-unclear-authority-needs-monitoring".to_string()),
            when: FuzzyExpression::Or {
                terms: vec![
                    FuzzyExpression::Is {
                        variable: "evidence_completeness".to_string(),
                        set: "weak".to_string(),
                    },
                    FuzzyExpression::Is {
                        variable: "authority_clarity".to_string(),
                        set: "unclear".to_string(),
                    },
                ],
            },
            then: FuzzyConsequent {
                variable: "operator_review_pressure".to_string(),
                set: "heightened_monitoring".to_string(),
            },
            weight: None,
        },
        FuzzyRule {
            id: Some("urgent-hard-to-reverse-requires-operator-control".to_string()),
            when: FuzzyExpression::And {
                terms: vec![
                    FuzzyExpression::Is {
                        variable: "time_pressure".to_string(),
                        set: "urgent".to_string(),
                    },
                    FuzzyExpression::Not {
                        term: Box::new(FuzzyExpression::Is {
                            variable: "reversibility".to_string(),
                            set: "high".to_string(),
                        }),
                    },
                ],
            },
            then: FuzzyConsequent {
                variable: "operator_review_pressure".to_string(),
                set: "operator_control_required".to_string(),
            },
            weight: None,
        },
        FuzzyRule {
            id: Some("weak-evidence-under-time-pressure-requires-operator-control".to_string()),
            when: FuzzyExpression::And {
                terms: vec![
                    FuzzyExpression::Is {
                        variable: "evidence_completeness".to_string(),
                        set: "weak".to_string(),
                    },
                    FuzzyExpression::Is {
                        variable: "time_pressure".to_string(),
                        set: "urgent".to_string(),
                    },
                ],
            },
            then: FuzzyConsequent {
                variable: "operator_review_pressure".to_string(),
                set: "operator_control_required".to_string(),
            },
            weight: None,
        },
    ];

    OperatorControlRulebook { variables, rules }
}

fn numeric_signal(ctx: &dyn Context, key: &str) -> Result<f64, FuzzySuggestorError> {
    let prefix = format!("{key}=");
    for fact in ctx.get(ContextKey::Signals) {
        let Some(text) = fact.text() else {
            continue;
        };
        if let Some(value) = text.strip_prefix(&prefix) {
            return value.parse::<f64>().map_err(FuzzySuggestorError::input);
        }
    }
    Err(FuzzySuggestorError::input(format!("missing {key} signal")))
}

fn helm_fuzzy_trace(trace: &FuzzyInferenceTrace, defuzzified_pressure: f64) -> FuzzyReadinessTrace {
    let prefix = "operator_review_pressure.";
    let memberships = trace
        .memberships
        .iter()
        .filter_map(|(key, membership)| {
            key.strip_prefix(prefix).map(|label| FuzzyMembership {
                label: label.to_string(),
                score_basis_points: basis_points(*membership),
            })
        })
        .collect::<Vec<_>>();
    let observed_value_basis_points = memberships
        .iter()
        .map(|membership| membership.score_basis_points)
        .max()
        .expect("operator-control membership should be present");

    FuzzyReadinessTrace {
        variable_key: "operator_review_pressure".to_string(),
        observed_value_basis_points,
        memberships,
        activated_rules: trace
            .activated_rules
            .iter()
            .map(|rule| FuzzyRuleActivation {
                rule_id: rule.id.clone(),
                strength_basis_points: basis_points(rule.strength),
                conclusion: rule.consequent.clone(),
            })
            .collect(),
        defuzzified_score: Some(FuzzyDefuzzifiedScore {
            method: "centroid".to_string(),
            score_basis_points: basis_points(defuzzified_pressure),
            domain_min_basis_points: 0,
            domain_max_basis_points: 10_000,
            domain_steps: 1_000,
        }),
    }
}

fn rule_strength(trace: &FuzzyInferenceTrace, rule_id: &str) -> f64 {
    trace
        .activated_rules
        .iter()
        .find(|rule| rule.id == rule_id)
        .unwrap_or_else(|| panic!("missing activated rule `{rule_id}`"))
        .strength
}

fn defuzzified_pressure(trace: &FuzzyInferenceTrace, variables: &[LinguisticVariable]) -> f64 {
    let output = PrismFuzzyInferenceOutput {
        input_memberships: BTreeMap::new(),
        memberships: trace
            .memberships
            .iter()
            .map(|(key, value)| (key.clone(), MembershipDegree::new(*value)))
            .collect(),
        activated_rules: Vec::new(),
        confidence: MembershipDegree::new(trace.confidence),
        total_rules: trace.total_rules,
    };

    defuzzify_mamdani(
        &output,
        variables,
        "operator_review_pressure",
        Domain::new(0.0, 1.0, 1_000),
        DefuzzMethod::Centroid,
    )
    .unwrap_or(0.0)
}

fn trace_membership(trace: &FuzzyInferenceTrace, label: &str) -> f64 {
    let key = format!("operator_review_pressure.{label}");
    trace.memberships.get(&key).copied().unwrap_or(0.0)
}

fn helm_membership(trace: &FuzzyReadinessTrace, label: &str) -> u16 {
    trace
        .memberships
        .iter()
        .find(|membership| membership.label == label)
        .unwrap_or_else(|| panic!("missing Helm fuzzy membership `{label}`"))
        .score_basis_points
}

fn helm_rule_strength(trace: &FuzzyReadinessTrace, rule_id: &str) -> u16 {
    trace
        .activated_rules
        .iter()
        .find(|rule| rule.rule_id == rule_id)
        .unwrap_or_else(|| panic!("missing Helm fuzzy rule `{rule_id}`"))
        .strength_basis_points
}

fn basis_points(value: f64) -> u16 {
    (value * 10_000.0).round() as u16
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {actual} to equal {expected}"
    );
}
