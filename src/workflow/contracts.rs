use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    Code,
    Jev,
    Openai,
}

#[derive(Clone, Deserialize, Serialize, TS)]
#[serde(deny_unknown_fields)]
pub struct WorkflowSubmission {
    pub message: String,
    pub providers: Vec<ProviderId>,
    pub client_request_id: String,
}

#[derive(Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct PlannerSetup {
    pub openrouter_api_key: String,
}

#[derive(Clone, Deserialize, Serialize, TS)]
#[serde(deny_unknown_fields)]
pub struct WorkflowResume {
    pub run_id: String,
    pub expected_plan_id: String,
    pub expected_task_id: String,
    pub client_request_id: String,
    pub message: String,
    pub employee_review: bool,
}

#[derive(Clone, Deserialize, Serialize, TS)]
pub struct PlannerOption {
    pub available: bool,
    pub model: Option<String>,
}
#[derive(Clone, Deserialize, Serialize, TS)]
pub struct DecisionProviderOption {
    pub id: ProviderId,
    pub available: bool,
    pub model: Option<String>,
    pub capability: String,
}
#[derive(Clone, Deserialize, Serialize, TS)]
pub struct WorkflowLimits {
    pub max_tasks: u32,
    pub max_steps: u32,
    pub deadline_ms: u32,
}
#[derive(Clone, Deserialize, Serialize, TS)]
pub struct WorkflowOptions {
    pub planner: PlannerOption,
    pub providers: Vec<DecisionProviderOption>,
    pub limits: WorkflowLimits,
    pub synthetic_only: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
    Running,
    Completed,
    Clarification,
    EmployeeReview,
    Failed,
    Interrupted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowTaskKind {
    Tool,
    Decision,
}
#[derive(Clone, Deserialize, Serialize, TS)]
pub struct WorkflowTask {
    pub id: String,
    pub kind: WorkflowTaskKind,
    pub name: String,
    pub depends_on: Vec<String>,
    pub state: String,
}
#[derive(Clone, Deserialize, Serialize, TS)]
pub struct WorkflowPlan {
    pub plan_id: String,
    pub tasks: Vec<WorkflowTask>,
}
#[derive(Clone, Deserialize, Serialize, TS)]
pub struct WorkflowEvent {
    pub sequence: u32,
    pub comparison_id: String,
    pub run_id: String,
    pub task_id: Option<String>,
    pub provider: Option<ProviderId>,
    pub model: Option<String>,
    pub actor: String,
    pub plan_id: Option<String>,
    pub policy_version: String,
    pub schema_version: String,
    pub stage: String,
    pub outcome: String,
    pub occurred_at_ms: f64,
    pub recorded_at_ms: Option<f64>,
}
#[derive(Clone, Default, Deserialize, Serialize, TS)]
pub struct WorkflowUsage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub attempts: u32,
    /// Gateway-reported spend (OpenRouter `usage.cost`); `None` when the route does not report it.
    #[serde(default)]
    pub cost_usd: Option<f64>,
}
#[derive(Clone, Deserialize, Serialize, TS)]
pub struct WorkflowRun {
    pub run_id: String,
    pub plan_id: String,
    pub provider: ProviderId,
    pub model: Option<String>,
    pub state: WorkflowState,
    pub reply: String,
    pub tasks: Vec<WorkflowTask>,
    pub event_trace: Vec<WorkflowEvent>,
    pub usage: WorkflowUsage,
    pub elapsed_ms: u32,
    pub failure_code: Option<String>,
}
#[derive(Clone, Deserialize, Serialize, TS)]
pub struct WorkflowComparison {
    pub comparison_id: String,
    pub input_key: String,
    pub base_plan: WorkflowPlan,
    pub runs: Vec<WorkflowRun>,
    pub mode: String,
    pub complete: bool,
    pub planner_model: Option<String>,
    pub planner_usage: WorkflowUsage,
    pub failure_code: Option<String>,
}
#[derive(Clone, Deserialize, Serialize, TS)]
pub struct WorkflowDecisionRecord {
    pub run_id: String,
    pub plan_id: String,
    pub decision_input_key: String,
    pub question_id: String,
    pub question_version: String,
    pub provider: ProviderId,
    pub model: Option<String>,
    pub policy_version: String,
    pub result: String,
    pub confidence_semantics: Option<String>,
    pub usage: WorkflowUsage,
    pub elapsed_ms: u32,
    pub complete: bool,
    pub task_id: String,
    pub attempt: u32,
    pub context_json: String,
    pub question_json: String,
}
#[derive(Clone, Deserialize, Serialize, TS)]
pub struct OperatorWorkflow {
    pub comparison: WorkflowComparison,
    pub message: String,
    pub decisions: Vec<WorkflowDecisionRecord>,
    pub protected_context_json: String,
}

pub fn declarations(config: &ts_rs::Config) -> Vec<String> {
    vec![
        ProviderId::decl(config),
        WorkflowSubmission::decl(config),
        PlannerSetup::decl(config),
        WorkflowResume::decl(config),
        PlannerOption::decl(config),
        DecisionProviderOption::decl(config),
        WorkflowLimits::decl(config),
        WorkflowOptions::decl(config),
        WorkflowState::decl(config),
        WorkflowTaskKind::decl(config),
        WorkflowTask::decl(config),
        WorkflowPlan::decl(config),
        WorkflowEvent::decl(config),
        WorkflowUsage::decl(config),
        WorkflowRun::decl(config),
        WorkflowComparison::decl(config),
        WorkflowDecisionRecord::decl(config),
        OperatorWorkflow::decl(config),
    ]
}
