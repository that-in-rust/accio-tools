use benchmark_eval_metrics_runner::{
    compare_routing_modes_metrics, evaluate_routing_subset_metrics, write_comparison_reports_files,
    write_evaluation_reports_files, RoutingMetricsRequestData,
};
use catalog_router_core_engine::{RouterModeNameData, RouterTypedErrorKind};
use std::path::PathBuf;
use std::str::FromStr;
use tool_router_tauri_core::{
    route_tools_with_judge, run_cpu_preview_only, AppError, RouteToolsRequestData,
};

fn main() -> std::process::ExitCode {
    match run_cli_command_surface(std::env::args().skip(1).collect()) {
        Ok(output) => {
            println!("{output}");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run_cli_command_surface(args: Vec<String>) -> Result<String, RouterTypedErrorKind> {
    let Some(command_name) = args.first().map(String::as_str) else {
        return Err(RouterTypedErrorKind::UnsupportedRouterMode {
            mode: usage_message_text_value(),
        });
    };
    if !matches!(
        command_name,
        "run-cpu-preview-only"
            | "route-tools-for-query"
            | "evaluate-routing-subset-metrics"
            | "compare-routing-modes-metrics"
    ) {
        return Err(RouterTypedErrorKind::UnsupportedRouterMode {
            mode: usage_message_text_value(),
        });
    }

    let options = parse_cli_options_data(&args)?;
    match command_name {
        "run-cpu-preview-only" => {
            let response = run_cpu_preview_only(create_route_request_data(&options, false)?)
                .map_err(map_app_error_kind)?;
            serde_json::to_string_pretty(&response).map_err(RouterTypedErrorKind::from)
        }
        "route-tools-for-query" => {
            let response = run_route_with_runtime(create_route_request_data(&options, true)?)?;
            serde_json::to_string_pretty(&response).map_err(RouterTypedErrorKind::from)
        }
        "compare-routing-modes-metrics" => {
            let reports = compare_routing_modes_metrics(create_metrics_request_data(&options))?;
            if let Some(report_dir) = options.report_dir {
                write_comparison_reports_files(&reports, &report_dir)?;
            }
            serde_json::to_string_pretty(&reports).map_err(RouterTypedErrorKind::from)
        }
        _ => {
            let report = evaluate_routing_subset_metrics(create_metrics_request_data(&options))?;
            if let Some(report_dir) = options.report_dir {
                write_evaluation_reports_files(&report, &report_dir)?;
            }
            serde_json::to_string_pretty(&report).map_err(RouterTypedErrorKind::from)
        }
    }
}

#[derive(Debug)]
struct CliOptionsData {
    dataset_path: Option<String>,
    report_dir: Option<PathBuf>,
    router_mode: RouterModeNameData,
    query: Option<String>,
    recent_context: Option<String>,
    api_key: Option<String>,
}

impl Default for CliOptionsData {
    fn default() -> Self {
        Self {
            dataset_path: None,
            report_dir: None,
            router_mode: RouterModeNameData::Lexical,
            query: None,
            recent_context: None,
            api_key: None,
        }
    }
}

fn parse_cli_options_data(args: &[String]) -> Result<CliOptionsData, RouterTypedErrorKind> {
    let mut options = CliOptionsData::default();
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--dataset" => {
                index += 1;
                options.dataset_path = Some(require_cli_arg_value(args, index, "--dataset")?);
            }
            "--mode" => {
                index += 1;
                let mode = require_cli_arg_value(args, index, "--mode")?;
                options.router_mode = RouterModeNameData::from_str(&mode)?;
            }
            "--report-dir" => {
                index += 1;
                options.report_dir = Some(PathBuf::from(require_cli_arg_value(
                    args,
                    index,
                    "--report-dir",
                )?));
            }
            "--query" => {
                index += 1;
                options.query = Some(require_cli_arg_value(args, index, "--query")?);
            }
            "--context" => {
                index += 1;
                options.recent_context = Some(require_cli_arg_value(args, index, "--context")?);
            }
            "--api-key" => {
                index += 1;
                options.api_key = Some(require_cli_arg_value(args, index, "--api-key")?);
            }
            _ => {}
        }
        index += 1;
    }
    Ok(options)
}

fn require_cli_arg_value(
    args: &[String],
    index: usize,
    flag: &str,
) -> Result<String, RouterTypedErrorKind> {
    args.get(index)
        .filter(|value| !value.starts_with("--"))
        .cloned()
        .ok_or_else(|| RouterTypedErrorKind::UnsupportedRouterMode {
            mode: format!("missing value for {flag}"),
        })
}

fn create_metrics_request_data(options: &CliOptionsData) -> RoutingMetricsRequestData {
    RoutingMetricsRequestData {
        dataset_path: options.dataset_path.clone(),
        catalog_tools: None,
        query_records: None,
        router_mode: options.router_mode,
        max_k: 10,
        threshold: 2.0,
    }
}

fn create_route_request_data(
    options: &CliOptionsData,
    include_key: bool,
) -> Result<RouteToolsRequestData, RouterTypedErrorKind> {
    let query =
        options
            .query
            .clone()
            .ok_or_else(|| RouterTypedErrorKind::QueryValidationFailed {
                message: "route commands require --query".to_string(),
            })?;
    let api_key = if include_key {
        Some(
            options
                .api_key
                .as_ref()
                .filter(|value| !value.trim().is_empty())
                .cloned()
                .ok_or_else(|| RouterTypedErrorKind::JudgeConfigurationFailed {
                    message: "route-tools-for-query requires --api-key; use run-cpu-preview-only without a key".to_string(),
                })?,
        )
    } else {
        None
    };
    Ok(RouteToolsRequestData {
        dataset_path: options.dataset_path.clone(),
        catalog_tools: None,
        query,
        recent_context: options.recent_context.clone(),
        router_mode: options.router_mode,
        api_key,
    })
}

