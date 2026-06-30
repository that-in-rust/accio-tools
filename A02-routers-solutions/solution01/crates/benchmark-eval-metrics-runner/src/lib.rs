use catalog_router_core_engine::{
    rank_tools_for_mode, validate_catalog_schema_input, validate_query_record_input,
    RouteQueryInputData, RouterModeNameData, RouterTypedErrorKind, ToolCatalogRecordData,
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
    pub average_selected_candidate_count: f64,
    pub token_reduction_estimate: f64,
    pub router_mode: RouterModeNameData,
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
    let dataset_path = request
        .dataset_path
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(default_dataset_path_value);
    let pack = load_bundled_evaluation_pack(&dataset_path)?;
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

    for query in &pack.queries {
        candidate_count_sum += predictions.get(&query.id).map(Vec::len).unwrap_or_default();
    }

    for query in &routed {
        let ranked = predictions.get(&query.id).cloned().unwrap_or_default();
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
        average_selected_candidate_count,
        token_reduction_estimate,
        router_mode: request.router_mode,
    })
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

fn rank_queries_for_mode(
    pack: &BundledEvaluationPackData,
    router_mode: RouterModeNameData,
    threshold: f64,
    max_k: usize,
) -> Result<HashMap<String, Vec<String>>, RouterTypedErrorKind> {
    let mut predictions = HashMap::new();
    for query in &pack.queries {
        let candidates =
            rank_candidates_for_mode(query, &pack.tools, router_mode, threshold, max_k)?;
        predictions.insert(
            query.id.clone(),
            candidates
                .into_iter()
                .take(max_k)
                .map(|candidate| candidate.tool_id)
                .collect(),
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

fn round_metric_value(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

fn create_markdown_report_text(report: &MetricReportOutputData) -> String {
    let recall_1 = report.recall_at_k.get("1").copied().unwrap_or_default();
    let recall_3 = report.recall_at_k.get("3").copied().unwrap_or_default();
    let recall_5 = report.recall_at_k.get("5").copied().unwrap_or_default();
    let recall_10 = report.recall_at_k.get("10").copied().unwrap_or_default();
    format!(
        "# Routing Metrics Report\n\n- router_mode: {:?}\n- queries: {}\n- route_required_queries: {}\n- abstention_queries: {}\n- Recall@1: {:.4}\n- Recall@3: {:.4}\n- Recall@5: {:.4}\n- Recall@10: {:.4}\n- MRR: {:.4}\n- nDCG@10: {:.4}\n- abstention_accuracy: {:.4}\n- average_selected_candidate_count: {:.4}\n- token_reduction_estimate: {:.4}\n",
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
        report.average_selected_candidate_count,
        report.token_reduction_estimate
    )
}

fn create_comparison_markdown_text(reports: &[MetricReportOutputData]) -> String {
    let mut lines = vec![
        "# Routing Mode Comparison Report".to_string(),
        String::new(),
        "| mode | Recall@5 | MRR | nDCG@10 | abstention | token reduction |".to_string(),
        "| --- | ---: | ---: | ---: | ---: | ---: |".to_string(),
    ];
    for report in reports {
        lines.push(format!(
            "| {:?} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} |",
            report.router_mode,
            report.recall_at_k.get("5").copied().unwrap_or_default(),
            report.mrr,
            report.ndcg_at_10,
            report.abstention_accuracy,
            report.token_reduction_estimate
        ));
    }
    lines.join("\n")
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
        assert_eq!(report.token_reduction_estimate, 0.9894);
    }

    #[test]
    fn markdown_includes_proof_fields() {
        let report = evaluate_routing_subset_metrics(RoutingMetricsRequestData {
            dataset_path: Some(bundled_dataset_path().display().to_string()),
            router_mode: RouterModeNameData::Lexical,
            max_k: 10,
            threshold: 2.0,
        })
        .expect("metrics should run");
        let markdown = create_markdown_report_text(&report);

        assert!(markdown.contains("Recall@1"));
        assert!(markdown.contains("Recall@10"));
        assert!(markdown.contains("token_reduction_estimate: 0.9894"));
    }

    #[test]
    fn comparison_includes_all_modes() {
        let reports = compare_routing_modes_metrics(RoutingMetricsRequestData {
            dataset_path: Some(bundled_dataset_path().display().to_string()),
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
}
