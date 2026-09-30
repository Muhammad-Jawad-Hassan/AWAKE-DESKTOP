//! Ties `core` and `platform` together: owns the session, drives its tick loop,
//! and reports snapshots to the UI.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::{watch, Mutex};

use crate::automation::{self, Outcome};
use crate::core::activity::{ActivityScheduler, RateLimiter};
use crate::core::history::push_bounded;
use crate::core::{
    enabled_activities, ActivityKind, ActivityProfile, AppConfig, AppSettings, HistoryEntry,
    InactivityMonitor, PlatformCapabilities, PlatformError, PowerLease, Session, SessionConfig,
    SessionError, SessionState, SessionStats, SessionTemplate,
};
use crate::platform::Platform;

const TICK_INTERVAL: Duration = Duration::from_secs(1);
/// Long enough to see each action in a Test Activity run.
const TEST_ACTIVITY_GAP: Duration = Duration::from_millis(2500);

/// Broadcast as each Test Activity step starts.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestActivityProgress {
    pub kind: ActivityKind,
    pub index: u32,
    pub total: u32,
}

/// `performed` is false when a safety check skipped the action.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestActivityResult {
    pub kind: ActivityKind,
    pub performed: bool,
}

pub struct AppState {
    pub platform: Platform,
    config_dir: PathBuf,
    /// Test runs with an id at or below this are cancelled.
    tests_cancelled_through: Arc<AtomicU64>,
    latest_test_run: AtomicU64,
    test_running: AtomicBool,
    /// True once shutdown has finished; held while it runs.
    shut_down: Mutex<bool>,
    notices: std::sync::Mutex<Vec<String>>,
    inner: Mutex<Inner>,
}

struct Inner {
    config: AppConfig,
    session: Option<Session>,
    /// Validated copy of the session's profile, fixed for its lifetime.
    profile: Option<ActivityProfile>,
    /// Bumped per session so a stale tick loop can't touch a newer one.
    generation: u64,
    /// Cuts off in-flight input on stop, pause or emergency stop.
    session_abort: Arc<AtomicBool>,
    power_lease: Option<Box<dyn PowerLease>>,
    inactivity: Option<InactivityMonitor>,
    activity_paused: bool,
    scheduler: ActivityScheduler,
    limiter: RateLimiter,
    last_activity: Option<(ActivityKind, Instant)>,
    activity_error: Option<String>,
    idle_error: Option<String>,
    last_tick_at: Option<Instant>,
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
    /// Why automation isn't working right now, if it isn't.
    pub activity_warning: Option<String>,
    pub stats: Option<SessionStats>,
}

/// What one tick decided, handed back to the loop outside the lock.
struct Tick {
    snapshot: SessionSnapshot,
    actions: Vec<ActivityKind>,
    ended: Option<SessionState>,
    notify_on_end: bool,
}

impl AppState {
    pub fn new(platform: Platform, config_dir: PathBuf) -> Self {
        let (config, warning) = AppConfig::load_or_recover(&config_dir);
        Self {
            platform,
            config_dir,
            tests_cancelled_through: Arc::new(AtomicU64::new(0)),
            latest_test_run: AtomicU64::new(0),
            test_running: AtomicBool::new(false),
            shut_down: Mutex::new(false),
            notices: std::sync::Mutex::new(warning.into_iter().collect()),
            inner: Mutex::new(Inner {
                config,
                session: None,
                profile: None,
                generation: 0,
                session_abort: Arc::new(AtomicBool::new(true)),
                power_lease: None,
                inactivity: None,
                activity_paused: false,
                scheduler: ActivityScheduler::default(),
                limiter: RateLimiter::default(),
                last_activity: None,
                activity_error: None,
                idle_error: None,
                last_tick_at: None,
                stop_tx: None,
                stats: None,
            }),
        }
    }

    /// Queues a message for the UI to show once.
    pub fn push_notice(&self, notice: String) {
        tracing::warn!("{notice}");
        self.notices
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(notice);
    }

