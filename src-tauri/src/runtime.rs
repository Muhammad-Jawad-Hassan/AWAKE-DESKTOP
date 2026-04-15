//! Ties `core` and `platform` together into a running application: owns the
//! active session, drives its tick loop, and reports snapshots to the UI.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rand::Rng;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::{watch, Mutex};

use crate::core::history::push_bounded;
use crate::core::{
    decide_outcome, enabled_activities, next_delay, split_path_steps, ActivityKind,
    ActivityOutcome, ActivityProfile, AppConfig, AppSettings, HistoryEntry, InactivityMonitor,
    PlatformCapabilities, PowerLease, Session, SessionConfig, SessionError, SessionState,
    SessionStats, SessionTemplate,
};
use crate::platform::Platform;

const TICK_INTERVAL: Duration = Duration::from_millis(1000);
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);
/// Pause between each action in a Test Activity run, long enough to actually see each one.
const TEST_ACTIVITY_GAP: Duration = Duration::from_millis(2500);

pub struct AppState {
    pub platform: Platform,
    config_dir: PathBuf,
    inner: Mutex<Inner>,
}

struct Inner {
    config: AppConfig,
    session: Option<Session>,
    power_lease: Option<Box<dyn PowerLease>>,
    inactivity: Option<InactivityMonitor>,
    activity_paused: bool,
    activity_next_fire: Option<Instant>,
    recent_actions: VecDeque<Instant>,
    last_activity: Option<(ActivityKind, Instant)>,
    stop_tx: Option<watch::Sender<bool>>,
    stats: Option<SessionStats>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshot {
    pub state: SessionState,
    pub remaining_secs: u64,
    pub elapsed_secs: u64,
    pub duration_secs: u64,
    pub keep_system_awake: bool,
    pub keep_display_awake: bool,
    pub activity_profile_id: Option<String>,
    pub user_inactive: bool,
    pub idle_secs: u64,
    pub inactivity_threshold_secs: u64,
    pub activity_paused: bool,
    pub last_activity: Option<ActivityKind>,
    pub last_activity_secs_ago: Option<u64>,
}

impl AppState {
    pub fn new(platform: Platform, config_dir: PathBuf) -> Self {
        let config = AppConfig::load(&config_dir).unwrap_or_default();
        Self {
            platform,
            config_dir,
            inner: Mutex::new(Inner {
                config,
                session: None,
                power_lease: None,
                inactivity: None,
                activity_paused: false,
                activity_next_fire: None,
                recent_actions: VecDeque::new(),
                last_activity: None,
                stop_tx: None,
                stats: None,
            }),
        }
    }

    pub async fn settings(&self) -> AppSettings {
        self.inner.lock().await.config.settings.clone()
    }

    /// Synchronous read for app startup, before anything else can contend for the lock.
    pub fn settings_blocking(&self) -> Option<AppSettings> {
        self.inner
            .try_lock()
            .ok()
            .map(|inner| inner.config.settings.clone())
    }

    pub async fn last_session_config(&self) -> Option<SessionConfig> {
        self.inner.lock().await.config.last_session_config.clone()
    }

    pub async fn session_stats(&self) -> Option<SessionStats> {
        self.inner.lock().await.stats
    }

    /// Best-effort synchronous release, for the app-exit path where we can't `.await`.
    pub fn release_power_lease_blocking(&self) {
        if let Ok(mut inner) = self.inner.try_lock() {
            inner.power_lease = None;
        }
    }

    pub async fn update_settings(&self, settings: AppSettings) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        inner.config.settings = settings;
        inner
            .config
            .save(&self.config_dir)
            .map_err(|e| e.to_string())
    }

    pub async fn profiles(&self) -> Vec<ActivityProfile> {
        self.inner.lock().await.config.profiles.clone()
    }

