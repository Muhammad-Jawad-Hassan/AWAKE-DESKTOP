# Permissions

Awake asks for the minimum permission each active feature needs, and only when that feature is turned on. A plain "keep the system awake" session with activity automation off requires **no special permissions on any platform**.

## macOS

| Feature                                       | Permission        | Notes                                                                                                                                                       |
| --------------------------------------------- | ----------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Keep system/display awake                     | None              | Uses `/usr/bin/caffeinate`, a standard system utility.                                                                                                      |
| Inactivity detection                          | None              | `CGEventSourceSecondsSinceLastEventType` is a read-only query; no Accessibility grant needed.                                                               |
| Activity automation (mouse/keyboard/gestures) | **Accessibility** | Required by macOS for any app that synthesizes input events. Prompted automatically the first time automation runs, or manually via Settings → Permissions. |
| Screen-capture exclusion                      | None              | `NSWindow.sharingType` is a normal window property.                                                                                                         |

## Windows

| Feature                   | Permission       | Notes                                                                                                                                                                                                                                                         |
| ------------------------- | ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Keep system/display awake | None             | `SetThreadExecutionState` is available to any process.                                                                                                                                                                                                        |
| Inactivity detection      | None             | `GetLastInputInfo` is a read-only query.                                                                                                                                                                                                                      |
| Activity automation       | None (typically) | Uses `SendInput`-equivalent APIs available to standard user processes. If the _foreground_ window is running elevated (as Administrator), a non-elevated Awake cannot send input to it. This is standard Windows UI Access Control, not an Awake limitation. |
| Screen-capture exclusion  | None             | `SetWindowDisplayAffinity` is a normal window property.                                                                                                                                                                                                       |

## Linux

| Feature                   | Permission                           | Notes                                                                                                                |
| ------------------------- | ------------------------------------ | -------------------------------------------------------------------------------------------------------------------- |
| Keep system/display awake | None                                 | Uses `systemd-inhibit`, available on any systemd-based distro. Reports unsupported if `systemd-inhibit` isn't found. |
| Inactivity detection      | None, but **X11 only**               | Uses `xprintidle`. No standard, unprivileged idle-time API exists under Wayland; reports unsupported there.          |
| Activity automation       | None, but **X11 (or XWayland) only** | Uses `enigo`, which has no reliable Wayland-native backend.                                                          |
| Screen-capture exclusion  | Not supported                        | No general cross-compositor API exists.                                                                              |

## Why activity automation needs Accessibility (and what it can't do)

macOS treats synthetic input events as a sensitive capability specifically to prevent malware from silently controlling the user's computer. Awake requesting Accessibility only when you enable Activity Automation, never on first launch and never for the core keep-awake feature, is a deliberate design choice, not a workaround. Granting it does **not** give Awake any additional access beyond simulating mouse/keyboard/scroll events; it cannot read your screen, your keystrokes, or any other app's data.

## Revoking permissions

You can revoke Accessibility access at any time in System Settings → Privacy & Security → Accessibility (macOS). Awake detects this and reports Activity Automation as unavailable rather than failing silently or crashing.
