//! Application entry point: wires plugins, managed state, tray and commands.

mod automation;
mod commands;
mod core;
mod platform;
mod runtime;
mod shortcuts;
mod tray;
mod window_handle;

use std::sync::Arc;

use tauri::{Manager, RunEvent, WindowEvent};

use runtime::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle();
            let config_dir = handle
                .path()
                .app_config_dir()
                .expect("no app config directory available on this platform");

            let platform = platform::current();
            let state = AppState::new(platform, config_dir);

            // Hide before first paint.
            if state.settings_blocking().is_some_and(|s| s.start_minimized) {
                if let Some(window) = handle.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            let state = Arc::new(state);
            app.manage(state.clone());

            tray::build(handle)?;

            let app_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                let settings = state.settings().await;
                if let Err(err) = shortcuts::register_emergency_stop(
                    &app_handle,
                    &settings.emergency_stop_shortcut,
                ) {
                    state.push_notice(format!(
                        "The emergency-stop shortcut isn't active: {err}. Pick another in Settings."
                    ));
                }
                commands::apply_screen_capture_exclusion(
                    &app_handle,
                    &state,
                    settings.exclude_from_screen_capture,
                );
                commands::apply_launch_at_login(&app_handle, settings.launch_at_login);
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle().clone();
                let window = window.clone();
                api.prevent_close();
                tauri::async_runtime::spawn(async move {
                    let state = app.state::<Arc<AppState>>().inner().clone();
                    let close_to_tray = state.settings().await.close_to_tray;
                    if close_to_tray {
                        let _ = window.hide();
                    } else {
                        app.exit(0);
                    }
                });
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::get_last_session_config,
            commands::take_notices,
            commands::list_session_history,
            commands::clear_session_history,
            commands::start_session,
            commands::stop_session,
            commands::extend_session,
            commands::pause_activity,
            commands::resume_activity,
            commands::emergency_stop,
            commands::list_profiles,
            commands::get_limits,
            commands::new_profile,
            commands::save_profile,
            commands::delete_profile,
            commands::export_profile,
            commands::import_profile,
            commands::test_activity,
            commands::cancel_test_activity,
            commands::list_templates,
            commands::save_template,
            commands::delete_template,
            commands::get_settings,
            commands::update_settings,
            commands::get_capabilities,
            commands::request_permissions,
        ])
        .build(tauri::generate_context!())
        .expect("error while building the Awake application")
        .run(|app_handle, event| {
            let Some(state) = app_handle.try_state::<Arc<AppState>>() else {
                return;
            };
            match event {
                // Finish the session before exiting.
                RunEvent::ExitRequested { api, code, .. } if !state.is_shut_down() => {
                    if code == Some(tauri::RESTART_EXIT_CODE) {
                        tauri::async_runtime::block_on(state.shutdown());
                        return;
                    }
                    api.prevent_exit();
                    let state = state.inner().clone();
                    let app = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        state.shutdown().await;
                        app.exit(code.unwrap_or(0));
                    });
                }
                // Cmd+Q and logoff skip ExitRequested.
                RunEvent::Exit => tauri::async_runtime::block_on(state.shutdown()),
                _ => {}
            }
        });
}