    pub async fn save_profile(&self, profile: ActivityProfile) -> Result<(), String> {
        profile.validate().map_err(|e| e.to_string())?;
        let mut inner = self.inner.lock().await;
        if let Some(existing) = inner
            .config
            .profiles
            .iter_mut()
            .find(|p| p.id == profile.id)
        {
            if existing.built_in {
                return Err("built-in profiles cannot be modified".to_string());
            }
            *existing = profile;
        } else {
            inner.config.profiles.push(profile);
        }
        inner
            .config
            .save(&self.config_dir)
            .map_err(|e| e.to_string())
    }

    pub async fn delete_profile(&self, id: String) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        let Some(profile) = inner.config.profiles.iter().find(|p| p.id == id) else {
            return Err("profile not found".to_string());
        };
        if profile.built_in {
            return Err("built-in profiles cannot be deleted".to_string());
        }
        inner.config.profiles.retain(|p| p.id != id);
        inner
            .config
            .save(&self.config_dir)
            .map_err(|e| e.to_string())
    }

    pub async fn templates(&self) -> Vec<SessionTemplate> {
        self.inner.lock().await.config.templates.clone()
    }

    pub async fn save_template(&self, template: SessionTemplate) -> Result<(), String> {
        template.validate().map_err(|e| e.to_string())?;
        template.config.validate().map_err(|e| e.to_string())?;
        let mut inner = self.inner.lock().await;
        if let Some(existing) = inner
            .config
            .templates
            .iter_mut()
            .find(|t| t.id == template.id)
        {
            *existing = template;
        } else {
            inner.config.templates.push(template);
        }
        inner
            .config
            .save(&self.config_dir)
            .map_err(|e| e.to_string())
    }

    pub async fn delete_template(&self, id: String) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        inner.config.templates.retain(|t| t.id != id);
        inner
            .config
            .save(&self.config_dir)
            .map_err(|e| e.to_string())
    }

    pub fn capabilities(&self) -> PlatformCapabilities {
        self.platform.capabilities.clone()
    }

    pub async fn snapshot(&self) -> SessionSnapshot {
        let inner = self.inner.lock().await;
        build_snapshot(&inner)
    }

    pub async fn start_session(
        self: std::sync::Arc<Self>,
        app: AppHandle,
        config: SessionConfig,
    ) -> Result<SessionSnapshot, String> {
        config.validate().map_err(|e| e.to_string())?;

        let mut inner = self.inner.lock().await;
        if inner.session.as_ref().is_some_and(Session::is_active) {
            return Err(SessionError::AlreadyActive.to_string());
        }

        let power_lease = if config.keep_system_awake || config.keep_display_awake {
            let lease = self
                .platform
                .power
                .acquire(
                    config.keep_system_awake,
                    config.keep_display_awake,
                    config.duration,
                )
                .map_err(|e| e.to_string())?;
            Some(lease)
        } else {
            None
        };

        let mut session = Session::idle(config.clone());
        session.start(Instant::now()).map_err(|e| e.to_string())?;

        inner.config.last_session_config = Some(config.clone());
        let _ = inner.config.save(&self.config_dir);
        inner.session = Some(session);
        inner.power_lease = power_lease;
        inner.inactivity = Some(InactivityMonitor::new(config.inactivity_threshold));
        inner.activity_paused = false;
        inner.activity_next_fire = None;
        inner.recent_actions.clear();
        inner.last_activity = None;
        inner.stats = inner
            .config
            .settings
            .record_activity_statistics
            .then(SessionStats::default);

        let (stop_tx, stop_rx) = watch::channel(false);
        inner.stop_tx = Some(stop_tx);

        let snapshot = build_snapshot(&inner);
        drop(inner);

        tauri::async_runtime::spawn(run_session_loop(self, app, stop_rx));

        Ok(snapshot)
    }

    /// Adds time to the running session, re-acquiring the power lease so its
    /// self-expiring timeout (see platform `PowerManager::acquire`) covers the new end time.
    pub async fn extend_session(&self, extra: Duration) -> Result<SessionSnapshot, String> {
        let mut inner = self.inner.lock().await;
        let session = inner
            .session
            .as_mut()
            .ok_or_else(|| SessionError::NotActive.to_string())?;
        session.extend(extra).map_err(|e| e.to_string())?;

        if inner.power_lease.is_some() {
            let now = Instant::now();
            let session = inner.session.as_ref().expect("just extended above");
            let (keep_system, keep_display) = {
                let cfg = session.config();
                (cfg.keep_system_awake, cfg.keep_display_awake)
            };
            let remaining = session.remaining(now);
            let lease = self
                .platform
                .power
                .acquire(keep_system, keep_display, remaining)
                .map_err(|e| e.to_string())?;
            inner.power_lease = Some(lease);
        }

        Ok(build_snapshot(&inner))
    }

    pub async fn stop_session(&self) -> Result<SessionSnapshot, String> {
        let mut inner = self.inner.lock().await;
        let session = inner
            .session
            .as_mut()
            .ok_or_else(|| SessionError::NotActive.to_string())?;
        session.stop().map_err(|e| e.to_string())?;
        if let Some(tx) = inner.stop_tx.take() {
            let _ = tx.send(true);
        }
        inner.power_lease = None;
        Ok(build_snapshot(&inner))
    }

    pub async fn set_activity_paused(&self, paused: bool) -> SessionSnapshot {
        let mut inner = self.inner.lock().await;
        inner.activity_paused = paused;
        inner.activity_next_fire = None;
        build_snapshot(&inner)
    }

    pub async fn session_history(&self) -> Vec<HistoryEntry> {
        self.inner.lock().await.config.history.clone()
    }

    pub async fn clear_session_history(&self) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        inner.config.history.clear();
        inner
            .config
            .save(&self.config_dir)
            .map_err(|e| e.to_string())
    }
}

