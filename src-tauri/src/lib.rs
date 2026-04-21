//! Application entry point: wires plugins, managed state and the tray.

mod core;
mod platform;
mod runtime;
mod tray;

use std::sync::Arc;

use tauri::Manager;

use runtime::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle();
            let config_dir = handle
                .path()
                .app_config_dir()
                .expect("no app config directory available on this platform");

            let platform = platform::current();
            app.manage(Arc::new(AppState::new(platform, config_dir)));

            tray::build(handle)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the Awake application");
}