    pub fn take_notices(&self) -> Vec<String> {
        std::mem::take(
            &mut *self
                .notices
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }

    /// Changes a copy of the config and only adopts it once it's saved.
    fn update_config(
        &self,
        inner: &mut Inner,
        change: impl FnOnce(&mut AppConfig) -> Result<(), String>,
    ) -> Result<(), String> {
        let mut next = inner.config.clone();
        change(&mut next)?;
        next.save(&self.config_dir)
            .map_err(|e| format!("couldn't save settings: {e}"))?;
        inner.config = next;
        Ok(())
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

    pub async fn update_settings(&self, settings: AppSettings) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        self.update_config(&mut inner, |config| {
            config.settings = settings;
            Ok(())
        })
    }

    pub async fn last_session_config(&self) -> Option<SessionConfig> {
        self.inner.lock().await.config.last_session_config.clone()
    }

    pub async fn profiles(&self) -> Vec<ActivityProfile> {
        self.inner.lock().await.config.profiles.clone()
    }

    pub async fn save_profile(&self, profile: ActivityProfile) -> Result<(), String> {
        profile.validate().map_err(|e| e.to_string())?;
        let mut inner = self.inner.lock().await;
        self.update_config(&mut inner, |config| {
            match config.profiles.iter_mut().find(|p| p.id == profile.id) {
                Some(existing) if existing.built_in => {
                    Err("built-in profiles cannot be modified".to_string())
                }
                Some(existing) => {
                    *existing = profile;
                    Ok(())
                }
                None if profile.built_in => Err("only custom profiles can be added".to_string()),
                None => {
                    config.profiles.push(profile);
                    Ok(())
                }
            }
        })
    }

    pub async fn delete_profile(&self, id: String) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        let in_use = inner.session.as_ref().is_some_and(|s| {
            s.is_active() && s.config().activity_profile_id.as_deref() == Some(id.as_str())
        });
        if in_use {
            return Err("this profile is in use by the running session".to_string());
        }
        self.update_config(&mut inner, |config| {
            match config.profiles.iter().find(|p| p.id == id) {
                None => return Err("profile not found".to_string()),
                Some(p) if p.built_in => {
                    return Err("built-in profiles cannot be deleted".to_string())
                }
                Some(_) => {}
            }
            config.profiles.retain(|p| p.id != id);
            Ok(())
        })
    }

    pub async fn templates(&self) -> Vec<SessionTemplate> {
        self.inner.lock().await.config.templates.clone()
    }

    pub async fn save_template(&self, template: SessionTemplate) -> Result<(), String> {
        template.validate().map_err(|e| e.to_string())?;
        template.config.validate().map_err(|e| e.to_string())?;
        let mut inner = self.inner.lock().await;
        self.update_config(&mut inner, |config| {
            if let Some(id) = &template.config.activity_profile_id {
                if !config.profiles.iter().any(|p| &p.id == id) {
                    return Err("the template's activity profile no longer exists".to_string());
                }
            }
            match config.templates.iter_mut().find(|t| t.id == template.id) {
                Some(existing) => *existing = template,
                None => config.templates.push(template),
            }
            Ok(())
        })
    }

