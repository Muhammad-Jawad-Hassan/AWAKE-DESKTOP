# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project intends to adhere to [Semantic Versioning](https://semver.org/) once it reaches 1.0.0.

## [Unreleased]

### Changed

- Activity automation no longer simulates mouse clicks, and keyboard taps are limited to Shift and Control (Control by default). Saved profiles with any other key fall back to Control.
- Profiles set a per-minute rate for mouse movement, key taps, and scrolling separately, replacing the shared random delay range. Existing profiles default to 1 per minute each. The combined safety cap is now editable.

### Fixed

- Automated input no longer makes Awake think the user came back a couple of seconds later, which held activity to about one action per inactivity threshold regardless of the configured rates.
- Mouse movement returns the cursor to where it started, and skips moves whose far end would reach a screen corner.
- Quitting mid-session releases the sleep assertion and records history. On macOS `caffeinate` now exits with the app even after a crash.
- An unreadable config file is moved aside and reported instead of being overwritten with defaults.
- Built-in profiles now track the installed version instead of being frozen in the config file. Older custom profiles are repaired on load and on import.
- An invalid emergency-stop shortcut is rejected and the previous one kept, instead of silently leaving no shortcut. The recorder no longer captures plain typing like Shift+A, and keeps Ctrl and Cmd distinct.
- The emergency stop now also cancels a running Test Activity and any mouse path in progress.
- On macOS, granting Accessibility access takes effect without restarting the app.
- Skipped actions no longer count as performed, activity and idle-detection failures are shown during a session, and a failed idle reading no longer restarts the inactivity wait.
- Session history records how long a session actually ran, and elapsed time stops counting once a session ends.
- Deleting the profile a session is using is refused, and starting a session with a missing or invalid profile reports why.
- Real typing right after an automated action is no longer mistaken for Awake's own input, so automation stops when you come back, even at high rates.
- Stop, Pause and the emergency stop cut off input that was already planned, and a cancelled Test Activity can't start late or run twice.
- Quitting with Cmd+Q, or logging off, now finishes the session and records its history.
- A failed idle reading stops automation instead of keeping it running.
- On macOS, a long system sleep no longer ends a session early.
- The default emergency shortcut on Windows and Linux is Ctrl+Alt+Shift+Esc, since Ctrl+Shift+Esc opens Task Manager.
- In the profile editor, turning an activity on can't push the total past the rate limit, and Setup keeps its values after a trip to Settings.

### Added

- A "Random" key option that taps a different safe key each time.
- Initial release: session engine, OS-level power management (macOS/Windows/Linux), inactivity detection, optional activity automation with randomized timing, tray/menu-bar UI, activity profiles, screen-capture exclusion, emergency stop (button, tray, global shortcut), and local-only configuration.
