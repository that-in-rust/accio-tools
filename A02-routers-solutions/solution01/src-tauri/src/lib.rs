mod commands;

pub fn run() {
    let builder = tauri::Builder::default().invoke_handler(tauri::generate_handler![
        commands::validate_judge_api_key,
        commands::run_cpu_preview_only,
        commands::route_tools_for_query,
        commands::evaluate_routing_subset_metrics,
        commands::download_evaluation_pack_files,
        commands::export_route_evidence_report,
        commands::export_diagnostic_logs_text,
    ]);

    if let Err(error) = builder.run(tauri::generate_context!()) {
        eprintln!("failed to run Tool Router Tauri app: {error}");
    }
}
