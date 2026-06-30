use benchmark_eval_metrics_runner::{
    compare_routing_modes_metrics, evaluate_routing_subset_metrics, write_comparison_reports_files,
    write_evaluation_reports_files, RoutingMetricsRequestData,
};
use catalog_router_core_engine::{RouterModeNameData, RouterTypedErrorKind};
use std::path::PathBuf;
use std::str::FromStr;

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
            mode: "usage: evaluate-routing-subset-metrics|compare-routing-modes-metrics"
                .to_string(),
        });
    };
    if !matches!(
        command_name,
        "evaluate-routing-subset-metrics" | "compare-routing-modes-metrics"
    ) {
        return Err(RouterTypedErrorKind::UnsupportedRouterMode {
            mode: "usage: evaluate-routing-subset-metrics|compare-routing-modes-metrics"
                .to_string(),
        });
    }

    let mut dataset_path = None;
    let mut report_dir = None;
    let mut router_mode = RouterModeNameData::Lexical;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--dataset" => {
                index += 1;
                dataset_path = args.get(index).cloned();
            }
            "--mode" => {
                index += 1;
                let mode =
                    args.get(index)
                        .ok_or_else(|| RouterTypedErrorKind::UnsupportedRouterMode {
                            mode: "missing mode value".to_string(),
                        })?;
                router_mode = RouterModeNameData::from_str(mode)?;
            }
            "--report-dir" => {
                index += 1;
                report_dir = args.get(index).map(PathBuf::from);
            }
            _ => {}
        }
        index += 1;
    }
    let request = RoutingMetricsRequestData {
        dataset_path,
        catalog_tools: None,
        query_records: None,
        router_mode,
        max_k: 10,
        threshold: 2.0,
    };
    if command_name == "compare-routing-modes-metrics" {
        let reports = compare_routing_modes_metrics(request)?;
        if let Some(report_dir) = report_dir {
            write_comparison_reports_files(&reports, &report_dir)?;
        }
        return serde_json::to_string_pretty(&reports).map_err(RouterTypedErrorKind::from);
    }

    let report = evaluate_routing_subset_metrics(request)?;
    if let Some(report_dir) = report_dir {
        write_evaluation_reports_files(&report, &report_dir)?;
    }
    serde_json::to_string_pretty(&report).map_err(RouterTypedErrorKind::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

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
}
