# Privacy

Awake follows a strict local-first model. This document is the detailed version of the summary in the [README](../README.md#privacy-policy).

## What Awake does not do

- No user accounts, no sign-in, no cloud storage.
- No backend server of any kind. The app has no network client code for talking to one.
- No remote telemetry, crash reporting, or analytics.
- No transmission of configuration, activity data, or anything else off your machine, ever.

## What Awake stores, and where

Everything lives in one local JSON file (see [docs/configuration.md](configuration.md) for the exact path and schema): your settings, activity profiles, session templates, the configuration of your last session (so the setup screen can remember your usual preferences), and, if you opt in, a bounded history of past sessions. Nothing here is encrypted at rest, on the assumption that anything sensitive enough to need that shouldn't be modeled by a keep-awake utility's settings in the first place. This file contains no credentials, no personal data, and no content from your other applications.

## Activity statistics and session history (opt-in, off by default)

If you turn on "Record Local Activity Statistics" in Settings, Awake tracks, for each session: active vs. inactive duration, the number of automated activity events performed, and your longest continuous inactive streak. It distinguishes strictly between:

- **"Active vs. inactive time"** is a coarse, threshold-based bucket derived from the same idle-time polling the inactivity detector already does, never a discrete log of individual key/mouse events.
- **"What you typed"** is never recorded, never has been, isn't planned. Awake's inactivity and automation logic only ever needs to know _whether_ input happened, never its content, so there is no code path that could capture keystroke content even accidentally.

When a session ends, that session's statistics are appended to a local session history, capped at the 50 most recent sessions (older entries are dropped automatically). This history is stored in the same local config file described above, never transmitted anywhere, and only exists at all while the setting is on. You can clear it at any time from Settings with "Clear History", or turn the setting off to stop new sessions from being recorded.

## Activity automation and your screen

When Activity Automation is on, Awake simulates mouse movement, clicks, keyboard taps, or scroll gestures. It does not read your screen content, other applications' data, or clipboard. The corner-avoidance safety setting exists purely to reduce the chance of an automated click accidentally triggering something (like a hot corner), not to observe what's on screen.

## Screen-capture exclusion

This is a privacy feature _for_ you, not a data-collection mechanism: it asks the OS to exclude Awake's own window from screen recordings/shares, using the same legitimate APIs many password managers and meeting apps use. It cannot exclude, hide, or interfere with any other application's window, and the app never attempts to do so.

## Your data, your control

Since everything is local, you have full control: read the config file directly, back it up, edit it, or delete it. See [docs/configuration.md](configuration.md#resetting-configuration).
