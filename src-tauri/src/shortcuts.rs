//! Registers the configurable emergency-stop global shortcut.

use std::sync::Arc;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::runtime::{broadcast_snapshot, AppState};

fn parse(accelerator: &str) -> Result<Shortcut, String> {
    accelerator
        .parse()
        .map_err(|e| format!("invalid shortcut \"{accelerator}\": {e}"))
}

pub fn register_emergency_stop(app: &AppHandle, accelerator: &str) -> Result<(), String> {
    let shortcut = parse(accelerator)?;
    app.global_shortcut()
        .on_shortcut(shortcut, |app, _shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let state = app.state::<Arc<AppState>>().inner().clone();
                let snapshot = state.emergency_stop().await;
                broadcast_snapshot(&app, &snapshot);
            });
        })
        .map_err(|e| format!("couldn't register shortcut \"{accelerator}\": {e}"))
}

/// Swaps shortcuts, keeping the old one if the new one can't be registered.
pub fn replace_emergency_stop(app: &AppHandle, old: &str, new: &str) -> Result<(), String> {
    let (old_shortcut, new_shortcut) = (parse(old).ok(), parse(new)?);
    if old_shortcut == Some(new_shortcut) && app.global_shortcut().is_registered(new_shortcut) {
        return Ok(());
    }
    if let Some(old_shortcut) = old_shortcut {
        let _ = app.global_shortcut().unregister(old_shortcut);
    }
    register_emergency_stop(app, new).inspect_err(|_| {
        if let Err(err) = register_emergency_stop(app, old) {
            tracing::error!("emergency-stop shortcut lost: {err}");
        }
    })
}
