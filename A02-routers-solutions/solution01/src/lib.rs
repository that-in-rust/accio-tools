//! PIE Tauri-ready Rust core.

mod commands;
mod deterministic_verifier;
mod diagnostic_logger;
mod error;
mod judge_runner;
mod llm_verifier;
mod model;
mod openai_client;
mod openai_config;
mod patch_runner;
mod prompt_normalizer;
mod prompt_parser;
mod recommendation_engine;
mod reporter;
mod version_store;

pub use commands::{
    analyze_prompt, analyze_updated_prompt, apply_selected_fixes, get_app_readiness,
    reverify_prompt, validate_openai_key, verify_and_export_update,
};
pub use deterministic_verifier::check_update_deterministically;
pub use diagnostic_logger::DiagnosticLogger;
pub use error::{AppError, PieError};
pub use judge_runner::{build_judge_request, deterministic_finding_groups};
pub use llm_verifier::build_llm_verify_request;
pub use model::{
    AnalysisResult, AnalyzePromptRequest, AppReadiness, AppReadinessRequest, AssignmentPrompt,
    DeterministicCheck, Finding, FindingGroup, FindingSeverity, FixMode, JudgeRequest,
    JudgeSection, LlmVerifyRequest, NormalizedPrompt, OpenAiRuntimeConfig, PatchCandidate,
    PatchFailureMode, PatchRequest, PromptParagraph, PromptPatchMode, PromptSection, PromptVersion,
    ReverifyFindingStatus, ReverifyPromptRequest, ReverifyPromptResult, ReverifyStatus,
    SelectedFixesRequest, SelectedFixesResult, SemanticCheck, ToolDefinition, VerificationExport,
    VersionKind,
};
pub use openai_client::{
    fix_patch_instructions, judge_prompt_instructions, map_openai_status_error,
    parse_judge_response_text, parse_reverify_response_text, reverify_prompt_instructions,
    FakeOpenAiClient, OpenAiClient, OpenAiHttpClient,
};
pub use openai_config::{model_label_text, OPENAI_MODEL_ID};
pub use patch_runner::{
    build_patch_request, create_prompt_candidate_from_patch, create_prompt_candidate_from_rewrite,
    create_prompt_candidate_json, ideal_voice_prompt_structure_json, PromptExportCandidate,
    PromptPatchEnvelope, PromptReplacement,
};
pub use prompt_normalizer::normalize_prompt_sections_model;
pub use prompt_parser::parse_assignment_prompt_json;
pub use recommendation_engine::{
    collect_recommended_patch_ids, create_recommended_action_copy, RecommendedAction,
};
pub use reporter::{export_findings_markdown_text, export_verification_report};
pub use version_store::SqliteVersionStore;
