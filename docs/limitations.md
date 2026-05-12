# Known Limitations

## Linux (experimental)

Linux support is capability-gated: each feature is detected independently at startup, and Settings → Platform Notes lists exactly what's unavailable on your system rather than failing silently.

- **Power management** requires `systemd-inhibit` (any systemd-based distro has it; non-systemd distros won't).
- **Inactivity detection** requires `xprintidle` and an X11 session. There is no standard, unprivileged way to query system-wide idle time under Wayland, so this reports unsupported there.
- **Activity automation** requires X11 or XWayland; `enigo` has no reliable native-Wayland input-injection backend across compositors.
- **Screen-capture exclusion** has no general, cross-compositor API on Linux and is always reported unsupported.

## Screen-capture exclusion (macOS/Windows)

- Works via legitimate OS APIs (`NSWindow.sharingType`, `SetWindowDisplayAffinity`), not a hack, but those APIs are opt-in for the _recording_ side too. Some third-party capture tools and older recording methods may not respect them.
- Excludes only Awake's own window. It cannot and does not attempt to affect recording of any other application.

## Activity automation

- Needs Accessibility permission on macOS. Without it, the feature is visibly disabled (toggle greyed out, explanatory banner) rather than silently doing nothing.
- On Windows, cannot send input to an elevated (Administrator) foreground window from a non-elevated Awake process. This is Windows' own User Interface Privilege Isolation, not something Awake can or should bypass.
- The "avoid screen corners" safety margin only checks the mouse cursor's _current_ position when a movement/click is about to fire; it doesn't predict where a relative move will land.

## Activity statistics and session history

- Opt-in: turning on "Record Local Activity Statistics" tracks the current session live, and appends a summary to a local session history when it ends, capped at the 50 most recent sessions (older ones are dropped automatically). See [docs/configuration.md](configuration.md#session-history) and [docs/privacy.md](privacy.md) for details.
- Deliberately cannot report a discrete count of real input events (only active/inactive time buckets). See [docs/privacy.md](privacy.md) for why.

## Session duration

- Capped at 24 hours. This is a deliberate sanity bound, not a technical limit. If you need longer, start a new session when the first one ends.

## Single window

- Awake supports exactly one main window. Multiple monitors are fully supported for cursor-position/corner-avoidance logic, but the app itself doesn't have a "pick which display" concept, and it isn't needed for a keep-awake utility.