/// Pushes a snapshot to every listener: the main window and the tray.
pub(crate) fn broadcast_snapshot(app: &AppHandle, snapshot: &SessionSnapshot) {
    let _ = app.emit("session://update", snapshot);
    crate::tray::update(app, snapshot);
}

fn build_snapshot(inner: &Inner) -> SessionSnapshot {
    let now = Instant::now();
    let (
        state,
        remaining_secs,
        elapsed_secs,
        duration_secs,
        keep_system_awake,
        keep_display_awake,
        profile_id,
        inactivity_threshold_secs,
    ) = match &inner.session {
        Some(session) => (
            session.state(),
            session.remaining(now).as_secs(),
            session.timer().map_or(0, |t| t.elapsed(now).as_secs()),
            session.config().duration.as_secs(),
            session.config().keep_system_awake,
            session.config().keep_display_awake,
            session.config().activity_profile_id.clone(),
            session.config().inactivity_threshold.as_secs(),
        ),
        None => (SessionState::Idle, 0, 0, 0, false, false, None, 0),
    };

    let user_inactive = inner
        .inactivity
        .as_ref()
        .is_some_and(InactivityMonitor::is_inactive);
    let idle_secs = inner
        .inactivity
        .as_ref()
        .map_or(0, |m| m.effective_idle().as_secs());
    let (last_activity, last_activity_secs_ago) = match inner.last_activity {
        Some((kind, at)) => (
            Some(kind),
            Some(now.saturating_duration_since(at).as_secs()),
        ),
        None => (None, None),
    };

    SessionSnapshot {
        state,
        remaining_secs,
        elapsed_secs,
        duration_secs,
        keep_system_awake,
        keep_display_awake,
        activity_profile_id: profile_id,
        user_inactive,
        idle_secs,
        inactivity_threshold_secs,
        activity_paused: inner.activity_paused,
        last_activity,
        last_activity_secs_ago,
    }
}

