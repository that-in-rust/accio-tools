use benchmark_eval_metrics_runner::{MetricReportOutputData, RoutingMetricsRequestData};
use tool_router_tauri_core::{
    compare_routing_modes_metrics as compare_metrics_inner,
    download_evaluation_pack_files as download_pack_files_inner,
    evaluate_routing_subset_metrics as evaluate_metrics_inner,
    export_diagnostic_logs_text as export_logs_inner,
    export_route_evidence_report as export_report_inner,
    route_tools_with_judge as route_query_inner, run_cpu_preview_only as run_preview_inner,
    validate_judge_api_key as validate_key_inner, AppError, EvaluationPackFileData,
    RouteEvidencePayloadData, RouteToolsRequestData, RouteToolsResponseData,
    RouterAppReadinessData,
};

#[tauri::command]
pub async fn validate_judge_api_key(
    api_key: Option<String>,
) -> Result<RouterAppReadinessData, AppError> {
    Ok(validate_key_inner(api_key))
}

#[tauri::command]
pub async fn run_cpu_preview_only(
    request: RouteToolsRequestData,
) -> Result<RouteToolsResponseData, AppError> {
    run_preview_inner(request)
}

#[tauri::command]
pub async fn route_tools_for_query(
    request: RouteToolsRequestData,
) -> Result<RouteToolsResponseData, AppError> {
    route_query_inner(request).await
}

#[tauri::command]
pub async fn evaluate_routing_subset_metrics(
    request: RoutingMetricsRequestData,
) -> Result<MetricReportOutputData, AppError> {
    evaluate_metrics_inner(request)
}

#[tauri::command]
pub async fn compare_routing_modes_metrics(
    request: RoutingMetricsRequestData,
) -> Result<Vec<MetricReportOutputData>, AppError> {
    compare_metrics_inner(request)
}

#[tauri::command]
pub async fn download_evaluation_pack_files(
    dataset_path: Option<String>,
) -> Result<Vec<EvaluationPackFileData>, AppError> {
    download_pack_files_inner(dataset_path)
}

#[tauri::command]
pub async fn export_route_evidence_report(
    payload: RouteEvidencePayloadData,
) -> Result<String, AppError> {
    Ok(export_report_inner(&payload))
}

#[tauri::command]
pub async fn export_diagnostic_logs_text() -> Result<String, AppError> {
    Ok(export_logs_inner())
}
