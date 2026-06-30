use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssignmentPrompt {
    pub agent_name: String,
    pub model: String,
    pub general_prompt: String,
    pub general_tools: Vec<ToolDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    pub description: String,
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: Value,
    pub parameters: ToolParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolParameters {
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default)]
    pub properties: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenAiRuntimeConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedPrompt {
    pub raw_text: String,
    pub sections: Vec<PromptSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptSection {
    pub section_id: String,
    pub section_name: String,
    pub better_home: String,
    pub stability: String,
    pub judge_section: JudgeSection,
    pub belongs_in_final_prompt: bool,
    pub paragraphs: Vec<PromptParagraph>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptParagraph {
    pub paragraph_id: String,
    pub source_order: usize,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum JudgeSection {
    Structure,
    ToolGaps,
    WorkflowOrder,
    SafetyPhi,
    Verification,
}

impl JudgeSection {
    pub fn as_label(&self) -> &'static str {
        match self {
            Self::Structure => "Structure",
            Self::ToolGaps => "Tool Gaps",
            Self::WorkflowOrder => "Workflow Order",
            Self::SafetyPhi => "Safety / PHI",
            Self::Verification => "Verification",
        }
    }

    fn from_model_text(value: &str) -> Self {
        let normalized = normalize_model_label(value);

        if normalized.contains("tool") || normalized.contains("capability") {
            Self::ToolGaps
        } else if normalized.contains("workflow")
            || normalized.contains("order")
            || normalized.contains("sequence")
        {
            Self::WorkflowOrder
        } else if normalized.contains("phi")
            || normalized.contains("hipaa")
            || normalized.contains("safety")
        {
            Self::SafetyPhi
        } else if normalized.contains("verify")
            || normalized.contains("evidence")
            || normalized.contains("eval")
            || normalized.contains("regression")
        {
            Self::Verification
        } else {
            Self::Structure
        }
    }
}

impl<'de> Deserialize<'de> for JudgeSection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::from_model_text(&value))
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum FindingSeverity {
    High,
    Medium,
    Low,
}

impl FindingSeverity {
    pub fn as_label(&self) -> &'static str {
        match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        }
    }

    fn from_model_text(value: &str) -> Self {
        let normalized = normalize_model_label(value);

        if normalized.contains("critical")
            || normalized.contains("high")
            || normalized.contains("urgent")
        {
            Self::High
        } else if normalized.contains("low")
            || normalized.contains("minor")
            || normalized.contains("small")
        {
            Self::Low
        } else {
            Self::Medium
        }
    }
}

impl<'de> Deserialize<'de> for FindingSeverity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::from_model_text(&value))
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum FixMode {
    AutoFixable,
    HumanReviewOnly,
    Backlog,
}

impl FixMode {
    pub fn as_label(&self) -> &'static str {
        match self {
            Self::AutoFixable => "auto-fixable",
            Self::HumanReviewOnly => "human-review-only",
            Self::Backlog => "backlog",
        }
    }

    fn from_model_text(value: &str) -> Self {
        let normalized = normalize_model_label(value);

        if normalized.contains("backlog")
            || normalized.contains("later")
            || normalized.contains("future")
        {
            Self::Backlog
        } else if normalized.contains("human")
            || normalized.contains("review")
            || normalized.contains("manual")
            || normalized.contains("approval")
            || normalized.contains("escalat")
        {
            Self::HumanReviewOnly
        } else {
            Self::AutoFixable
        }
    }
}

impl<'de> Deserialize<'de> for FixMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::from_model_text(&value))
    }
}