    pub async fn delete_template(&self, id: String) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        self.update_config(&mut inner, |config| {
            config.templates.retain(|t| t.id != id);
            Ok(())
        })
    }

    pub async fn session_history(&self) -> Vec<HistoryEntry> {
        self.inner.lock().await.config.history.clone()
    }

    pub async fn clear_session_history(&self) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        self.update_config(&mut inner, |config| {
            config.history.clear();
            Ok(())
        })
    }

    pub fn capabilities(&self) -> PlatformCapabilities {
        self.platform.current_capabilities()
    }

    pub async fn snapshot(&self) -> SessionSnapshot {
        build_snapshot(&*self.inner.lock().await)
    }

    fn check_automation_available(&self) -> Result<(), String> {
        let caps = self.platform.current_capabilities();
        if !caps.input_permission_granted {
            return Err(
                "activity automation needs Accessibility access; grant it in Settings".into(),
            );
        }
        if !caps.input_simulation {
            return Err(
                "activity automation isn't available: input simulation is unsupported here".into(),
            );
        }
        if !caps.idle_detection {
            return Err(
                "activity automation isn't available: inactivity detection is unsupported here"
                    .into(),
            );
        }
        Ok(())
    }

    pub async fn start_session(
        self: Arc<Self>,
        app: AppHandle,
        config: SessionConfig,
    ) -> Result<SessionSnapshot, String> {
        config.validate().map_err(|e| e.to_string())?;

        let mut inner = self.inner.lock().await;
        if inner.session.as_ref().is_some_and(Session::is_active) {
            return Err(SessionError::AlreadyActive.to_string());
        }

        let profile = match &config.activity_profile_id {
            None => None,
            Some(id) => {
                let profile = inner
                    .config
                    .profiles
                    .iter()
                    .find(|p| &p.id == id)
                    .cloned()
                    .ok_or("the selected activity profile no longer exists")?;
                profile
                    .validate()
                    .map_err(|e| format!("profile \"{}\": {e}", profile.name))?;
                self.check_automation_available()?;
                Some(profile)
            }
        };

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

        let now = Instant::now();
        let mut session = Session::idle(config.clone());
        session.start(now).map_err(|e| e.to_string())?;

        let (stop_tx, stop_rx) = watch::channel(false);
        inner.generation += 1;
        inner.session_abort = Arc::new(AtomicBool::new(false));
        inner.session = Some(session);
        inner.profile = profile;
        inner.power_lease = power_lease;
        inner.inactivity = Some(InactivityMonitor::new(config.inactivity_threshold));
        inner.activity_paused = false;
        inner.scheduler.reset();
        inner.limiter.reset();
        inner.last_activity = None;
        inner.activity_error = None;
        inner.idle_error = None;
        inner.last_tick_at = Some(now);
        inner.stop_tx = Some(stop_tx);
        inner.stats = inner
            .config
            .settings
            .record_activity_statistics
            .then(SessionStats::default);

        if let Err(err) = self.update_config(&mut inner, |c| {
            c.last_session_config = Some(config);
            Ok(())
        }) {
            tracing::warn!("{err}");
        }

        let snapshot = build_snapshot(&inner);
        let generation = inner.generation;
        drop(inner);

        tauri::async_runtime::spawn(run_session_loop(self, app, generation, stop_rx));
        Ok(snapshot)
    }

    /// Adds time, re-acquiring the power lease first so a failure changes nothing.
    pub async fn extend_session(&self, extra: Duration) -> Result<SessionSnapshot, String> {
        let mut inner = self.inner.lock().await;
        let mut extended = inner
            .session
            .clone()
            .filter(Session::is_active)
            .ok_or_else(|| SessionError::NotActive.to_string())?;
        extended.extend(extra).map_err(|e| e.to_string())?;

        if inner.power_lease.is_some() {
            let cfg = extended.config();
            let lease = self
                .platform
                .power
                .acquire(
                    cfg.keep_system_awake,
                    cfg.keep_display_awake,
                    extended.remaining(Instant::now()),
                )
                .map_err(|e| e.to_string())?;
            inner.power_lease = Some(lease);
        }
        inner.session = Some(extended);
        Ok(build_snapshot(&inner))
    }

    pub async fn stop_session(&self) -> Result<SessionSnapshot, String> {
        let mut inner = self.inner.lock().await;
        let now = Instant::now();
        inner
            .session
            .as_mut()
            .ok_or_else(|| SessionError::NotActive.to_string())?
            .stop(now)
            .map_err(|e| e.to_string())?;
        self.finish_session(&mut inner, now);
        Ok(build_snapshot(&inner))
    }

    /// Stops any running session before the app exits. Later calls wait for the first.
    pub async fn shutdown(&self) {
        let mut done = self.shut_down.lock().await;
        if *done {
            return;
        }
        self.cancel_all_tests();
        let mut inner = self.inner.lock().await;
        let now = Instant::now();
        if let Some(session) = inner.session.as_mut().filter(|s| s.is_active()) {
            let _ = session.stop(now);
            self.finish_session(&mut inner, now);
        }
        *done = true;
    }

    pub fn is_shut_down(&self) -> bool {
        self.shut_down.try_lock().is_ok_and(|done| *done)
    }

    /// Releases everything a session held. The session must already have ended.
    fn finish_session(&self, inner: &mut Inner, now: Instant) {
        if let Some(tx) = inner.stop_tx.take() {
            let _ = tx.send(true);
        }
        inner.session_abort.store(true, Ordering::SeqCst);
        inner.power_lease = None;
        inner.scheduler.reset();
        self.record_history_entry(inner, now);
    }

    fn record_history_entry(&self, inner: &mut Inner, now: Instant) {
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
            // Millis so quick restarts don't collide.
            id: format!("history-{}", since_epoch.as_millis()),
            ended_at_unix_secs: since_epoch.as_secs(),
            duration_secs: session.elapsed(now).as_secs(),
            activity_profile_id: session.config().activity_profile_id.clone(),
            stats,
        };
        if let Err(err) = self.update_config(inner, |c| {
            push_bounded(&mut c.history, entry);
            Ok(())
        }) {
            tracing::warn!("{err}");
        }
    }

    pub async fn set_activity_paused(&self, paused: bool) -> SessionSnapshot {
        let mut inner = self.inner.lock().await;
        self.apply_pause(&mut inner, paused);
        build_snapshot(&inner)
    }

    pub async fn toggle_activity_paused(&self) -> SessionSnapshot {
        let mut inner = self.inner.lock().await;
        let paused = !inner.activity_paused;
        self.apply_pause(&mut inner, paused);
        build_snapshot(&inner)
    }

    /// Pauses automation and cuts off any input in flight, including Test Activity.
    pub async fn emergency_stop(&self) -> SessionSnapshot {
        self.cancel_all_tests();
        self.set_activity_paused(true).await
    }

    /// Cancels test run `run_id`, even if it hasn't started yet.
    pub fn cancel_test_activity(&self, run_id: u64) {
        self.tests_cancelled_through
            .fetch_max(run_id, Ordering::SeqCst);
    }

    fn cancel_all_tests(&self) {
        self.cancel_test_activity(self.latest_test_run.load(Ordering::SeqCst));
    }

    fn apply_pause(&self, inner: &mut Inner, paused: bool) {
        inner.activity_paused = paused;
        inner.scheduler.reset();
        if paused {
            inner.session_abort.store(true, Ordering::SeqCst);
        } else if inner.session.as_ref().is_some_and(Session::is_active) {
            inner.session_abort = Arc::new(AtomicBool::new(false));
        }
    }

    /// Fires every enabled activity once, in order, so the editor can preview a profile.
    pub async fn test_activity(
        &self,
        app: &AppHandle,
        profile: ActivityProfile,
        run_id: u64,
    ) -> Result<Vec<TestActivityResult>, String> {
        profile.validate().map_err(|e| e.to_string())?;
        self.check_automation_available()?;
        if self
            .test_running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err("a test is already running".to_string());
        }
        self.latest_test_run.fetch_max(run_id, Ordering::SeqCst);
        let result = self.run_test(app, profile, run_id).await;
        self.test_running.store(false, Ordering::SeqCst);
        result
    }

    async fn run_test(
        &self,
        app: &AppHandle,
        profile: ActivityProfile,
        run_id: u64,
    ) -> Result<Vec<TestActivityResult>, String> {
        let enabled = enabled_activities(&profile);
        let total = enabled.len() as u32;
        let cancelled_through = self.tests_cancelled_through.clone();
        let cancelled = move || cancelled_through.load(Ordering::SeqCst) >= run_id;

        let mut results = Vec::with_capacity(enabled.len());
        for (index, kind) in enabled.into_iter().enumerate() {
            if index > 0 {
                tokio::time::sleep(TEST_ACTIVITY_GAP).await;
            }
            if cancelled() {
                break;
            }
            let _ = app.emit(
                "test-activity://progress",
                TestActivityProgress {
                    kind,
                    index: index as u32,
                    total,
                },
            );
            let input = self.platform.input.clone();
            let (profile, cancelled) = (profile.clone(), cancelled.clone());
            let outcome = tauri::async_runtime::spawn_blocking(move || {
                automation::perform(&*input, &profile, kind, &cancelled)
            })
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
            self.attribute_to_session(outcome).await;
            match outcome {
                Outcome::Performed { .. } => results.push(TestActivityResult {
                    kind,
                    performed: true,
                }),
                Outcome::Skipped => results.push(TestActivityResult {
                    kind,
                    performed: false,
                }),
                Outcome::Aborted { .. } => break,
            }
        }
        Ok(results)
    }

    /// Tells a running session that test input was ours, not the user's.
    async fn attribute_to_session(&self, outcome: Outcome) {
        if let Outcome::Performed { started, ended } | Outcome::Aborted { started, ended } = outcome
        {
            if let Some(monitor) = self.inner.lock().await.inactivity.as_mut() {
                monitor.record_synthetic_action(started, ended);
            }
        }
    }

    /// Advances the session one tick and plans the actions due now.
    /// Returns `None` once `generation` is no longer the running session.
    async fn tick(
        &self,
        generation: u64,
        idle: Result<f64, PlatformError>,
        measured_at: Instant,
    ) -> Option<Tick> {
        let mut inner = self.inner.lock().await;
        let inner = &mut *inner;
        if inner.generation != generation {
            return None;
        }
        let now = Instant::now();

        let lease_died = inner
            .power_lease
            .as_mut()
            .is_some_and(|lease| !lease.is_alive());
        let session = inner.session.as_mut().filter(|s| s.is_active())?;
        if lease_died {
            tracing::error!("power-management process exited unexpectedly; failing session");
            session.fail(now);
        }
        let state = session.tick(now);

        let monitor = inner.inactivity.as_mut()?;
        match idle {
            Ok(secs) => {
                monitor.poll(measured_at, Duration::from_secs_f64(secs));
                inner.idle_error = None;
            }
            Err(err) => {
                // Fail safe: assume the user is here.
                monitor.mark_unknown(measured_at);
                let message = format!("inactivity detection failed: {err}");
                if inner.idle_error.as_ref() != Some(&message) {
                    tracing::warn!("{message}");
                }
                inner.idle_error = Some(message);
            }
        }
        let user_inactive = monitor.is_inactive();
        let elapsed = inner
            .last_tick_at
            .replace(now)
            .map_or(TICK_INTERVAL, |last| now.saturating_duration_since(last));

        let mut actions = Vec::new();
        if state == SessionState::Active {
            if let Some(stats) = inner.stats.as_mut() {
                stats.record_tick(elapsed, user_inactive);
            }
            actions = plan_actions(inner, now, user_inactive);
        }

        let ended = (state != SessionState::Active).then_some(state);
        if ended.is_some() {
            self.finish_session(inner, now);
        }
        Some(Tick {
            snapshot: build_snapshot(inner),
            actions,
            ended,
            notify_on_end: inner.config.settings.notify_on_session_end,
        })
    }

    /// Injects `actions` off the async runtime, then records what happened.
    /// Returns `None` if the session ended meanwhile.
    async fn perform_and_record(
        &self,
        generation: u64,
        actions: Vec<ActivityKind>,
    ) -> Option<SessionSnapshot> {
        let (profile, abort) = {
            let inner = self.inner.lock().await;
            if !is_current(&inner, generation) {
                return None;
            }
            if inner.activity_paused {
                return Some(build_snapshot(&inner));
            }
            (inner.profile.clone()?, inner.session_abort.clone())
        };
        let input = self.platform.input.clone();
        let results = tauri::async_runtime::spawn_blocking(move || {
            let aborted = || abort.load(Ordering::SeqCst);
            actions
                .into_iter()
                .map(|kind| (kind, automation::perform(&*input, &profile, kind, &aborted)))
                .collect::<Vec<_>>()
        })
        .await;

        let mut inner = self.inner.lock().await;
        if !is_current(&inner, generation) {
            return None;
        }
        match results {
            Ok(results) => {
                for (kind, result) in results {
                    record_outcome(&mut inner, kind, result);
                }
            }
            Err(err) => tracing::error!("activity task panicked: {err}"),
        }
        Some(build_snapshot(&inner))
    }
}

