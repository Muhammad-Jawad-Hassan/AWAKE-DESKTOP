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
│   ├── activity.rs             Randomized delay/selection, pure decision functions
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

`InactivityMonitor` (`core/inactivity.rs`) solves this with a short grace window: right after performing a synthetic action, it ignores the next OS reading and keeps counting up as if nothing happened. If a _second_ consecutive reading (outside the grace window) still shows near-zero idle time, that's real user input, and the monitor correctly reports "active." This is unit-tested directly (`core/inactivity.rs` tests) rather than relying on integration testing, since it's the trickiest piece of logic in the app.

## Runtime orchestration

`runtime::AppState` holds one `tokio::sync::Mutex<Inner>` guarding all session-related state. `start_session` spawns a single background task (`run_session_loop`) that ticks once per second: advances the session timer, polls OS idle time, decides whether to fire a scheduled activity, and pushes a `SessionSnapshot` to both the main window (`session://update` event) and the tray. Every command handler that mutates state (`stop_session`, `pause_activity`, …) also pushes a snapshot immediately, rather than waiting for the next tick. The loop's periodic emit and each command's immediate emit both funnel through the same `runtime::broadcast_snapshot` helper to avoid the two drifting out of sync.

## Power-management safety

A held "prevent sleep" assertion must never outlive the app in a way the user didn't ask for. Two independent safety nets enforce this:

1. **Graceful path**: `Drop` on the platform's `PowerLease` releases the assertion (kills the `caffeinate`/`systemd-inhibit` child process, or calls `SetThreadExecutionState(ES_CONTINUOUS)` on Windows) whenever a session stops, completes, or the app exits normally.
2. **Crash path**: on macOS and Linux, the helper process is spawned with a `-t <duration>` timeout matching the session's own duration (plus a small margin). If the app is killed ungracefully (e.g. `SIGKILL`, or `[NSApp terminate:]` on macOS, which does not unwind Rust's stack), the helper process self-expires instead of holding the assertion forever. Windows doesn't need this: `SetThreadExecutionState`'s flags are tied to the calling thread and are cleared automatically by the OS when the process dies.

## Frontend

The React app is intentionally thin: it renders whatever `SessionSnapshot` the backend pushes, and every action (start/stop/pause/save profile/…) is a single Tauri command invocation. There's no client-side session logic to keep in sync with the backend, because the backend is the single source of truth, pushed via events (`session://update`) rather than polled.
