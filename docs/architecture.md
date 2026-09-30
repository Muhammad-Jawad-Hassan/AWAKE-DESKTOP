# Architecture

## The central abstraction: Session

Everything in Awake attaches to a **session**. A session doesn't know or care what the user is doing. It just knows a duration, whether to keep the system/display awake, an inactivity threshold, and (optionally) which activity profile to run. This is deliberate: the app must stay useful for ML training, builds, downloads, presentations, or anything else the plan didn't anticipate, without ever being coupled to any of them.

## Layering

```text
src/                          React/TypeScript frontend (thin, no business logic)
src-tauri/src/
├── core/                     Platform-agnostic domain logic
│   ├── session.rs             Session state machine (Idle → Active → Completed/Stopped/Failed)
│   ├── timer.rs                Pure countdown math, takes an explicit Instant
│   ├── inactivity.rs           Filters OS idle readings for self-inflicted resets
│   ├── activity.rs             Per-activity jittered scheduling, pure decision functions
│   ├── profiles.rs             ActivityProfile schema + built-in profiles
│   ├── config.rs               Local JSON persistence, atomic writes
│   └── ports.rs                Traits the platform layer implements (PowerManager,
│                                IdleProvider, InputSimulator, VisibilityManager)
├── platform/                 One implementation of core::ports per OS
│   ├── macos/                  caffeinate, CoreGraphics, Accessibility, NSWindow
│   ├── windows/                 SetThreadExecutionState, GetLastInputInfo, SetWindowDisplayAffinity
│   ├── linux/                   systemd-inhibit, xprintidle (X11 only)
│   └── input.rs                 Shared `enigo`-backed input simulator (all 3 OSes)
├── runtime.rs                 Owns the active session + its 1s tick loop; the only
│                                place that ties core + platform + Tauri together
├── commands/                  Tauri IPC command handlers (the JS↔Rust boundary)
├── tray.rs                    System tray/menu-bar UI
└── shortcuts.rs                Global emergency-stop shortcut
```

`core` never imports `platform` or `tauri`. `platform` implements the traits `core::ports` defines, a classic dependency-inversion / hexagonal-architecture split. `runtime.rs` is the only place allowed to depend on both.

## Why this split

Section 23/32 of the original spec requires the application core to contain no platform-specific implementation details, and a capability-based architecture so a missing OS feature degrades cleanly instead of crashing. Concretely:

- `PlatformCapabilities` is computed once at startup per OS and exposed to the UI, so the frontend can show "not supported on this platform" instead of a broken toggle.
- Every `platform::*` implementation returns `Result<_, PlatformError>` with a distinguished `Unsupported` variant instead of panicking when a capability genuinely isn't available (e.g. idle detection under Wayland).

## The synthetic-input problem

Automated mouse/keyboard input is itself a real HID event, and the OS's "seconds since last input" counter doesn't know the difference between the user and `enigo`. If the activity engine trusted that counter naively, every automated action would reset it to zero and the engine would immediately (and wrongly) decide the user had returned.

`InactivityMonitor` (`core/inactivity.rs`) solves this by timestamp. Each poll works out when the last input happened (`now - os_idle`). If that falls inside the span of our most recent synthetic action (plus a second of slack), the input was ours, and idle time keeps counting from the last _real_ input. Anything else is the user. There is no time window after which our own input starts to look like the user returning, so per-minute rates hold for as long as the user is away. This is unit-tested directly, including an hour of simulated actions.

## Runtime orchestration

`runtime::AppState` holds one `tokio::sync::Mutex<Inner>` guarding all session-related state. `start_session` validates the chosen profile, keeps its own copy for the session's lifetime, and spawns a background task (`run_session_loop`) that ticks once per second. Each tick plans under the lock (advance the timer, poll idle time, pick due actions that fit under the rate cap), injects input in `spawn_blocking` _without_ the lock so Stop and the emergency stop never wait on a mouse path, then re-locks to record the outcome. A per-session generation number stops a stale loop from touching a newer session. Stop, expiry, failure and app exit all end through one `finish_session`, and quitting mid-session finishes the session (lease released, history written) before the process exits. The tick pushes a `SessionSnapshot` to both the main window (`session://update` event) and the tray. Every command handler that mutates state (`stop_session`, `pause_activity`, …) also pushes a snapshot immediately, rather than waiting for the next tick. The loop's periodic emit and each command's immediate emit both funnel through the same `runtime::broadcast_snapshot` helper to avoid the two drifting out of sync.

## Power-management safety

A held "prevent sleep" assertion must never outlive the app in a way the user didn't ask for. Two independent safety nets enforce this:

1. **Graceful path**: `Drop` on the platform's `PowerLease` releases the assertion (kills the `caffeinate`/`systemd-inhibit` child process, or calls `SetThreadExecutionState(ES_CONTINUOUS)` on Windows) whenever a session stops, completes, or the app exits normally.
2. **Crash path**: on macOS, `caffeinate` is started with `-w <app pid>` and no timeout, so it exits the moment the app does, however it dies, and a long system sleep can't expire it early. On Linux, `systemd-inhibit` runs a `sleep` matching the session's duration (plus a small margin), so it self-expires after a crash instead of holding the assertion forever. Windows doesn't need this: `SetThreadExecutionState`'s flags are tied to the calling thread and are cleared automatically by the OS when the process dies.

## Frontend

The React app is intentionally thin: it renders whatever `SessionSnapshot` the backend pushes, and every action (start/stop/pause/save profile/…) is a single Tauri command invocation. There's no client-side session logic to keep in sync with the backend, because the backend is the single source of truth, pushed via events (`session://update`) rather than polled.
