# Installation

## From a release (once published)

Download the installer for your platform from [GitHub Releases](../../../releases):

- **macOS**: `Awake_<version>_<arch>.dmg`. Open it and drag Awake into Applications. On first launch, macOS Gatekeeper may warn that the app is from an unidentified developer if it isn't notarized yet; right-click → Open to bypass this once.
- **Windows**: `Awake_<version>_<arch>-setup.exe` or `.msi`. Run it and follow the installer. SmartScreen may warn on first run for the same reason as Gatekeeper above.
- **Linux**: `Awake_<version>_<arch>.AppImage` (make it executable and run it) or `Awake_<version>_<arch>.deb` (`sudo dpkg -i ...`).

## From source

See [README.md § Development setup](../README.md#development-setup) and [§ Build instructions](../README.md#build-instructions). In short:

```bash
pnpm install
pnpm tauri build
```

The resulting installer/bundle is written to `src-tauri/target/release/bundle/`.

## First launch

- The app opens its setup window once, then lives in the system tray/menu bar. Closing the window does not quit it (configurable in Settings → "Close To Tray").
- If you enable **Activity Automation**, the OS will prompt for Accessibility access (macOS) the first time it's needed. You can also trigger this manually from Settings → Permissions → "Grant Accessibility Access."
- No account, sign-in, or network connection is required at any point.

## Uninstalling

- **macOS**: quit Awake (tray menu → Quit), then move it to the Trash from Applications. Configuration lives in `~/Library/Application Support/com.jawadhassan.awake/` if you want to remove it too.
- **Windows**: uninstall via _Settings → Apps_, or use the uninstaller in the install directory.
- **Linux**: remove the `.AppImage` file, or `sudo dpkg -r awake` if installed via `.deb`.
