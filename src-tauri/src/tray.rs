//! System tray/menu-bar UI: the app's primary surface.

use std::sync::Arc;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Emitter, Manager};

use crate::core::SessionState;
use crate::runtime::{broadcast_snapshot, AppState, SessionSnapshot};

/// Menu item handles kept around so ticks can update their text in place.
pub struct TrayHandles {
    status: MenuItem<tauri::Wry>,
    toggle_activity: MenuItem<tauri::Wry>,
}

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let status = MenuItem::with_id(app, "status", "Awake · Idle", false, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let toggle_activity =
        MenuItem::with_id(app, "toggle_activity", "Pause Activity", true, None::<&str>)?;
    let stop_session = MenuItem::with_id(app, "stop_session", "Stop Session", true, None::<&str>)?;
    let show_window = MenuItem::with_id(app, "show", "Open Awake", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &status,
            &separator,
            &toggle_activity,
            &stop_session,
            &separator,
            &show_window,
            &settings,
            &quit,
        ],
    )?;

    app.manage(TrayHandles {
        status: status.clone(),
        toggle_activity: toggle_activity.clone(),
    });

    TrayIconBuilder::with_id("main-tray")
        .icon(tauri::include_image!("icons/tray-icon.png"))
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("Awake")
        .on_menu_event(handle_menu_event)
        .build(app)?;

    Ok(())
}

/// Refreshes the tray's status line, tooltip and pause/resume label.
pub fn update(app: &AppHandle, snapshot: &SessionSnapshot) {
    let Some(handles) = app.try_state::<TrayHandles>() else {
        return;
    };

    let status_text = match snapshot.state {
        SessionState::Active => format!("Awake · {}", format_remaining(snapshot.remaining_secs)),
        SessionState::Idle => "Awake · Idle".to_string(),
        SessionState::Completed => "Awake · Session complete".to_string(),
        SessionState::Stopped => "Awake · Session stopped".to_string(),
        SessionState::Failed => "Awake · Session failed".to_string(),
    };
    let _ = handles.status.set_text(&status_text);

    let toggle_text = if snapshot.activity_paused {
        "Resume Activity"
    } else {
        "Pause Activity"
    };
    let _ = handles.toggle_activity.set_text(toggle_text);

    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = TrayIcon::set_tooltip(&tray, Some(&status_text));
    }
}

fn format_remaining(total_secs: u64) -> String {
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "stop_session" => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let state = app.state::<Arc<AppState>>().inner().clone();
                if let Ok(snapshot) = state.stop_session().await {
                    broadcast_snapshot(&app, &snapshot);
                }
            });
        }
        "toggle_activity" => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let state = app.state::<Arc<AppState>>().inner().clone();
                let paused = state.snapshot().await.activity_paused;
                let snapshot = state.set_activity_paused(!paused).await;
                broadcast_snapshot(&app, &snapshot);
            });
        }
        "show" | "settings" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                if event.id().as_ref() == "settings" {
                    let _ = app.emit("navigate", "settings");
                }
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}
