//! Tauri command handlers: the thin IPC boundary between the UI and `runtime::AppState`.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::core::{
    ActivityProfile, AppSettings, HistoryEntry, PlatformCapabilities, SessionConfig, SessionStats,
    SessionTemplate,
};
use crate::runtime::{broadcast_snapshot, AppState, SessionSnapshot};
use crate::shortcuts;

#[tauri::command]
pub async fn get_snapshot(state: State<'_, Arc<AppState>>) -> Result<SessionSnapshot, String> {
    Ok(state.snapshot().await)
}

#[tauri::command]
pub async fn get_last_session_config(
    state: State<'_, Arc<AppState>>,
) -> Result<Option<SessionConfig>, String> {
    Ok(state.last_session_config().await)
}

#[tauri::command]
pub async fn get_session_stats(
    state: State<'_, Arc<AppState>>,
) -> Result<Option<SessionStats>, String> {
    Ok(state.session_stats().await)
}

#[tauri::command]
pub async fn list_session_history(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<HistoryEntry>, String> {
    Ok(state.session_history().await)
}

#[tauri::command]
pub async fn clear_session_history(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.clear_session_history().await
}

#[tauri::command]
pub async fn start_session(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    config: SessionConfig,
) -> Result<SessionSnapshot, String> {
    let state = state.inner().clone();
    let snapshot = state.start_session(app.clone(), config).await?;
    broadcast_snapshot(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn stop_session(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<SessionSnapshot, String> {
    let snapshot = state.stop_session().await?;
    broadcast_snapshot(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn extend_session(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    extra_secs: u64,
) -> Result<SessionSnapshot, String> {
    let snapshot = state
        .extend_session(Duration::from_secs(extra_secs))
        .await?;
    broadcast_snapshot(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn pause_activity(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<SessionSnapshot, String> {
    let snapshot = state.set_activity_paused(true).await;
    broadcast_snapshot(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn resume_activity(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<SessionSnapshot, String> {
    let snapshot = state.set_activity_paused(false).await;
    broadcast_snapshot(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn emergency_stop(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<SessionSnapshot, String> {
    let snapshot = state.set_activity_paused(true).await;
    broadcast_snapshot(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub async fn list_profiles(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<ActivityProfile>, String> {
    Ok(state.profiles().await)
}

#[tauri::command]
pub async fn save_profile(
    state: State<'_, Arc<AppState>>,
    profile: ActivityProfile,
) -> Result<(), String> {
    state.save_profile(profile).await
}

#[tauri::command]
pub async fn delete_profile(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    state.delete_profile(id).await
}

/// Fires one activity from `profile` right now, so the profile editor can
/// preview what it does without waiting for the random delay to elapse.
#[tauri::command]
pub async fn test_activity(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    profile: ActivityProfile,
) -> Result<Vec<crate::runtime::TestActivityResult>, String> {
    state.test_activity(&app, profile).await
}

/// Exports a profile to a user-chosen `.json` file. Returns `false` if the user cancels.
#[tauri::command]
pub async fn export_profile(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<bool, String> {
    let profile = state
        .profiles()
        .await
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| "profile not found".to_string())?;
    let json = serde_json::to_string_pretty(&profile).map_err(|e| e.to_string())?;
    let file_name = format!("{}.json", slugify(&profile.name));

    tauri::async_runtime::spawn_blocking(move || {
        let path = app
            .dialog()
            .file()
            .add_filter("Awake Profile", &["json"])
            .set_file_name(&file_name)
            .blocking_save_file();
        let Some(path) = path else {
            return Ok(false);
        };
        let path = path.into_path().map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())?;
        Ok(true)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Imports a profile from a user-chosen `.json` file, assigning it a fresh id
/// so it never overwrites an existing one. `None` if the user cancels.
#[tauri::command]
pub async fn import_profile(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<Option<ActivityProfile>, String> {
    let mut profile = tauri::async_runtime::spawn_blocking(move || {
        let path = app
            .dialog()
            .file()
            .add_filter("Awake Profile", &["json"])
            .blocking_pick_file();
        let Some(path) = path else {
            return Ok::<_, String>(None);
        };
        let path = path.into_path().map_err(|e| e.to_string())?;
        let contents = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let profile: ActivityProfile =
            serde_json::from_str(&contents).map_err(|e| format!("invalid profile file: {e}"))?;
        Ok(Some(profile))
    })
    .await
    .map_err(|e| e.to_string())??;

    if let Some(profile) = profile.as_mut() {
        profile.id = format!("custom-{}", unique_suffix());
        profile.built_in = false;
        state.save_profile(profile.clone()).await?;
    }
    Ok(profile)
}

fn slugify(name: &str) -> String {
    let slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "profile".to_string()
    } else {
        slug.to_string()
    }
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default()
}

#[tauri::command]
pub async fn list_templates(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<SessionTemplate>, String> {
    Ok(state.templates().await)
}

#[tauri::command]
pub async fn save_template(
    state: State<'_, Arc<AppState>>,
    template: SessionTemplate,
) -> Result<(), String> {
    state.save_template(template).await
}

#[tauri::command]
pub async fn delete_template(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    state.delete_template(id).await
}

#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> Result<AppSettings, String> {
    Ok(state.settings().await)
}

#[tauri::command]
pub async fn update_settings(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    settings: AppSettings,
) -> Result<(), String> {
    state.update_settings(settings.clone()).await?;
    shortcuts::reregister_emergency_stop(&app, &settings.emergency_stop_shortcut);
    apply_screen_capture_exclusion(&app, state.inner(), settings.exclude_from_screen_capture);
    apply_launch_at_login(&app, settings.launch_at_login);
    Ok(())
}

/// Syncs the OS launch-at-login registration to match the setting.
pub fn apply_launch_at_login(app: &AppHandle, enabled: bool) {
    use tauri_plugin_autostart::ManagerExt;

    let autolaunch = app.autolaunch();
    let result = if enabled {
        autolaunch.enable()
    } else {
        autolaunch.disable()
    };
    if let Err(err) = result {
        tracing::warn!("failed to sync launch-at-login: {err}");
    }
}

#[tauri::command]
pub fn get_capabilities(state: State<'_, Arc<AppState>>) -> PlatformCapabilities {
    state.capabilities()
}

/// Applies the screen-capture-exclusion setting to the main window.
/// Dispatched to the main thread, since AppKit refuses window calls off it.
pub fn apply_screen_capture_exclusion(app: &AppHandle, state: &Arc<AppState>, exclude: bool) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let visibility = state.platform.visibility.clone();
    let target = window.clone();
    if let Err(err) =
        window.run_on_main_thread(move || match crate::window_handle::native_handle(&target) {
            Ok(handle) => {
                if let Err(err) = visibility.set_exclude_from_capture(handle, exclude) {
                    tracing::warn!("failed to set screen-capture exclusion: {err}");
                }
            }
            Err(err) => tracing::warn!("failed to resolve native window handle: {err}"),
        })
    {
        tracing::warn!("failed to dispatch screen-capture exclusion to main thread: {err}");
    }
}

#[tauri::command]
pub async fn request_permissions() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        crate::platform::macos::open_accessibility_settings().map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(())
    }
}
