use catalog_router_core_engine::{
    rank_tools_for_mode, validate_catalog_schema_input, validate_query_record_input,
    CandidateEvidenceCardData, RouteQueryInputData, RouterModeNameData, RouterTypedErrorKind,
    ToolCatalogRecordData,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BundledEvaluationPackData {
    pub tools: Vec<ToolCatalogRecordData>,
    pub queries: Vec<RouteQueryInputData>,
    pub manifest: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoutingMetricsRequestData {
    pub dataset_path: Option<String>,
    #[serde(default)]
    pub catalog_tools: Option<Vec<ToolCatalogRecordData>>,
    #[serde(default)]
    pub query_records: Option<Vec<RouteQueryInputData>>,
    pub router_mode: RouterModeNameData,
    pub max_k: usize,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricReportOutputData {
    pub queries: usize,
    pub route_required_queries: usize,
    pub abstention_queries: usize,
    pub recall_at_k: BTreeMap<String, f64>,
    pub mrr: f64,
    pub ndcg_at_10: f64,
    pub abstention_accuracy: f64,
    pub judged_route_accuracy: f64,
    pub failure_bucket_counts: BTreeMap<String, usize>,
    pub average_selected_candidate_count: f64,
    pub token_reduction_estimate: f64,
    pub router_mode: RouterModeNameData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkRouteOutcomeKindData {
    MatchedRequiredTool,
    MissingRequiredTool,
    WrongJudgeSelection,
    CorrectAbstain,
    AbstentionMiss,
    UnjudgedCpuPreview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BenchmarkRouteOutcomeData {
    pub query_id: String,
    pub should_route: bool,
    pub required_tool_ids: Vec<String>,
    pub cpu_required_tool_survived: bool,
    pub judged_route_attempted: bool,
    pub judge_abstained: bool,
    pub judged_selected_tool_id: Option<String>,
    pub judged_passed: bool,
    pub outcome_kind: BenchmarkRouteOutcomeKindData,
    pub failure_bucket: String,
}

pub fn load_bundled_evaluation_pack(
    dataset_path: &Path,
) -> Result<BundledEvaluationPackData, RouterTypedErrorKind> {
    if !dataset_path.exists() {
        return Err(RouterTypedErrorKind::MissingDatasetPath {
            path: dataset_path.display().to_string(),
        });
    }
    let tools: Vec<ToolCatalogRecordData> = read_json_file_value(&dataset_path.join("tools.json"))?;
    let queries: Vec<RouteQueryInputData> =
        read_json_file_value(&dataset_path.join("queries.json"))?;
    let manifest: serde_json::Value = read_json_file_value(&dataset_path.join("manifest.json"))?;
    validate_catalog_schema_input(&tools)?;
    validate_query_record_input(&queries)?;
    Ok(BundledEvaluationPackData {
        tools,
        queries,
        manifest,
    })
}

pub fn evaluate_routing_subset_metrics(
    request: RoutingMetricsRequestData,
) -> Result<MetricReportOutputData, RouterTypedErrorKind> {
    let pack = load_metrics_pack_request(&request)?;
    let predictions =
        rank_queries_for_mode(&pack, request.router_mode, request.threshold, request.max_k)?;
    let routed: Vec<&RouteQueryInputData> = pack
        .queries
        .iter()
        .filter(|query| query.should_route)
        .collect();
    let abstentions: Vec<&RouteQueryInputData> = pack
        .queries
        .iter()
        .filter(|query| !query.should_route)
        .collect();
    let k_values = [1usize, 3, 5, 10];
    let mut recall_sums: BTreeMap<usize, f64> = BTreeMap::new();
    let mut mrr_sum = 0.0;
    let mut ndcg_sum = 0.0;
    let mut ndcg_count = 0usize;
    let mut candidate_count_sum = 0usize;
    let mut judged_pass_count = 0usize;
    let mut failure_bucket_counts = BTreeMap::new();

    for query in &pack.queries {
        candidate_count_sum += predictions.get(&query.id).map(Vec::len).unwrap_or_default();
        let candidates = predictions.get(&query.id).cloned().unwrap_or_default();
        let judged_selected_tool_id = select_mock_judge_tool_id(&candidates);
        let outcome = score_benchmark_route_outcome(
            query,
            &candidates,
            judged_selected_tool_id.as_deref(),
            judged_selected_tool_id.is_none(),
            true,
        );
        if outcome.judged_passed {
            judged_pass_count += 1;
        }
        *failure_bucket_counts
            .entry(outcome.failure_bucket)
            .or_insert(0) += 1;
    }

    for query in &routed {
        let candidates = predictions.get(&query.id).cloned().unwrap_or_default();
        let ranked = collect_ranked_tool_ids(&candidates);
        let required: std::collections::BTreeSet<&str> =
            query.required_tool_ids.iter().map(String::as_str).collect();
        for k in k_values {
            let selected: std::collections::BTreeSet<&str> =
                ranked.iter().take(k).map(String::as_str).collect();
            let matches = required.intersection(&selected).count();
            let recall = matches as f64 / required.len().max(1) as f64;
            *recall_sums.entry(k).or_insert(0.0) += recall;
        }
        let reciprocal_rank = ranked
            .iter()
            .position(|tool_id| required.contains(tool_id.as_str()))
            .map(|index| 1.0 / (index + 1) as f64)
            .unwrap_or(0.0);
        mrr_sum += reciprocal_rank;

        let relevance_by_tool: HashMap<&str, u32> = query
            .graded_relevance
            .iter()
            .map(|item| (item.tool_id.as_str(), item.relevance))
            .collect();
        if !relevance_by_tool.is_empty() {
            let actual = ranked
                .iter()
                .take(10)
                .enumerate()
                .map(|(index, tool_id)| {
                    discounted_gain_value(
                        *relevance_by_tool.get(tool_id.as_str()).unwrap_or(&0),
                        index + 1,
                    )
                })
                .sum::<f64>();
            let mut ideal_values: Vec<u32> = relevance_by_tool.values().copied().collect();
            ideal_values.sort_by(|left, right| right.cmp(left));
            let ideal = ideal_values
                .into_iter()
                .take(10)
                .enumerate()
                .map(|(index, relevance)| discounted_gain_value(relevance, index + 1))
                .sum::<f64>();
            ndcg_sum += if ideal > 0.0 { actual / ideal } else { 0.0 };
            ndcg_count += 1;
        }
    }

    let routed_count = routed.len().max(1) as f64;
    let abstention_hits = abstentions
        .iter()
        .filter(|query| {
            predictions
                .get(&query.id)
                .map(Vec::is_empty)
                .unwrap_or(true)
        })
        .count();
    let recall_at_k = k_values
        .into_iter()
        .map(|k| {
            let value = recall_sums.get(&k).copied().unwrap_or_default() / routed_count;
            (k.to_string(), round_metric_value(value))
        })
        .collect();

    let average_selected_candidate_count =
        round_metric_value(candidate_count_sum as f64 / pack.queries.len().max(1) as f64);
    let token_reduction_estimate = round_metric_value(
        1.0 - (average_selected_candidate_count / pack.tools.len().max(1) as f64),
    );

    Ok(MetricReportOutputData {
        queries: pack.queries.len(),
        route_required_queries: routed.len(),
        abstention_queries: abstentions.len(),
        recall_at_k,
        mrr: round_metric_value(mrr_sum / routed_count),
        ndcg_at_10: round_metric_value(ndcg_sum / ndcg_count.max(1) as f64),
        abstention_accuracy: round_metric_value(
            abstention_hits as f64 / abstentions.len().max(1) as f64,
        ),
        judged_route_accuracy: round_metric_value(
            judged_pass_count as f64 / pack.queries.len().max(1) as f64,
        ),
        failure_bucket_counts,
        average_selected_candidate_count,
        token_reduction_estimate,
        router_mode: request.router_mode,
    })
}

pub fn score_benchmark_route_outcome(
    query: &RouteQueryInputData,
    candidates: &[CandidateEvidenceCardData],
    judged_selected_tool_id: Option<&str>,
    judge_abstained: bool,
    judged_route_attempted: bool,
) -> BenchmarkRouteOutcomeData {
    let required: std::collections::BTreeSet<&str> =
        query.required_tool_ids.iter().map(String::as_str).collect();
    let cpu_required_tool_survived = candidates
        .iter()
        .take(5)
        .any(|candidate| required.contains(candidate.tool_id.as_str()));
    let judged_selected_required = judged_selected_tool_id
        .map(|tool_id| required.contains(tool_id))
        .unwrap_or(false);
    let outcome_kind = if !judged_route_attempted {
        BenchmarkRouteOutcomeKindData::UnjudgedCpuPreview
    } else if query.should_route && !cpu_required_tool_survived {
        BenchmarkRouteOutcomeKindData::MissingRequiredTool
    } else if query.should_route && judged_selected_required {
        BenchmarkRouteOutcomeKindData::MatchedRequiredTool
    } else if query.should_route {
        BenchmarkRouteOutcomeKindData::WrongJudgeSelection
    } else if judge_abstained {
        BenchmarkRouteOutcomeKindData::CorrectAbstain
    } else {
        BenchmarkRouteOutcomeKindData::AbstentionMiss
    };
    let judged_passed = matches!(
        outcome_kind,
        BenchmarkRouteOutcomeKindData::MatchedRequiredTool
            | BenchmarkRouteOutcomeKindData::CorrectAbstain
    );
    BenchmarkRouteOutcomeData {
        query_id: query.id.clone(),
        should_route: query.should_route,
        required_tool_ids: query.required_tool_ids.clone(),
        cpu_required_tool_survived,
        judged_route_attempted,
        judge_abstained,
        judged_selected_tool_id: judged_selected_tool_id.map(str::to_string),
        judged_passed,
        failure_bucket: create_failure_bucket_value(&outcome_kind, query),
        outcome_kind,
    }
}

pub fn compare_routing_modes_metrics(
    request: RoutingMetricsRequestData,
) -> Result<Vec<MetricReportOutputData>, RouterTypedErrorKind> {
    [
        RouterModeNameData::Lexical,
        RouterModeNameData::SchemaAware,
        RouterModeNameData::Hybrid,
    ]
    .into_iter()
    .map(|router_mode| {
        evaluate_routing_subset_metrics(RoutingMetricsRequestData {
            dataset_path: request.dataset_path.clone(),
            catalog_tools: request.catalog_tools.clone(),
            query_records: request.query_records.clone(),
            router_mode,
            max_k: request.max_k,
            threshold: request.threshold,
        })
    })
    .collect()
}

pub fn write_evaluation_reports_files(
    report: &MetricReportOutputData,
    report_dir: &Path,
) -> Result<(), RouterTypedErrorKind> {
    std::fs::create_dir_all(report_dir).map_err(|error| RouterTypedErrorKind::ReadFileFailed {
        path: report_dir.display().to_string(),
        message: error.to_string(),
    })?;
    let json_path = report_dir.join("routing-metrics-report.json");
    let markdown_path = report_dir.join("routing-metrics-report.md");
    let json_content = serde_json::to_string_pretty(report).map_err(RouterTypedErrorKind::from)?;
    std::fs::write(&json_path, json_content).map_err(|error| {
        RouterTypedErrorKind::ReadFileFailed {
            path: json_path.display().to_string(),
            message: error.to_string(),
        }
    })?;
    std::fs::write(&markdown_path, create_markdown_report_text(report)).map_err(|error| {
        RouterTypedErrorKind::ReadFileFailed {
            path: markdown_path.display().to_string(),
            message: error.to_string(),
        }
    })?;
    Ok(())
}

pub fn write_comparison_reports_files(
    reports: &[MetricReportOutputData],
    report_dir: &Path,
) -> Result<(), RouterTypedErrorKind> {
    std::fs::create_dir_all(report_dir).map_err(|error| RouterTypedErrorKind::ReadFileFailed {
        path: report_dir.display().to_string(),
        message: error.to_string(),
    })?;
    let json_path = report_dir.join("routing-mode-comparison-report.json");
    let markdown_path = report_dir.join("routing-mode-comparison-report.md");
    let json_content = serde_json::to_string_pretty(reports).map_err(RouterTypedErrorKind::from)?;
    std::fs::write(&json_path, json_content).map_err(|error| {
        RouterTypedErrorKind::ReadFileFailed {
            path: json_path.display().to_string(),
            message: error.to_string(),
        }
    })?;
    std::fs::write(&markdown_path, create_comparison_markdown_text(reports)).map_err(|error| {
        RouterTypedErrorKind::ReadFileFailed {
            path: markdown_path.display().to_string(),
            message: error.to_string(),
        }
    })?;
    Ok(())
}

fn read_json_file_value<T>(path: &Path) -> Result<T, RouterTypedErrorKind>
where
    T: serde::de::DeserializeOwned,
{
    let content =
        std::fs::read_to_string(path).map_err(|error| RouterTypedErrorKind::ReadFileFailed {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;
    serde_json::from_str(&content).map_err(|error| RouterTypedErrorKind::ParseJsonFailed {
        path: path.display().to_string(),
        message: error.to_string(),
    })
}

fn default_dataset_path_value() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../A00-raw-research/benchmarks/tool-routing-subset")
}

fn load_metrics_pack_request(
    request: &RoutingMetricsRequestData,
) -> Result<BundledEvaluationPackData, RouterTypedErrorKind> {
    match (&request.catalog_tools, &request.query_records) {
        (Some(tools), Some(queries)) => {
            validate_catalog_schema_input(tools)?;
            validate_query_record_input(queries)?;
            Ok(BundledEvaluationPackData {
                tools: tools.clone(),
                queries: queries.clone(),
                manifest: serde_json::json!({
                    "source": "inline_upload",
                    "tool_count": tools.len(),
                    "query_count": queries.len()
                }),
            })
        }
        (Some(_), None) => Err(RouterTypedErrorKind::QueryValidationFailed {
            message: "inline metrics request requires query_records with catalog_tools".to_string(),
        }),
        (None, Some(_)) => Err(RouterTypedErrorKind::CatalogValidationFailed {
            message: "inline metrics request requires catalog_tools with query_records".to_string(),
        }),
        (None, None) => {
            let dataset_path = request
                .dataset_path
                .as_deref()
                .map(PathBuf::from)
                .unwrap_or_else(default_dataset_path_value);
            load_bundled_evaluation_pack(&dataset_path)
        }
    }
}

fn rank_queries_for_mode(
    pack: &BundledEvaluationPackData,
    router_mode: RouterModeNameData,
    threshold: f64,
    max_k: usize,
) -> Result<HashMap<String, Vec<CandidateEvidenceCardData>>, RouterTypedErrorKind> {
    let mut predictions = HashMap::new();
    for query in &pack.queries {
        let candidates =
            rank_candidates_for_mode(query, &pack.tools, router_mode, threshold, max_k)?;
        predictions.insert(
            query.id.clone(),
            candidates.into_iter().take(max_k).collect(),
        );
    }
    Ok(predictions)
}

fn rank_candidates_for_mode(
    query: &RouteQueryInputData,
    tools: &[ToolCatalogRecordData],
    router_mode: RouterModeNameData,
    threshold: f64,
    max_k: usize,
) -> Result<Vec<catalog_router_core_engine::CandidateEvidenceCardData>, RouterTypedErrorKind> {
    rank_tools_for_mode(&query.query, tools, router_mode, threshold, max_k)
}

fn discounted_gain_value(relevance: u32, rank: usize) -> f64 {
    (2.0_f64.powi(relevance as i32) - 1.0) / ((rank + 1) as f64).log2()
}

fn collect_ranked_tool_ids(candidates: &[CandidateEvidenceCardData]) -> Vec<String> {
    candidates
        .iter()
        .map(|candidate| candidate.tool_id.clone())
        .collect()
}

fn select_mock_judge_tool_id(candidates: &[CandidateEvidenceCardData]) -> Option<String> {
    candidates
        .first()
        .map(|candidate| candidate.tool_id.clone())
}

fn create_failure_bucket_value(
    outcome_kind: &BenchmarkRouteOutcomeKindData,
    query: &RouteQueryInputData,
) -> String {
    match outcome_kind {
        BenchmarkRouteOutcomeKindData::MatchedRequiredTool
        | BenchmarkRouteOutcomeKindData::CorrectAbstain => "none".to_string(),
        BenchmarkRouteOutcomeKindData::MissingRequiredTool => "missing_required_tool".to_string(),
        BenchmarkRouteOutcomeKindData::WrongJudgeSelection => "wrong_llm_top1".to_string(),
        BenchmarkRouteOutcomeKindData::AbstentionMiss => "abstention_miss".to_string(),
        BenchmarkRouteOutcomeKindData::UnjudgedCpuPreview => query
            .failure_modes
            .first()
            .cloned()
            .unwrap_or_else(|| "unjudged_cpu_preview".to_string()),
    }
}

fn round_metric_value(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

fn create_markdown_report_text(report: &MetricReportOutputData) -> String {
    let recall_1 = report.recall_at_k.get("1").copied().unwrap_or_default();
    let recall_3 = report.recall_at_k.get("3").copied().unwrap_or_default();
    let recall_5 = report.recall_at_k.get("5").copied().unwrap_or_default();
    let recall_10 = report.recall_at_k.get("10").copied().unwrap_or_default();
    let failure_bucket_lines = create_failure_bucket_markdown_lines(&report.failure_bucket_counts);
    format!(
        "# Routing Metrics Report\n\n- router_mode: {:?}\n- queries: {}\n- route_required_queries: {}\n- abstention_queries: {}\n- Recall@1: {:.4}\n- Recall@3: {:.4}\n- Recall@5: {:.4}\n- Recall@10: {:.4}\n- MRR: {:.4}\n- nDCG@10: {:.4}\n- abstention_accuracy: {:.4}\n- judged_route_accuracy: {:.4}\n- average_selected_candidate_count: {:.4}\n- token_reduction_estimate: {:.4}\n\n## failure_bucket_counts\n{}",
        report.router_mode,
        report.queries,
        report.route_required_queries,
        report.abstention_queries,
        recall_1,
        recall_3,
        recall_5,
        recall_10,
        report.mrr,
        report.ndcg_at_10,
        report.abstention_accuracy,
        report.judged_route_accuracy,
        report.average_selected_candidate_count,
        report.token_reduction_estimate,
        failure_bucket_lines
    )
}

fn create_failure_bucket_markdown_lines(failure_bucket_counts: &BTreeMap<String, usize>) -> String {
    if failure_bucket_counts.is_empty() {
        return "- none: 0\n".to_string();
    }
    failure_bucket_counts
        .iter()
        .map(|(bucket, count)| format!("- {bucket}: {count}"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn create_comparison_markdown_text(reports: &[MetricReportOutputData]) -> String {
    let mut lines = vec![
        "# Routing Mode Comparison Report".to_string(),
        String::new(),
        "| mode | Recall@5 | MRR | nDCG@10 | abstention | judged route | top failure bucket | token reduction |"
            .to_string(),
        "| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: |".to_string(),
    ];
    for report in reports {
        lines.push(format!(
            "| {:?} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {} | {:.4} |",
            report.router_mode,
            report.recall_at_k.get("5").copied().unwrap_or_default(),
            report.mrr,
            report.ndcg_at_10,
            report.abstention_accuracy,
            report.judged_route_accuracy,
            create_top_failure_bucket_text(&report.failure_bucket_counts),
            report.token_reduction_estimate
        ));
    }
    lines.join("\n")
}

fn create_top_failure_bucket_text(failure_bucket_counts: &BTreeMap<String, usize>) -> String {
    failure_bucket_counts
        .iter()
        .filter(|(bucket, _)| bucket.as_str() != "none")
        .max_by(|left, right| left.1.cmp(right.1).then_with(|| right.0.cmp(left.0)))
        .map(|(bucket, count)| format!("{bucket}: {count}"))
        .unwrap_or_else(|| "none: 0".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundled_dataset_path() -> PathBuf {
        default_dataset_path_value()
    }

    #[test]
    fn test_bundled_pack_counts() {
        let pack = load_bundled_evaluation_pack(&bundled_dataset_path())
            .expect("bundled evaluation pack should load");
        assert_eq!(pack.tools.len(), 947);
        assert_eq!(pack.queries.len(), 50);
        assert_eq!(
            pack.queries
                .iter()
                .filter(|query| query.should_route)
                .count(),
            46
        );
        assert_eq!(
            pack.queries
                .iter()
                .filter(|query| !query.should_route)
                .count(),
            4
        );
    }

    #[test]
    fn test_query_record_fields() {
        let pack = load_bundled_evaluation_pack(&bundled_dataset_path())
            .expect("bundled evaluation pack should load");
        let query = &pack.queries[0];
        assert_eq!(query.id, "TRQ-001");
        assert_eq!(
            query.required_tool_ids,
            vec!["votr.Google Sheets::listSpreadsheets"]
        );
        assert!(query.should_route);
        assert!(!query.graded_relevance.is_empty());
        assert!(!query.source_expected_tools.is_empty());
        assert!(!query.failure_modes.is_empty());
    }

    #[test]
    fn test_metrics_report_shape() {
        let report = evaluate_routing_subset_metrics(RoutingMetricsRequestData {
            dataset_path: Some(bundled_dataset_path().display().to_string()),
            catalog_tools: None,
            query_records: None,
            router_mode: RouterModeNameData::Lexical,
            max_k: 10,
            threshold: 2.0,
        })
        .expect("metrics should run");
        assert_eq!(report.queries, 50);
        assert_eq!(report.route_required_queries, 46);
        assert!(report.recall_at_k.contains_key("5"));
        assert_eq!(report.recall_at_k.get("5").copied(), Some(0.6493));
        assert_eq!(report.mrr, 0.5223);
        assert_eq!(report.abstention_accuracy, 0.0);
        assert_eq!(report.failure_bucket_counts.values().sum::<usize>(), 50);
        assert!(report.failure_bucket_counts.contains_key("wrong_llm_top1"));
        assert!(report.failure_bucket_counts.contains_key("abstention_miss"));
        assert_eq!(report.token_reduction_estimate, 0.9894);
    }

    #[test]
    fn markdown_includes_proof_fields() {
        let report = evaluate_routing_subset_metrics(RoutingMetricsRequestData {
            dataset_path: Some(bundled_dataset_path().display().to_string()),
            catalog_tools: None,
            query_records: None,
            router_mode: RouterModeNameData::Lexical,
            max_k: 10,
            threshold: 2.0,
        })
        .expect("metrics should run");
        let markdown = create_markdown_report_text(&report);

        assert!(markdown.contains("Recall@1"));
        assert!(markdown.contains("Recall@10"));
        assert!(markdown.contains("failure_bucket_counts"));
        assert!(markdown.contains("wrong_llm_top1"));
        assert!(markdown.contains("token_reduction_estimate: 0.9894"));
    }

    #[test]
    fn comparison_includes_all_modes() {
        let reports = compare_routing_modes_metrics(RoutingMetricsRequestData {
            dataset_path: Some(bundled_dataset_path().display().to_string()),
            catalog_tools: None,
            query_records: None,
            router_mode: RouterModeNameData::Lexical,
            max_k: 10,
            threshold: 2.0,
        })
        .expect("comparison should run");

        assert_eq!(reports.len(), 3);
        assert_eq!(reports[0].router_mode, RouterModeNameData::Lexical);
        assert_eq!(reports[1].router_mode, RouterModeNameData::SchemaAware);
        assert_eq!(reports[2].router_mode, RouterModeNameData::Hybrid);
    }

    #[test]
    fn metrics_use_uploaded_pack() {
        let report = evaluate_routing_subset_metrics(RoutingMetricsRequestData {
            dataset_path: Some("/missing/on/purpose".to_string()),
            catalog_tools: Some(vec![create_custom_catalog_tool()]),
            query_records: Some(vec![create_custom_query_record()]),
            router_mode: RouterModeNameData::Lexical,
            max_k: 10,
            threshold: 2.0,
        })
        .expect("inline upload metrics should run without bundled path");

        assert_eq!(report.queries, 1);
        assert_eq!(report.route_required_queries, 1);
        assert_eq!(report.recall_at_k.get("5").copied(), Some(1.0));
        assert_eq!(report.mrr, 1.0);
    }

    #[test]
    fn judged_scoring_requires_selected_required_tool() {
        let query = create_custom_query_record();
        let candidates = vec![
            create_candidate_card_data("custom.slack_read", 1),
            create_candidate_card_data("custom.slack_post", 2),
        ];

        let outcome = score_benchmark_route_outcome(
            &query,
            &candidates,
            Some("custom.slack_read"),
            false,
            true,
        );

        assert!(outcome.cpu_required_tool_survived);
        assert!(!outcome.judged_passed);
        assert_eq!(
            outcome.outcome_kind,
            BenchmarkRouteOutcomeKindData::WrongJudgeSelection
        );
        assert_eq!(outcome.failure_bucket, "wrong_llm_top1");
    }

    #[test]
    fn judged_scoring_scores_abstention_gold() {
        let mut query = create_custom_query_record();
        query.should_route = false;
        query.required_tool_ids = Vec::new();

        let correct = score_benchmark_route_outcome(&query, &[], None, true, true);
        let wrong = score_benchmark_route_outcome(
            &query,
            &[create_candidate_card_data("custom.slack_post", 1)],
            Some("custom.slack_post"),
            false,
            true,
        );

        assert!(correct.judged_passed);
        assert_eq!(
            correct.outcome_kind,
            BenchmarkRouteOutcomeKindData::CorrectAbstain
        );
        assert!(!wrong.judged_passed);
        assert_eq!(
            wrong.outcome_kind,
            BenchmarkRouteOutcomeKindData::AbstentionMiss
        );
    }

    #[test]
    fn metrics_report_counts_mock_judged_outcomes() {
        let mut abstain_query = create_custom_query_record();
        abstain_query.id = "custom-query-02".to_string();
        abstain_query.required_tool_ids = Vec::new();
        abstain_query.should_route = false;
        let report = evaluate_routing_subset_metrics(RoutingMetricsRequestData {
            dataset_path: Some("/missing/on/purpose".to_string()),
            catalog_tools: Some(vec![create_custom_catalog_tool()]),
            query_records: Some(vec![create_custom_query_record(), abstain_query]),
            router_mode: RouterModeNameData::Lexical,
            max_k: 10,
            threshold: 2.0,
        })
        .expect("inline upload metrics should run");

        assert_eq!(report.queries, 2);
        assert_eq!(report.recall_at_k.get("5").copied(), Some(1.0));
        assert_eq!(report.judged_route_accuracy, 0.5);
        assert_eq!(report.abstention_accuracy, 0.0);
        assert_eq!(report.failure_bucket_counts.get("none"), Some(&1));
        assert_eq!(
            report.failure_bucket_counts.get("abstention_miss"),
            Some(&1)
        );
    }

    #[test]
    fn inline_pack_requires_pair() {
        let error = evaluate_routing_subset_metrics(RoutingMetricsRequestData {
            dataset_path: None,
            catalog_tools: Some(vec![create_custom_catalog_tool()]),
            query_records: None,
            router_mode: RouterModeNameData::Lexical,
            max_k: 10,
            threshold: 2.0,
        })
        .expect_err("partial inline request should fail");

        assert!(error
            .to_string()
            .contains("requires query_records with catalog_tools"));
    }

    fn create_custom_catalog_tool() -> ToolCatalogRecordData {
        ToolCatalogRecordData {
            id: "custom.slack_post".to_string(),
            source_tool_id: None,
            server_id: Some("custom".to_string()),
            server_name: Some("Custom".to_string()),
            name: "post_message".to_string(),
            description: "Send Slack messages to incident channels.".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            tags: vec!["slack".to_string(), "message".to_string()],
            source: serde_json::Value::Null,
            metadata: serde_json::Value::Null,
            unknown_metadata: BTreeMap::new(),
        }
    }

    fn create_custom_query_record() -> RouteQueryInputData {
        RouteQueryInputData {
            id: "custom-query-01".to_string(),
            query: "Send a Slack message to the incident channel".to_string(),
            required_tool_ids: vec!["custom.slack_post".to_string()],
            should_route: true,
            graded_relevance: vec![catalog_router_core_engine::GradedRelevanceItemData {
                tool_id: "custom.slack_post".to_string(),
                relevance: 3,
            }],
            source_expected_tools: vec!["custom.slack_post".to_string()],
            failure_modes: vec!["confuse chat read with chat write".to_string()],
            unknown_metadata: BTreeMap::new(),
        }
    }

    fn create_candidate_card_data(tool_id: &str, rank: usize) -> CandidateEvidenceCardData {
        CandidateEvidenceCardData {
            rank,
            score: 10.0 - rank as f64,
            tool_id: tool_id.to_string(),
            matched_terms: vec!["slack".to_string()],
            matched_fields: vec!["description".to_string()],
            capability_match: vec!["write".to_string()],
            risk: "low".to_string(),
            why_matched: "Matched Slack language.".to_string(),
            signal_contributions: BTreeMap::new(),
        }
    }
}