fn is_current(inner: &Inner, generation: u64) -> bool {
    inner.generation == generation && inner.session.as_ref().is_some_and(Session::is_active)
}

/// Picks the due actions that fit under the rate cap. Clears the schedule while
/// paused or while the user is active.
fn plan_actions(inner: &mut Inner, now: Instant, user_inactive: bool) -> Vec<ActivityKind> {
    let Inner {
        profile,
        scheduler,
        limiter,
        activity_paused,
        ..
    } = inner;
    let Some(profile) = profile.as_ref() else {
        return Vec::new();
    };
    if *activity_paused || !user_inactive {
        scheduler.reset();
        return Vec::new();
    }
    let mut rng = rand::thread_rng();
    let mut planned = Vec::new();
    for kind in scheduler.due(profile, now, &mut rng) {
        if !limiter.try_acquire(now, profile.safety.max_actions_per_minute) {
            break;
        }
        scheduler.complete(kind, profile, now, &mut rng);
        planned.push(kind);
    }
    planned
}

fn record_outcome(inner: &mut Inner, kind: ActivityKind, result: Result<Outcome, PlatformError>) {
    match result {
        Ok(Outcome::Performed { started, ended }) => {
            if let Some(monitor) = inner.inactivity.as_mut() {
                monitor.record_synthetic_action(started, ended);
            }
            if let Some(stats) = inner.stats.as_mut() {
                stats.record_automated_event();
            }
            inner.last_activity = Some((kind, ended));
            inner.activity_error = None;
        }
        Ok(Outcome::Aborted { started, ended }) => {
            if let Some(monitor) = inner.inactivity.as_mut() {
                monitor.record_synthetic_action(started, ended);
            }
        }
        Ok(Outcome::Skipped) => inner.activity_error = None,
        Err(err) => {
            let message = format!("activity automation failed: {err}");
            if inner.activity_error.as_ref() != Some(&message) {
                tracing::warn!("{message}");
            }
            inner.activity_error = Some(message);
        }
    }
}

