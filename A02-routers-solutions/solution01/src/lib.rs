use benchmark_eval_metrics_runner::{
    compare_routing_modes_metrics as compare_modes_metrics_inner,
    evaluate_routing_subset_metrics as evaluate_subset_metrics_inner, load_bundled_evaluation_pack,
    MetricReportOutputData, RoutingMetricsRequestData,
};
use candidate_judge_openai_adapter::{
    judge_candidate_tools_top, JudgeCandidateRequestData, JudgeDecisionOutputData,
};
use catalog_router_core_engine::{
    rank_tools_for_mode, validate_catalog_schema_input, CandidateEvidenceCardData,
    RouterModeNameData, RouterTypedErrorKind, ToolCatalogRecordData,
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
    #[serde(default)]
    pub catalog_tools: Option<Vec<ToolCatalogRecordData>>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CatalogStatsSummaryData {
    pub tool_count: usize,
    pub query_count: usize,
    pub source_count: usize,
    pub schema_count: usize,
    pub route_required_count: usize,
    pub abstention_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkGoldMatchData {
    pub query_id: String,
    pub should_route: bool,
    pub required_tool_ids: Vec<String>,
    pub selected_tool_id: Option<String>,
    pub gold_match_status: String,
    pub failure_bucket: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteEvidencePayloadData {
    pub route_request: RouteToolsRequestData,
    pub route_response: RouteToolsResponseData,
    pub catalog_stats: CatalogStatsSummaryData,
    pub benchmark_gold_match: Option<BenchmarkGoldMatchData>,
    pub metrics_report: Option<MetricReportOutputData>,
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
    let tools = load_route_catalog_tools(&request)?;
    let candidates = rank_tools_for_mode(&request.query, &tools, request.router_mode, 2.0, 5)?;
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

pub fn compare_routing_modes_metrics(
    request: RoutingMetricsRequestData,
) -> Result<Vec<MetricReportOutputData>, AppError> {
    compare_modes_metrics_inner(request).map_err(AppError::from)
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

pub fn export_route_evidence_report(payload: &RouteEvidencePayloadData) -> String {
    build_route_evidence_report(payload)
}

pub fn build_route_evidence_report(payload: &RouteEvidencePayloadData) -> String {
    let response = &payload.route_response;
    let request = &payload.route_request;
    let mut lines = vec![
        "# Tool Router Evidence Report".to_string(),
        String::new(),
        "## Route Input".to_string(),
        format!("query: {}", redact_secret_values_text(&request.query)),
        format!("router_mode: {:?}", request.router_mode),
        format!(
            "recent_context: {}",
            request
                .recent_context
                .as_deref()
                .map(redact_secret_values_text)
                .unwrap_or_else(|| "none".to_string())
        ),
        format!(
            "dataset_path: {}",
            request
                .dataset_path
                .as_deref()
                .unwrap_or("bundled_evaluation_pack")
        ),
        "api_key: [SESSION_ONLY_REDACTED]".to_string(),
        String::new(),
        "## Catalog Stats".to_string(),
        format!("catalog_tool_count: {}", payload.catalog_stats.tool_count),
        format!("catalog_query_count: {}", payload.catalog_stats.query_count),
        format!(
            "catalog_source_count: {}",
            payload.catalog_stats.source_count
        ),
        format!(
            "catalog_schema_count: {}",
            payload.catalog_stats.schema_count
        ),
        format!(
            "route_required_count: {}",
            payload.catalog_stats.route_required_count
        ),
        format!(
            "abstention_count: {}",
            payload.catalog_stats.abstention_count
        ),
        String::new(),
        "## Route Result".to_string(),
        format!("route_label: {}", response.route_label),
        format!(
            "debug_preview: {}",
            response.route_label == "cpu_only_debug_preview"
        ),
        format!("candidate_count: {}", response.candidates.len()),
    ];

    if let Some(decision) = &response.judge_decision {
        lines.push(format!("decision: {:?}", decision.decision));
        lines.push(format!(
            "selected_tool_id: {}",
            decision.selected_tool_id.as_deref().unwrap_or("abstain")
        ));
        lines.push(format!("confidence: {:.3}", decision.confidence));
        lines.push(format!(
            "reason: {}",
            redact_secret_values_text(&decision.reason)
        ));
        lines.push(format!(
            "needs_more_metadata: {}",
            decision.needs_more_metadata
        ));
    } else {
        lines.push("decision: cpu_preview_only".to_string());
        lines.push("selected_tool_id: none".to_string());
        lines.push("confidence: 0.000".to_string());
        lines.push("reason: judge unavailable for CPU preview".to_string());
    }

    lines.push(String::new());
    lines.push("## CPU Top Five Candidates".to_string());
    for candidate in response.candidates.iter().take(5) {
        lines.push(format!(
            "candidate #{}: {} | score: {:.4} | risk: {} | fields: {} | reason: {}",
            candidate.rank,
            candidate.tool_id,
            candidate.score,
            candidate.risk,
            candidate.matched_fields.join(", "),
            redact_secret_values_text(&candidate.why_matched)
        ));
    }

    lines.push(String::new());
    lines.push("## Benchmark Gold".to_string());
    if let Some(gold) = &payload.benchmark_gold_match {
        lines.push(format!("query_id: {}", gold.query_id));
        lines.push(format!("should_route: {}", gold.should_route));
        lines.push(format!(
            "required_tool_ids: {}",
            gold.required_tool_ids.join(", ")
        ));
        lines.push(format!(
            "selected_tool_id: {}",
            gold.selected_tool_id.as_deref().unwrap_or("none")
        ));
        lines.push(format!("gold_match_status: {}", gold.gold_match_status));
        lines.push(format!("failure_bucket: {}", gold.failure_bucket));
    } else {
        lines.push("gold_match_status: unavailable".to_string());
        lines.push("failure_bucket: unlabeled_query".to_string());
    }

    lines.push(String::new());
    lines.push("## Metrics".to_string());
    if let Some(metrics) = &payload.metrics_report {
        for k in ["1", "3", "5", "10"] {
            lines.push(format!(
                "Recall@{}: {:.4}",
                k,
                metrics.recall_at_k.get(k).copied().unwrap_or_default()
            ));
        }
        lines.push(format!("MRR: {:.4}", metrics.mrr));
        lines.push(format!("nDCG@10: {:.4}", metrics.ndcg_at_10));
        lines.push(format!(
            "abstention_accuracy: {:.4}",
            metrics.abstention_accuracy
        ));
        lines.push(format!(
            "token_reduction_estimate: {:.4}",
            metrics.token_reduction_estimate
        ));
    } else {
        lines.push("metrics_status: not_attached".to_string());
    }

    lines.join("\n")
}

pub fn export_diagnostic_logs_text() -> String {
    "# Tool Router Diagnostic Log\n\nNo persisted diagnostic events yet.".to_string()
}

fn load_route_catalog_tools(
    request: &RouteToolsRequestData,
) -> Result<Vec<ToolCatalogRecordData>, AppError> {
    if let Some(tools) = &request.catalog_tools {
        validate_catalog_schema_input(tools)?;
        return Ok(tools.clone());
    }
    let dataset_path = resolve_dataset_path_value(request.dataset_path.as_deref());
    Ok(load_bundled_evaluation_pack(&dataset_path)?.tools)
}

pub fn redact_secret_values_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(|token| {
            if token.starts_with("sk-") {
                "[REDACTED_API_KEY]"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn resolve_dataset_path_value(dataset_path: Option<&str>) -> PathBuf {
    dataset_path.map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../A00-raw-research/benchmarks/tool-routing-subset")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use catalog_router_core_engine::RouterModeNameData;
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    #[test]
    fn preview_uses_router_mode() {
        let directory = tempdir().expect("temp dir should exist");
        write_pack_fixture_dir(directory.path());

        let response = run_cpu_preview_only(RouteToolsRequestData {
            dataset_path: Some(directory.path().display().to_string()),
            catalog_tools: None,
            query: "send message to channel".to_string(),
            recent_context: None,
            router_mode: RouterModeNameData::SchemaAware,
            api_key: None,
        })
        .expect("preview should run");

        assert_eq!(response.route_label, "cpu_only_debug_preview");
        assert!(response.candidates[0]
            .signal_contributions
            .contains_key("schema"));
    }

    #[test]
    fn compare_modes_returns_reports() {
        let directory = tempdir().expect("temp dir should exist");
        write_pack_fixture_dir(directory.path());

        let reports = compare_routing_modes_metrics(RoutingMetricsRequestData {
            dataset_path: Some(directory.path().display().to_string()),
            catalog_tools: None,
            query_records: None,
            router_mode: RouterModeNameData::Lexical,
            max_k: 10,
            threshold: 2.0,
        })
        .expect("mode comparison should run");

        assert_eq!(reports.len(), 3);
        assert_eq!(reports[0].router_mode, RouterModeNameData::Lexical);
        assert_eq!(reports[1].router_mode, RouterModeNameData::SchemaAware);
        assert_eq!(reports[2].router_mode, RouterModeNameData::Hybrid);
    }

    #[test]
    fn preview_uses_uploaded_catalog() {
        let response = run_cpu_preview_only(RouteToolsRequestData {
            dataset_path: Some("/missing/bundled/path".to_string()),
            catalog_tools: Some(vec![create_candidate_tool_data("custom.slack_post")]),
            query: "send message to channel".to_string(),
            recent_context: None,
            router_mode: RouterModeNameData::SchemaAware,
            api_key: None,
        })
        .expect("uploaded catalog preview should run");

        assert_eq!(response.route_label, "cpu_only_debug_preview");
        assert_eq!(response.candidates[0].tool_id, "custom.slack_post");
    }

    #[test]
    fn export_report_includes_context() {
        let report = export_route_evidence_report(&RouteEvidencePayloadData {
            route_request: RouteToolsRequestData {
                dataset_path: Some("fixtures/router-subset".to_string()),
                catalog_tools: None,
                query: "send a message to channel".to_string(),
                recent_context: Some("previous user turn".to_string()),
                router_mode: RouterModeNameData::Hybrid,
                api_key: Some("sk-should-not-leak".to_string()),
            },
            route_response: RouteToolsResponseData {
                route_label: "judged_route".to_string(),
                candidates: vec![create_candidate_card_data("tool.channel")],
                judge_decision: Some(JudgeDecisionOutputData {
                    selected_tool_id: Some("tool.channel".to_string()),
                    confidence: 0.91,
                    reason: "selected because channel and message matched".to_string(),
                    decision: candidate_judge_openai_adapter::JudgeDecisionKindData::SelectTool,
                    needs_more_metadata: false,
                }),
            },
            catalog_stats: CatalogStatsSummaryData {
                tool_count: 947,
                query_count: 50,
                source_count: 4,
                schema_count: 947,
                route_required_count: 46,
                abstention_count: 4,
            },
            benchmark_gold_match: Some(BenchmarkGoldMatchData {
                query_id: "TRQ-001".to_string(),
                should_route: true,
                required_tool_ids: vec!["tool.channel".to_string()],
                selected_tool_id: Some("tool.channel".to_string()),
                gold_match_status: "matched_required_tool".to_string(),
                failure_bucket: "none".to_string(),
            }),
            metrics_report: Some(create_metric_report_data()),
        });

        assert!(report.contains("query: send a message to channel"));
        assert!(report.contains("router_mode: Hybrid"));
        assert!(report.contains("catalog_tool_count: 947"));
        assert!(report.contains("candidate #1: tool.channel"));
        assert!(report.contains("selected_tool_id: tool.channel"));
        assert!(report.contains("gold_match_status: matched_required_tool"));
        assert!(report.contains("failure_bucket: none"));
        assert!(report.contains("token_reduction_estimate: 0.9894"));
        assert!(!report.contains("sk-should-not-leak"));
    }

    #[test]
    fn export_report_redacts_secret() {
        let report = export_route_evidence_report(&RouteEvidencePayloadData {
            route_request: RouteToolsRequestData {
                dataset_path: None,
                catalog_tools: None,
                query: "debug sk-secret123 should hide".to_string(),
                recent_context: None,
                router_mode: RouterModeNameData::Lexical,
                api_key: Some("sk-also-secret".to_string()),
            },
            route_response: RouteToolsResponseData {
                route_label: "cpu_only_debug_preview".to_string(),
                candidates: vec![],
                judge_decision: None,
            },
            catalog_stats: CatalogStatsSummaryData::default(),
            benchmark_gold_match: None,
            metrics_report: None,
        });

        assert!(report.contains("[REDACTED_API_KEY]"));
        assert!(!report.contains("sk-secret123"));
        assert!(!report.contains("sk-also-secret"));
        assert!(report.contains("debug_preview: true"));
    }

    fn write_pack_fixture_dir(path: &std::path::Path) {
        let tools = serde_json::json!([
            {
                "id": "tool.channel",
                "name": "send_notification",
                "description": "Post a message",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "channel": { "type": "string" },
                        "message": { "type": "string" }
                    }
                },
                "tags": ["message"]
            }
        ]);
        let queries = serde_json::json!([
            {
                "id": "fixture-query",
                "query": "send message to channel",
                "required_tool_ids": ["tool.channel"],
                "should_route": true,
                "graded_relevance": [{"tool_id": "tool.channel", "relevance": 3}],
                "source_expected_tools": ["tool.channel"],
                "failure_modes": []
            }
        ]);
        std::fs::write(path.join("tools.json"), tools.to_string()).expect("tools should write");
        std::fs::write(path.join("queries.json"), queries.to_string())
            .expect("queries should write");
        std::fs::write(path.join("manifest.json"), "{}").expect("manifest should write");
    }

    fn create_candidate_card_data(tool_id: &str) -> CandidateEvidenceCardData {
        CandidateEvidenceCardData {
            rank: 1,
            score: 3.5,
            tool_id: tool_id.to_string(),
            matched_terms: vec!["message".to_string()],
            matched_fields: vec!["input_schema".to_string()],
            capability_match: vec!["parameter".to_string()],
            risk: "low".to_string(),
            why_matched: "schema parameter matched".to_string(),
            signal_contributions: BTreeMap::from([("schema".to_string(), 1.5)]),
        }
    }

    fn create_candidate_tool_data(
        tool_id: &str,
    ) -> catalog_router_core_engine::ToolCatalogRecordData {
        catalog_router_core_engine::ToolCatalogRecordData {
            id: tool_id.to_string(),
            source_tool_id: None,
            server_id: Some("custom-upload".to_string()),
            server_name: Some("custom".to_string()),
            name: "post_message".to_string(),
            description: "Send a message to a channel".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "channel": { "type": "string" },
                    "message": { "type": "string" }
                }
            }),
            tags: vec!["message".to_string()],
            source: serde_json::json!({}),
            metadata: serde_json::json!({}),
            unknown_metadata: BTreeMap::new(),
        }
    }

    fn create_metric_report_data() -> MetricReportOutputData {
        MetricReportOutputData {
            queries: 50,
            route_required_queries: 46,
            abstention_queries: 4,
            recall_at_k: BTreeMap::from([
                ("1".to_string(), 0.413),
                ("3".to_string(), 0.5797),
                ("5".to_string(), 0.6493),
                ("10".to_string(), 0.679),
            ]),
            mrr: 0.5223,
            ndcg_at_10: 0.5553,
            abstention_accuracy: 0.0,
            average_selected_candidate_count: 10.0,
            token_reduction_estimate: 0.9894,
            router_mode: RouterModeNameData::Lexical,
        }
    }
}
