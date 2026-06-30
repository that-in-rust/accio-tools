use benchmark_eval_metrics_runner::{evaluate_routing_subset_metrics, RoutingMetricsRequestData};
use catalog_router_core_engine::{RouterModeNameData, RouterTypedErrorKind};
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
    if args.first().map(String::as_str) != Some("evaluate-routing-subset-metrics") {
        return Err(RouterTypedErrorKind::UnsupportedRouterMode {
            mode: "usage: evaluate-routing-subset-metrics --dataset <path> --mode <mode>"
                .to_string(),
        });
    }
    let mut dataset_path = None;
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
            _ => {}
        }
        index += 1;
    }
    let report = evaluate_routing_subset_metrics(RoutingMetricsRequestData {
        dataset_path,
        router_mode,
        max_k: 10,
        threshold: 2.0,
    })?;
    serde_json::to_string_pretty(&report).map_err(RouterTypedErrorKind::from)
}
