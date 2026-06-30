mod commands;

use pie_tauri_core::{DiagnosticLogger, OpenAiHttpClient, SqliteVersionStore};
use tauri::Manager;

pub fn run() {
    let builder = tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let logger = DiagnosticLogger::open_at_path(app_data_dir.join("logs").join("pie.log"))?;
            let _ = logger.record_info_event("app_started", None, "PIE started.");
            let store =
                SqliteVersionStore::open_store_at_path(app_data_dir.join("pie-versions.sqlite"))?;
            app.manage(logger);
            app.manage(store);
            app.manage(OpenAiHttpClient::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_readiness,
            commands::validate_api_key,
            commands::analyze_prompt,
            commands::export_findings,
            commands::export_diagnostic_logs,
            commands::apply_selected_fixes,
            commands::reverify_prompt,
            commands::verify_and_export_update,
            commands::analyze_updated_prompt,
        ]);

    if let Err(error) = builder.run(tauri::generate_context!()) {
        eprintln!("failed to run PIE Tauri app: {error}");
    }
}