fn normalize_model_label(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace(['_', '-', '/', '.'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Finding {
    pub finding_id: String,
    pub title: String,
    pub severity: FindingSeverity,
    pub prompt_evidence: String,
    pub impact: String,
    pub suggested_fix: String,
    pub fix_mode: FixMode,
    pub verification_scenario: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FindingGroup {
    pub section: JudgeSection,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JudgeRequest {
    #[serde(skip_serializing)]
    pub api_key: String,
    pub model: String,
    pub agent_name: String,
    pub required_sections: Vec<JudgeSection>,
    pub normalized_sections_count: usize,
    pub normalized_prompt: NormalizedPrompt,
    pub tool_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PromptPatchMode {
    TargetedPatch,
    SectionRewrite,
    CleanScaffold,
    HumanReviewOnly,
}

impl PromptPatchMode {
    pub fn as_label(&self) -> &'static str {
        match self {
            Self::TargetedPatch => "targeted_patch",
            Self::SectionRewrite => "section_rewrite",
            Self::CleanScaffold => "clean_scaffold",
            Self::HumanReviewOnly => "human_review_only",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatchRequest {
    #[serde(skip_serializing)]
    pub api_key: String,
    pub model: String,
    pub original_prompt_json: String,
    pub selected_finding_ids: Vec<String>,
    pub evidence_spans: Vec<String>,
    pub patch_instructions: Vec<String>,
    pub patch_mode: PromptPatchMode,
    pub patch_mode_reason: String,
    pub ideal_prompt_structure_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatchCandidate {
    pub updated_prompt_json: String,
    pub applied_finding_ids: Vec<String>,
    pub patch_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatchFailureMode {
    SelectedFindingFailed { finding_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VersionKind {
    Original,
    Updated,
}

impl VersionKind {
    pub fn as_label(&self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Updated => "updated",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptVersion {
    pub version_name: String,
    pub kind: VersionKind,
    pub payload_size: usize,
    pub analysis_run_id: String,
    pub model_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeterministicCheck {
    pub scenario_id: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticCheck {
    pub scenario_id: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppReadinessRequest {
    pub api_key: Option<String>,
    pub storage_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppReadiness {
    pub api_key_ready: bool,
    pub storage_ready: bool,
    pub analyze_enabled: bool,
    pub model_label: String,
    pub readiness_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalyzePromptRequest {
    pub filename: String,
    pub prompt_json: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalysisResult {
    pub prompt_version_name: String,
    pub model_label: String,
    pub finding_groups: Vec<FindingGroup>,
    pub normalized_prompt: NormalizedPrompt,
    pub action_log: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectedFixesRequest {
    pub original_prompt_json: String,
    pub finding_groups: Vec<FindingGroup>,
    pub selected_finding_ids: Vec<String>,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectedFixesResult {
    pub candidate: PatchCandidate,
    pub model_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LlmVerifyRequest {
    #[serde(skip_serializing)]
    pub api_key: String,
    pub model: String,
    pub original_prompt_json: String,
    pub updated_prompt_json: String,
    pub selected_finding_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReverifyPromptRequest {
    #[serde(skip_serializing)]
    pub api_key: String,
    pub original_prompt_json: String,
    pub updated_prompt_json: String,
    pub previous_finding_groups: Vec<FindingGroup>,
    pub updated_version_name: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum ReverifyStatus {
    Fixed,
    StillFailing,
    Unknown,
}

impl ReverifyStatus {
    pub fn as_label(&self) -> &'static str {
        match self {
            Self::Fixed => "Fixed",
            Self::StillFailing => "Still failing",
            Self::Unknown => "Unknown",
        }
    }

    fn from_model_text(value: &str) -> Self {
        let normalized = normalize_model_label(value);

        if normalized.contains("fixed")
            || normalized.contains("resolved")
            || normalized.contains("pass")
            || normalized.contains("green")
        {
            Self::Fixed
        } else if normalized.contains("still")
            || normalized.contains("fail")
            || normalized.contains("unresolved")
            || normalized.contains("regress")
            || normalized.contains("red")
        {
            Self::StillFailing
        } else {
            Self::Unknown
        }
    }
}

impl<'de> Deserialize<'de> for ReverifyStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::from_model_text(&value))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReverifyFindingStatus {
    pub finding_id: String,
    pub status: ReverifyStatus,
    pub status_label: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReverifyPromptResult {
    pub prompt_version_name: String,
    pub model_label: String,
    pub finding_statuses: Vec<ReverifyFindingStatus>,
    pub action_log: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationExport {
    pub markdown_report: String,
    pub updated_prompt_json: String,
    pub deterministic_checks: Vec<DeterministicCheck>,
    pub semantic_checks: Vec<SemanticCheck>,
}
