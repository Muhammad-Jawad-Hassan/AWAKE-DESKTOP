# Configuration

Awake stores all configuration in a single local JSON file. No database, no server, nothing that leaves your machine.

## Location

| Platform | Path                                                              |
| -------- | ----------------------------------------------------------------- |
| macOS    | `~/Library/Application Support/com.jawadhassan.awake/config.json` |
| Windows  | `%APPDATA%\com.jawadhassan.awake\config.json`                     |
| Linux    | `~/.config/com.jawadhassan.awake/config.json`                     |

## Contents

```jsonc
{
  "schema_version": 1,
  "settings": {/* AppSettings, see below */},
  "profiles": [/* built-in + custom ActivityProfile entries */],
  "last_session_config": {/* the last session you started, reused to pre-fill the setup screen */},
  "templates": [/* named, reusable SessionConfig presets you've saved */],
  "history": [/* bounded log of past sessions, opt-in, see below */],
}
```

### Settings

| Field                      | Default                         | Meaning                                                                                                      |
| -------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `startMinimized`           | `false`                         | Launch straight into the tray with no window.                                                                |
| `launchAtLogin`            | `false`                         | Register Awake to start automatically when you log in.                                                       |
| `closeToTray`              | `true`                          | Closing the window hides it instead of quitting.                                                             |
| `excludeFromScreenCapture` | `false`                         | Hide the Awake window from screen recordings (where supported).                                              |
| `notifyOnSessionEnd`       | `true`                          | Show a system notification when a session completes/stops/fails.                                             |
| `emergencyStopShortcut`    | `CommandOrControl+Shift+Escape` | Global shortcut that immediately pauses activity automation.                                                 |
| `recordActivityStatistics` | `false`                         | Opt-in local activity stats and session history; never records _what_ you typed, only _that_ input occurred. |

### Activity profiles

Each profile bundles: an inactivity threshold, a random delay range, mouse/keyboard/gesture configuration, and safety guardrails (corner-avoidance margin, max actions/minute). Three built-in profiles ship with the app (`Developer`, `Presentation`, `Testing`) and can't be modified or deleted, so clone their settings into a new custom profile instead via Settings → Activity Profiles → New Profile. Custom profiles can be exported to a `.json` file and imported back in from the same screen.

### Session templates

A template is just a named `SessionConfig` (duration, keep-awake flags, inactivity threshold, activity profile) saved from the setup screen for quick reuse later. Manage them from Settings → Session Templates or the Templates card on the setup screen.

### Session history

When `recordActivityStatistics` is on, each session's stats (active/inactive time, automated event count, longest inactive streak) are appended to `history` when the session ends, capped at the 50 most recent entries, and older ones are dropped automatically. View or clear it from Settings → Session History. See [docs/privacy.md](privacy.md#activity-statistics-and-session-history-opt-in-off-by-default) for the full privacy story.

## Editing the file directly

The app writes this file atomically (temp file + rename), so it's always either the old version or the new one, never a half-written file. You can edit it by hand while Awake is closed; a malformed file is reported as a load error rather than silently discarded, so back it up before hand-editing if you're unsure.

## Resetting configuration

Quit Awake and delete the config file (or just the fields you want reset, since missing fields fall back to defaults). Built-in profiles are always re-seeded if missing.