fn map_app_error_kind(error: AppError) -> RouterTypedErrorKind {
    match error {
        AppError::Router(error) => error,
    }
}

fn run_route_with_runtime(
    request: RouteToolsRequestData,
) -> Result<tool_router_tauri_core::RouteToolsResponseData, RouterTypedErrorKind> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| RouterTypedErrorKind::JudgeConfigurationFailed {
            message: format!("failed to start route runtime: {error}"),
        })?;
    runtime
        .block_on(route_tools_with_judge(request))
        .map_err(map_app_error_kind)
}

fn usage_message_text_value() -> String {
    "usage: run-cpu-preview-only|route-tools-for-query|evaluate-routing-subset-metrics|compare-routing-modes-metrics".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn cli_runs_cpu_preview() {
        let directory = tempdir().expect("temp dir should exist");
        write_route_fixture_pack(directory.path());
        let output = run_cli_command_surface(vec![
            "run-cpu-preview-only".to_string(),
            "--dataset".to_string(),
            directory.path().display().to_string(),
            "--query".to_string(),
            "send message to channel".to_string(),
            "--mode".to_string(),
            "schema-aware".to_string(),
        ])
        .expect("cpu preview should run");

        assert!(output.contains("\"route_label\": \"cpu_only_debug_preview\""));
        assert!(output.contains("\"judge_decision\": null"));
        assert!(output.contains("\"tool_id\": \"tool.channel\""));
    }

    #[test]
    fn cli_runs_judged_route() {
        let directory = tempdir().expect("temp dir should exist");
        write_route_fixture_pack(directory.path());
        let output = run_cli_command_surface(vec![
            "route-tools-for-query".to_string(),
            "--dataset".to_string(),
            directory.path().display().to_string(),
            "--query".to_string(),
            "send message to channel".to_string(),
            "--api-key".to_string(),
            "sk-router-test".to_string(),
        ])
        .expect("judged route should run");

        assert!(output.contains("\"route_label\": \"judged_route\""));
        assert!(output.contains("\"decision\": \"select_tool\""));
        assert!(output.contains("\"selected_tool_id\": \"tool.channel\""));
    }

    #[test]
    fn cli_rejects_judged_route_without_key() {
        let directory = tempdir().expect("temp dir should exist");
        write_route_fixture_pack(directory.path());
        let error = run_cli_command_surface(vec![
            "route-tools-for-query".to_string(),
            "--dataset".to_string(),
            directory.path().display().to_string(),
            "--query".to_string(),
            "send message to channel".to_string(),
        ])
        .expect_err("judged route should require a key");

        assert!(error
            .to_string()
            .contains("route-tools-for-query requires --api-key"));
    }

    #[test]
    fn cli_writes_report_files() {
        let report_dir = tempdir().expect("report dir should exist");
        let dataset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../A00-raw-research/benchmarks/tool-routing-subset");
        let output = run_cli_command_surface(vec![
            "evaluate-routing-subset-metrics".to_string(),
            "--dataset".to_string(),
            dataset_path.display().to_string(),
            "--mode".to_string(),
            "lexical".to_string(),
            "--report-dir".to_string(),
            report_dir.path().display().to_string(),
        ])
        .expect("cli should run");

        assert!(output.contains("token_reduction_estimate"));
        assert!(report_dir
            .path()
            .join("routing-metrics-report.json")
            .exists());
        assert!(report_dir.path().join("routing-metrics-report.md").exists());
    }

    #[test]
    fn cli_writes_comparison_files() {
        let report_dir = tempdir().expect("report dir should exist");
        let dataset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../A00-raw-research/benchmarks/tool-routing-subset");
        let output = run_cli_command_surface(vec![
            "compare-routing-modes-metrics".to_string(),
            "--dataset".to_string(),
            dataset_path.display().to_string(),
            "--report-dir".to_string(),
            report_dir.path().display().to_string(),
        ])
        .expect("cli comparison should run");

        assert!(output.contains("\"router_mode\": \"lexical\""));
        assert!(output.contains("\"router_mode\": \"schema_aware\""));
        assert!(output.contains("\"router_mode\": \"hybrid\""));
        assert!(report_dir
            .path()
            .join("routing-mode-comparison-report.json")
            .exists());
        assert!(report_dir
            .path()
            .join("routing-mode-comparison-report.md")
            .exists());
    }

    fn write_route_fixture_pack(path: &std::path::Path) {
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
                "id": "query-1",
                "query": "send message to channel",
                "required_tool_ids": ["tool.channel"],
                "should_route": true,
                "graded_relevance": [{"tool_id": "tool.channel", "relevance": 3}],
                "source_expected_tools": ["tool.channel"],
                "failure_modes": ["none"]
            }
        ]);
        std::fs::write(path.join("tools.json"), tools.to_string())
            .expect("tools fixture should write");
        std::fs::write(path.join("queries.json"), queries.to_string())
            .expect("queries fixture should write");
        std::fs::write(path.join("manifest.json"), "{\"version\":\"test\"}")
            .expect("manifest fixture should write");
    }
}