async fn run_session_loop(
    state: std::sync::Arc<AppState>,
    app: AppHandle,
    mut stop_rx: watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            _ = stop_rx.changed() => break,
            _ = tokio::time::sleep(TICK_INTERVAL) => {}
        }
        if *stop_rx.borrow() {
            break;
        }

        let now = Instant::now();
        // Queried before locking, because on Linux this spawns `xprintidle` and would stall every other command.
        let idle_secs = state.platform.idle.idle_seconds().unwrap_or(0.0);

        let mut inner = state.inner.lock().await;

        let power_lease_died = inner
            .power_lease
            .as_mut()
            .is_some_and(|lease| !lease.is_alive());
        if power_lease_died {
            if let Some(session) = inner.session.as_mut() {
                tracing::error!("power-management process exited unexpectedly; failing session");
                session.fail();
            }
        }

        let new_state = match inner.session.as_mut() {
            Some(session) => session.tick(now),
            None => break,
        };

        if let Some(monitor) = inner.inactivity.as_mut() {
            monitor.poll(now, Duration::from_secs_f64(idle_secs));
        }

        if new_state == SessionState::Active {
            let user_inactive = inner
                .inactivity
                .as_ref()
                .is_some_and(InactivityMonitor::is_inactive);
            if let Some(stats) = inner.stats.as_mut() {
                stats.record_tick(TICK_INTERVAL, user_inactive);
            }
        }

        if new_state == SessionState::Active && !inner.activity_paused {
            tick_activity(&mut inner, &state.platform, now);
        }

        let snapshot = build_snapshot(&inner);
        let session_ended = new_state != SessionState::Active;
        if session_ended {
            inner.power_lease = None;
            record_history_entry(&state, &mut inner);
        }
        let notify_on_end = inner.config.settings.notify_on_session_end;
        drop(inner);

        broadcast_snapshot(&app, &snapshot);

        if session_ended {
            if notify_on_end {
                notify_session_ended(&app, new_state);
            }
            break;
        }
    }
}

fn tick_activity(inner: &mut Inner, platform: &Platform, now: Instant) {
    let Some(profile_id) = inner
        .session
        .as_ref()
        .and_then(|s| s.config().activity_profile_id.clone())
    else {
        return;
    };
    let Some(profile) = inner
        .config
        .profiles
        .iter()
        .find(|p| p.id == profile_id)
        .cloned()
    else {
        return;
    };
    let Some(monitor) = inner.inactivity.as_ref() else {
        return;
    };
    let user_inactive = monitor.is_inactive();

    if !user_inactive {
        inner.activity_next_fire = None;
        return;
    }

    match inner.activity_next_fire {
        None => {
            let delay = next_delay(
                profile.min_delay,
                profile.max_delay,
                &mut rand::thread_rng(),
            );
            inner.activity_next_fire = Some(now + delay);
        }
        Some(fire_at) if now >= fire_at => {
            inner.activity_next_fire = None;
            let enabled = enabled_activities(&profile);
            if let ActivityOutcome::Performed(kind) =
                decide_outcome(&enabled, true, &mut rand::thread_rng())
            {
                if within_rate_limit(inner, &profile, now)
                    && perform_activity(platform, &profile, kind).is_ok()
                {
                    if let Some(monitor) = inner.inactivity.as_mut() {
                        monitor.notify_synthetic_action(now);
                    }
                    if let Some(stats) = inner.stats.as_mut() {
                        stats.record_automated_event();
                    }
                    inner.last_activity = Some((kind, now));
                    inner.recent_actions.push_back(now);
                }
            }
        }
        _ => {}
    }
}

fn within_rate_limit(inner: &mut Inner, profile: &ActivityProfile, now: Instant) -> bool {
    while inner
        .recent_actions
        .front()
        .is_some_and(|t| now.saturating_duration_since(*t) > RATE_LIMIT_WINDOW)
    {
        inner.recent_actions.pop_front();
    }
    inner.recent_actions.len() < profile.safety.max_actions_per_minute as usize
}

