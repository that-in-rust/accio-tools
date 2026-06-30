use benchmark_eval_metrics_runner::{
    evaluate_routing_subset_metrics as evaluate_subset_metrics_inner, load_bundled_evaluation_pack,
    MetricReportOutputData, RoutingMetricsRequestData,
};
use candidate_judge_openai_adapter::{
    judge_candidate_tools_top, JudgeCandidateRequestData, JudgeDecisionOutputData,
};
use catalog_router_core_engine::{
    rank_lexical_tools_baseline, CandidateEvidenceCardData, RouterModeNameData,
    RouterTypedErrorKind,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Router(#[from] RouterTypedErrorKind),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouterAppReadinessData {
    pub judge_key_ready: bool,
    pub route_preview_enabled: bool,
    pub judged_route_enabled: bool,
    pub model_label: String,
    pub readiness_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteToolsRequestData {
    pub dataset_path: Option<String>,
    pub query: String,
    pub recent_context: Option<String>,
    pub router_mode: RouterModeNameData,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteToolsResponseData {
    pub route_label: String,
    pub candidates: Vec<CandidateEvidenceCardData>,
    pub judge_decision: Option<JudgeDecisionOutputData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationPackFileData {
    pub filename: String,
    pub content: String,
}

pub fn validate_judge_api_key(api_key: Option<String>) -> RouterAppReadinessData {
    let judge_key_ready = api_key
        .as_deref()
        .map(str::trim)
        .filter(|value| value.len() >= 8)
        .is_some();
    RouterAppReadinessData {
        judge_key_ready,
        route_preview_enabled: true,
        judged_route_enabled: judge_key_ready,
        model_label: std::env::var("OPENAI_ROUTER_JUDGE_MODEL")
            .unwrap_or_else(|_| "mock-router-judge".to_string()),
        readiness_message: if judge_key_ready {
            "Judge key accepted for local route execution.".to_string()
        } else {
            "CPU preview is available; judged route needs a key.".to_string()
        },
    }
}

pub fn run_cpu_preview_only(
    request: RouteToolsRequestData,
) -> Result<RouteToolsResponseData, AppError> {
    let dataset_path = resolve_dataset_path_value(request.dataset_path.as_deref());
    let pack = load_bundled_evaluation_pack(&dataset_path)?;
    let candidates = rank_lexical_tools_baseline(&request.query, &pack.tools, 2.0, 5)?;
    Ok(RouteToolsResponseData {
        route_label: "cpu_only_debug_preview".to_string(),
        candidates,
        judge_decision: None,
    })
}

pub fn route_tools_for_query(
    request: RouteToolsRequestData,
) -> Result<RouteToolsResponseData, AppError> {
    let mut response = run_cpu_preview_only(request.clone())?;
    if request
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .is_some()
    {
        let judge_request = JudgeCandidateRequestData {
            query: request.query,
            recent_context: request.recent_context,
            candidates: response.candidates.clone(),
        };
        response.judge_decision = Some(judge_candidate_tools_top(&judge_request)?);
        response.route_label = "judged_route".to_string();
    }
    Ok(response)
}

pub fn evaluate_routing_subset_metrics(
    request: RoutingMetricsRequestData,
) -> Result<MetricReportOutputData, AppError> {
    evaluate_subset_metrics_inner(request).map_err(AppError::from)
}

pub fn download_evaluation_pack_files(
    dataset_path: Option<String>,
) -> Result<Vec<EvaluationPackFileData>, AppError> {
    let path = resolve_dataset_path_value(dataset_path.as_deref());
    let pack = load_bundled_evaluation_pack(&path)?;
    Ok(vec![
        EvaluationPackFileData {
            filename: "tools.json".to_string(),
            content: serde_json::to_string_pretty(&pack.tools)
                .map_err(RouterTypedErrorKind::from)?,
        },
        EvaluationPackFileData {
            filename: "queries.json".to_string(),
            content: serde_json::to_string_pretty(&pack.queries)
                .map_err(RouterTypedErrorKind::from)?,
        },
        EvaluationPackFileData {
            filename: "manifest.json".to_string(),
            content: serde_json::to_string_pretty(&pack.manifest)
                .map_err(RouterTypedErrorKind::from)?,
        },
    ])
}

pub fn export_route_evidence_report(response: &RouteToolsResponseData) -> String {
    let mut lines = vec![
        "# Tool Router Evidence Report".to_string(),
        String::new(),
        format!("route_label: {}", response.route_label),
        format!("candidate_count: {}", response.candidates.len()),
    ];
    if let Some(decision) = &response.judge_decision {
        lines.push(format!("decision: {:?}", decision.decision));
        lines.push(format!("selected_tool_id: {:?}", decision.selected_tool_id));
        lines.push(format!("confidence: {:.3}", decision.confidence));
        lines.push(format!("reason: {}", decision.reason));
    }
    lines.join("\n")
}

pub fn export_diagnostic_logs_text() -> String {
    "# Tool Router Diagnostic Log\n\nNo persisted diagnostic events yet.".to_string()
}

fn resolve_dataset_path_value(dataset_path: Option<&str>) -> PathBuf {
    dataset_path.map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../A00-raw-research/benchmarks/tool-routing-subset")
    })
}