/// Pushes a snapshot to every listener: the main window and the tray.
pub(crate) fn broadcast_snapshot(app: &AppHandle, snapshot: &SessionSnapshot) {
    let _ = app.emit("session://update", snapshot);
    crate::tray::update(app, snapshot);
}

fn build_snapshot(inner: &Inner) -> SessionSnapshot {
    let now = Instant::now();
    let session = inner.session.as_ref();
    let config = session.map(Session::config);
    let monitor = inner.inactivity.as_ref();
    let automating = session.is_some_and(Session::is_active) && inner.profile.is_some();

    SessionSnapshot {
        state: session.map_or(SessionState::Idle, Session::state),
        remaining_secs: session.map_or(0, |s| s.remaining(now).as_secs()),
        elapsed_secs: session.map_or(0, |s| s.elapsed(now).as_secs()),
        duration_secs: config.map_or(0, |c| c.duration.as_secs()),
        keep_system_awake: config.is_some_and(|c| c.keep_system_awake),
        keep_display_awake: config.is_some_and(|c| c.keep_display_awake),
        activity_profile_id: config.and_then(|c| c.activity_profile_id.clone()),
        user_inactive: monitor.is_some_and(InactivityMonitor::is_inactive),
        idle_secs: monitor.map_or(0, |m| m.effective_idle().as_secs()),
        inactivity_threshold_secs: config.map_or(0, |c| c.inactivity_threshold.as_secs()),
        activity_paused: inner.activity_paused,
        last_activity: inner.last_activity.map(|(kind, _)| kind),
        last_activity_secs_ago: inner
            .last_activity
            .map(|(_, at)| now.saturating_duration_since(at).as_secs()),
        activity_warning: automating
            .then(|| {
                inner
                    .activity_error
                    .clone()
                    .or_else(|| inner.idle_error.clone())
            })
            .flatten(),
        stats: inner.stats,
    }
}

async fn run_session_loop(
    state: Arc<AppState>,
    app: AppHandle,
    generation: u64,
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

        // Unlocked: spawns a process on Linux.
        let idle = state.platform.idle.idle_seconds();
        let measured_at = Instant::now();
        let Some(tick) = state.tick(generation, idle, measured_at).await else {
            break;
        };
        let snapshot = if tick.actions.is_empty() {
            Some(tick.snapshot)
        } else {
            state.perform_and_record(generation, tick.actions).await
        };
        let Some(snapshot) = snapshot else {
            break;
        };
        broadcast_snapshot(&app, &snapshot);

        if let Some(end_state) = tick.ended {
            if tick.notify_on_end {
                notify_session_ended(&app, end_state);
            }
            break;
        }
    }
}

fn notify_session_ended(app: &AppHandle, state: SessionState) {
    use tauri_plugin_notification::NotificationExt;

    let body = match state {
        SessionState::Completed => "Your Awake session finished.",
        SessionState::Failed => "Your Awake session ended unexpectedly.",
        _ => return,
    };
    let _ = app
        .notification()
        .builder()
        .title("Awake")
        .body(body)
        .show();
}
