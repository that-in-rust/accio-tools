use pie_tauri_core::{
    analyze_prompt as analyze_prompt_core, analyze_updated_prompt as analyze_updated_prompt_core,
    apply_selected_fixes as apply_selected_fixes_core, export_findings_markdown_text,
    get_app_readiness as get_app_readiness_core, model_label_text,
    reverify_prompt as reverify_prompt_core, validate_openai_key as validate_openai_key_core,
    verify_and_export_update as verify_and_export_update_core, AnalysisResult,
    AnalyzePromptRequest, AppError, AppReadiness, AppReadinessRequest, DiagnosticLogger,
    FindingGroup, OpenAiHttpClient, ReverifyPromptRequest, ReverifyPromptResult,
    SelectedFixesRequest, SelectedFixesResult, SqliteVersionStore, VerificationExport, VersionKind,
};
use serde::Serialize;
use std::time::Instant;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct ApplySelectedFixesResponse {
    pub result: SelectedFixesResult,
    pub updated_version_name: String,
    pub action_log: Vec<String>,
}

#[tauri::command]
pub async fn get_app_readiness(
    api_key: Option<String>,
    client: State<'_, OpenAiHttpClient>,
    logger: State<'_, DiagnosticLogger>,
) -> Result<AppReadiness, AppError> {
    let started = Instant::now();
    record_command_started(logger.inner(), "get_app_readiness");
    let result = get_app_readiness_core(
        AppReadinessRequest {
            api_key,
            storage_ready: true,
        },
        client.inner(),
    )
    .await;
    record_command_outcome(logger.inner(), "get_app_readiness", started, &result);
    result
}

#[tauri::command]
pub async fn validate_api_key(
    api_key: String,
    client: State<'_, OpenAiHttpClient>,
    logger: State<'_, DiagnosticLogger>,
) -> Result<AppReadiness, AppError> {
    let started = Instant::now();
    record_command_started(logger.inner(), "validate_api_key");
    let result = validate_openai_key_core(api_key, client.inner()).await;
    record_command_outcome(logger.inner(), "validate_api_key", started, &result);
    result
}

#[tauri::command]
pub async fn analyze_prompt(
    request: AnalyzePromptRequest,
    client: State<'_, OpenAiHttpClient>,
    store: State<'_, SqliteVersionStore>,
    logger: State<'_, DiagnosticLogger>,
) -> Result<AnalysisResult, AppError> {
    let started = Instant::now();
    record_command_started(logger.inner(), "analyze_prompt");
    let result = analyze_prompt_core(request, client.inner(), store.inner()).await;
    record_command_outcome(logger.inner(), "analyze_prompt", started, &result);
    result
}

#[tauri::command]
pub fn export_findings(
    finding_groups: Vec<FindingGroup>,
    logger: State<'_, DiagnosticLogger>,
) -> Result<String, AppError> {
    let started = Instant::now();
    record_command_started(logger.inner(), "export_findings");
    let result =
        export_findings_markdown_text(&finding_groups, model_label_text()).map_err(AppError::from);
    record_command_outcome(logger.inner(), "export_findings", started, &result);
    result
}