/// Performs `kind`, or skips it (returning `Ok(false)`) if the safety check
/// blocks it - e.g. the cursor is currently too close to a screen corner.
fn perform_activity(
    platform: &Platform,
    profile: &ActivityProfile,
    kind: ActivityKind,
) -> Result<bool, crate::core::PlatformError> {
    let mut rng = rand::thread_rng();
    match kind {
        ActivityKind::MouseMovement => {
            if !cursor_clear_of_corners(platform, profile.safety.avoid_screen_corners_px)? {
                return Ok(false);
            }
            let dx = rng.gen_range(-40..=40);
            let dy = rng.gen_range(-40..=40);
            move_mouse_along_path(platform, dx, dy, &mut rng)?;
            Ok(true)
        }
        ActivityKind::MouseClick => {
            if !cursor_clear_of_corners(platform, profile.safety.avoid_screen_corners_px)? {
                return Ok(false);
            }
            platform
                .input
                .click_mouse(profile.mouse.button, profile.mouse.click)?;
            Ok(true)
        }
        ActivityKind::KeyboardInput => {
            platform
                .input
                .key_tap(&profile.keyboard.key, &profile.keyboard.modifiers)?;
            Ok(true)
        }
        ActivityKind::GestureHorizontal => {
            platform.input.scroll(rng.gen_range(-5..=5), 0)?;
            Ok(true)
        }
        ActivityKind::GestureVertical => {
            platform.input.scroll(0, rng.gen_range(-5..=5))?;
            Ok(true)
        }
    }
}

/// Traces a relative move over several small steps instead of teleporting,
/// so the movement reads as human to anything watching the cursor.
fn move_mouse_along_path(
    platform: &Platform,
    total_dx: i32,
    total_dy: i32,
    rng: &mut impl Rng,
) -> Result<(), crate::core::PlatformError> {
    let steps = rng.gen_range(4..=7);
    let deltas = split_path_steps(total_dx, total_dy, steps);
    let last = deltas.len() - 1;
    for (i, (step_dx, step_dy)) in deltas.into_iter().enumerate() {
        platform.input.move_mouse_relative(step_dx, step_dy)?;
        if i < last {
            std::thread::sleep(Duration::from_millis(rng.gen_range(12..=28)));
        }
    }
    Ok(())
}

fn cursor_clear_of_corners(
    platform: &Platform,
    margin_px: u32,
) -> Result<bool, crate::core::PlatformError> {
    let (x, y) = platform.input.cursor_position()?;
    let (width, height) = platform.input.display_size()?;
    let margin = margin_px as i32;
    let near_left = x < margin;
    let near_right = x > width - margin;
    let near_top = y < margin;
    let near_bottom = y > height - margin;
    Ok(!((near_left || near_right) && (near_top || near_bottom)))
}

/// Appends a bounded history entry when a session ends, if stats were being recorded.
fn record_history_entry(state: &AppState, inner: &mut Inner) {
    if !inner.config.settings.record_activity_statistics {
        return;
    }
    let (Some(stats), Some(session)) = (inner.stats, inner.session.as_ref()) else {
        return;
    };
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let entry = HistoryEntry {
        // Millisecond precision so two sessions ending in the same second
        // (e.g. a quick "Start Again") never collide on id.
        id: format!("history-{}", since_epoch.as_millis()),
        ended_at_unix_secs: since_epoch.as_secs(),
        duration_secs: session.config().duration.as_secs(),
        activity_profile_id: session.config().activity_profile_id.clone(),
        stats,
    };
    push_bounded(&mut inner.config.history, entry);
    let _ = inner.config.save(&state.config_dir);
}

fn notify_session_ended(app: &AppHandle, state: SessionState) {
    use tauri_plugin_notification::NotificationExt;

    let body = match state {
        SessionState::Completed => "Your Awake session finished.",
        SessionState::Stopped => "Your Awake session was stopped.",
        SessionState::Failed => "Your Awake session ended unexpectedly.",
        SessionState::Idle | SessionState::Active => return,
    };
    let _ = app
        .notification()
        .builder()
        .title("Awake")
        .body(body)
        .show();
}
