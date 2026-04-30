//! Registers the configurable emergency-stop global shortcut.

use std::sync::Arc;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::runtime::{broadcast_snapshot, AppState};

pub fn reregister_emergency_stop(app: &AppHandle, accelerator: &str) {
    let shortcuts = app.global_shortcut();
    let _ = shortcuts.unregister_all();

    let result = shortcuts.on_shortcut(accelerator, move |app, _shortcut, event| {
        if event.state() != ShortcutState::Pressed {
            return;
        }
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let state = app.state::<Arc<AppState>>().inner().clone();
            let snapshot = state.set_activity_paused(true).await;
            broadcast_snapshot(&app, &snapshot);
        });
    });

    if let Err(err) = result {
        tracing::warn!("failed to register emergency-stop shortcut '{accelerator}': {err}");
    }
}