#[tauri::command]
pub fn export_diagnostic_logs(logger: State<'_, DiagnosticLogger>) -> Result<String, AppError> {
    let started = Instant::now();
    record_command_started(logger.inner(), "export_diagnostic_logs");
    let initial_export = logger.export_log_text_report().map_err(AppError::from);

    match initial_export {
        Ok(_) => {
            record_command_succeeded(logger.inner(), "export_diagnostic_logs", started);
            logger.export_log_text_report().map_err(AppError::from)
        }
        Err(error) => {
            record_command_failed(logger.inner(), "export_diagnostic_logs", started, &error);
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn apply_selected_fixes(
    request: SelectedFixesRequest,
    client: State<'_, OpenAiHttpClient>,
    store: State<'_, SqliteVersionStore>,
    logger: State<'_, DiagnosticLogger>,
) -> Result<ApplySelectedFixesResponse, AppError> {
    let started = Instant::now();
    record_command_started(logger.inner(), "apply_selected_fixes");
    let result = async {
        let result = apply_selected_fixes_core(request, client.inner()).await?;
        let analysis_run_id = format!("patch_{}", create_epoch_millis_timestamp());
        let version = store
            .store_prompt_version(
                "updated_prompt.json",
                VersionKind::Updated,
                &result.candidate.updated_prompt_json,
                &analysis_run_id,
                model_label_text(),
            )
            .map_err(AppError::from)?;

        Ok(ApplySelectedFixesResponse {
            result,
            updated_version_name: version.version_name.clone(),
            action_log: vec![format!(
                "Stored updated prompt version {}",
                version.version_name
            )],
        })
    }
    .await;
    record_command_outcome(logger.inner(), "apply_selected_fixes", started, &result);
    result
}

#[tauri::command]
pub async fn reverify_prompt(
    request: ReverifyPromptRequest,
    client: State<'_, OpenAiHttpClient>,
    logger: State<'_, DiagnosticLogger>,
) -> Result<ReverifyPromptResult, AppError> {
    let started = Instant::now();
    record_command_started(logger.inner(), "reverify_prompt");
    let result = reverify_prompt_core(request, client.inner()).await;
    record_command_outcome(logger.inner(), "reverify_prompt", started, &result);
    result
}

#[tauri::command]
pub async fn verify_and_export_update(
    original_prompt_json: String,
    updated_prompt_json: String,
    selected_finding_ids: Vec<String>,
    api_key: String,
    client: State<'_, OpenAiHttpClient>,
    logger: State<'_, DiagnosticLogger>,
) -> Result<VerificationExport, AppError> {
    let started = Instant::now();
    record_command_started(logger.inner(), "verify_and_export_update");
    let result = verify_and_export_update_core(
        original_prompt_json,
        updated_prompt_json,
        selected_finding_ids,
        api_key,
        client.inner(),
    )
    .await;
    record_command_outcome(logger.inner(), "verify_and_export_update", started, &result);
    result
}

#[tauri::command]
pub async fn analyze_updated_prompt(
    request: AnalyzePromptRequest,
    client: State<'_, OpenAiHttpClient>,
    store: State<'_, SqliteVersionStore>,
    logger: State<'_, DiagnosticLogger>,
) -> Result<AnalysisResult, AppError> {
    let started = Instant::now();
    record_command_started(logger.inner(), "analyze_updated_prompt");
    let result = analyze_updated_prompt_core(request, client.inner(), store.inner()).await;
    record_command_outcome(logger.inner(), "analyze_updated_prompt", started, &result);
    result
}

fn create_epoch_millis_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn record_command_started(logger: &DiagnosticLogger, command_name: &str) {
    let _ = logger.record_info_event(
        "command_started",
        Some(command_name),
        &format!("Started {command_name}."),
    );
}

fn record_command_outcome<T>(
    logger: &DiagnosticLogger,
    command_name: &str,
    started: Instant,
    result: &Result<T, AppError>,
) {
    match result {
        Ok(_) => record_command_succeeded(logger, command_name, started),
        Err(error) => record_command_failed(logger, command_name, started, error),
    }
}

fn record_command_succeeded(logger: &DiagnosticLogger, command_name: &str, started: Instant) {
    let _ = logger.record_info_event(
        "command_succeeded",
        Some(command_name),
        &format!(
            "Completed {command_name} in {} ms.",
            started.elapsed().as_millis()
        ),
    );
}

fn record_command_failed(
    logger: &DiagnosticLogger,
    command_name: &str,
    started: Instant,
    error: &AppError,
) {
    let _ = logger.record_error_event(
        "command_failed",
        Some(command_name),
        &format!(
            "Failed {command_name} in {} ms: {}",
            started.elapsed().as_millis(),
            error.message
        ),
        Some(error.code.as_str()),
    );
}
