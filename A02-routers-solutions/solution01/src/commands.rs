use crate::{
    build_judge_request, build_llm_verify_request, build_patch_request,
    check_update_deterministically, export_verification_report, model_label_text,
    normalize_prompt_sections_model, parse_assignment_prompt_json, AnalysisResult,
    AnalyzePromptRequest, AppError, AppReadiness, AppReadinessRequest, OpenAiClient,
    OpenAiRuntimeConfig, PieError, ReverifyPromptRequest, ReverifyPromptResult,
    SelectedFixesRequest, SelectedFixesResult, SqliteVersionStore, VersionKind,
};

pub async fn get_app_readiness<C>(
    request: AppReadinessRequest,
    client: &C,
) -> Result<AppReadiness, AppError>
where
    C: OpenAiClient,
{
    let model_label = model_label_text();
    let Some(api_key) = request.api_key else {
        return Ok(AppReadiness {
            api_key_ready: false,
            storage_ready: request.storage_ready,
            analyze_enabled: false,
            model_label,
            readiness_message: "OpenAI API key is missing.".to_string(),
        });
    };

    if !request.storage_ready {
        return Ok(AppReadiness {
            api_key_ready: false,
            storage_ready: false,
            analyze_enabled: false,
            model_label,
            readiness_message: "Local storage is not ready.".to_string(),
        });
    }

    let config = OpenAiRuntimeConfig::from_api_key(api_key);
    match client.validate_key_with_model(&config).await {
        Ok(()) => Ok(AppReadiness {
            api_key_ready: true,
            storage_ready: true,
            analyze_enabled: true,
            model_label,
            readiness_message: "Ready to analyze prompt.".to_string(),
        }),
        Err(PieError::ModelUnavailable) => Ok(AppReadiness {
            api_key_ready: false,
            storage_ready: true,
            analyze_enabled: false,
            model_label,
            readiness_message: "gpt-5.4-mini is unavailable for the provided key.".to_string(),
        }),
        Err(error) => Err(AppError::from(error)),
    }
}

pub async fn validate_openai_key<C>(api_key: String, client: &C) -> Result<AppReadiness, AppError>
where
    C: OpenAiClient,
{
    get_app_readiness(
        AppReadinessRequest {
            api_key: Some(api_key),
            storage_ready: true,
        },
        client,
    )
    .await
}

pub async fn analyze_prompt<C>(
    request: AnalyzePromptRequest,
    client: &C,
    store: &SqliteVersionStore,
) -> Result<AnalysisResult, AppError>
where
    C: OpenAiClient,
{
    let config = OpenAiRuntimeConfig::from_api_key(request.api_key);
    client
        .validate_key_with_model(&config)
        .await
        .map_err(AppError::from)?;

    let prompt = parse_assignment_prompt_json(&request.prompt_json).map_err(AppError::from)?;
    let normalized = normalize_prompt_sections_model(&prompt).map_err(AppError::from)?;
    let analysis_run_id = format!("analysis_{}", chrono::Utc::now().format("%Y%m%d%H%M%S%3f"));
    let version = store
        .store_prompt_version(
            &request.filename,
            VersionKind::Original,
            &request.prompt_json,
            &analysis_run_id,
            model_label_text(),
        )
        .map_err(AppError::from)?;

    let judge_request =
        build_judge_request(&config, &prompt, &normalized).map_err(AppError::from)?;
    let finding_groups = client
        .judge_prompt_quality(judge_request)
        .await
        .map_err(AppError::from)?;

    Ok(AnalysisResult {
        prompt_version_name: version.version_name.clone(),
        model_label: model_label_text(),
        finding_groups,
        normalized_prompt: normalized,
        action_log: vec![
            format!("Stored prompt version {}", version.version_name),
            "Generated five LLM Judge finding groups.".to_string(),
        ],
    })
}

pub async fn analyze_updated_prompt<C>(
    request: AnalyzePromptRequest,
    client: &C,
    store: &SqliteVersionStore,
) -> Result<AnalysisResult, AppError>
where
    C: OpenAiClient,
{
    analyze_prompt(request, client, store).await
}

pub async fn apply_selected_fixes<C>(
    request: SelectedFixesRequest,
    client: &C,
) -> Result<SelectedFixesResult, AppError>
where
    C: OpenAiClient,
{
    let config = OpenAiRuntimeConfig::from_api_key(request.api_key);
    let patch_request = build_patch_request(
        &config,
        &request.original_prompt_json,
        &request.finding_groups,
        &request.selected_finding_ids,
    )
    .map_err(AppError::from)?;
    let candidate = client
        .patch_selected_prompt(patch_request)
        .await
        .map_err(AppError::from)?;

    Ok(SelectedFixesResult {
        candidate,
        model_label: model_label_text(),
    })
}

pub async fn reverify_prompt<C>(
    request: ReverifyPromptRequest,
    client: &C,
) -> Result<ReverifyPromptResult, AppError>
where
    C: OpenAiClient,
{
    let config = OpenAiRuntimeConfig::from_api_key(request.api_key.clone());
    client
        .validate_key_with_model(&config)
        .await
        .map_err(AppError::from)?;

    parse_assignment_prompt_json(&request.original_prompt_json).map_err(AppError::from)?;
    parse_assignment_prompt_json(&request.updated_prompt_json).map_err(AppError::from)?;

    client
        .reverify_prompt_history(request)
        .await
        .map_err(AppError::from)
}

pub async fn verify_and_export_update<C>(
    original_prompt_json: String,
    updated_prompt_json: String,
    selected_finding_ids: Vec<String>,
    api_key: String,
    client: &C,
) -> Result<crate::VerificationExport, AppError>
where
    C: OpenAiClient,
{
    let config = OpenAiRuntimeConfig::from_api_key(api_key);
    client
        .validate_key_with_model(&config)
        .await
        .map_err(AppError::from)?;

    let prompt = parse_assignment_prompt_json(&original_prompt_json).map_err(AppError::from)?;
    let deterministic_checks =
        check_update_deterministically(&prompt, &original_prompt_json, &updated_prompt_json)
            .map_err(AppError::from)?;
    let verify_request = build_llm_verify_request(
        config.api_key,
        original_prompt_json,
        updated_prompt_json.clone(),
        selected_finding_ids.clone(),
    );
    let semantic_checks = client
        .verify_semantic_update(verify_request)
        .await
        .map_err(AppError::from)?;

    Ok(export_verification_report(
        updated_prompt_json,
        model_label_text(),
        &selected_finding_ids,
        deterministic_checks,
        semantic_checks,
    ))
}
