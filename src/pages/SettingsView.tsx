import { useState } from "react";

import { AlertIcon } from "@/components/Icons";
import { ShortcutRecorder } from "@/components/ShortcutRecorder";
import { ToggleRow } from "@/components/ToggleRow";
import { commands } from "@/lib/commands";
import { formatActivityRates, formatDateTime, formatDuration } from "@/lib/format";
import type {
  ActivityProfile,
  AppSettings,
  HistoryEntry,
  PlatformCapabilities,
  SessionTemplate,
} from "@/lib/types";

interface SettingsViewProps {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
  profiles: ActivityProfile[];
  templates: SessionTemplate[];
  history: HistoryEntry[];
  error: string | null;
  /** Profile used by the running session; it can't be deleted. */
  activeProfileId: string | null;
  onEditProfile: (profile: ActivityProfile | null) => void;
  onDeleteProfile: (id: string) => void;
  onImportProfile: () => void;
  onExportProfile: (id: string) => void;
  onDeleteTemplate: (id: string) => void;
  onClearHistory: () => void;
  capabilities: PlatformCapabilities | null;
}

export function SettingsView({
  settings,
  onSettingsChange,
  profiles,
  templates,
  history,
  error,
  activeProfileId,
  onEditProfile,
  onDeleteProfile,
  onImportProfile,
  onExportProfile,
  onDeleteTemplate,
  onClearHistory,
  capabilities,
}: SettingsViewProps) {
  const [saveError, setSaveError] = useState<string | null>(null);

  async function update(patch: Partial<AppSettings>) {
    const next = { ...settings, ...patch };
    setSaveError(null);
    onSettingsChange(next);
    try {
      await commands.updateSettings(next);
    } catch (err) {
      setSaveError(String(err));
      // Re-sync; a later save may have landed.
      commands.getSettings().then(onSettingsChange, () => {});
    }
  }

  return (
    <div className="view">
      {(saveError ?? error) && (
        <div className="banner banner-danger" role="alert">
          <AlertIcon />
          <span>{saveError ?? error}</span>
        </div>
      )}

      <div className="card stack">
        <div className="section-title">Visibility</div>
        <ToggleRow
          label="Launch At Login"
          hint="Start Awake automatically when you sign in."
          checked={settings.launchAtLogin}
          onChange={(v) => update({ launchAtLogin: v })}
        />
        <hr className="divider" />
        <ToggleRow
          label="Start Minimized"
          hint="Launch directly into the tray, with no window."
          checked={settings.startMinimized}
          onChange={(v) => update({ startMinimized: v })}
        />
        <hr className="divider" />
        <ToggleRow
          label="Close To Tray"
          hint="Closing the window keeps Awake running."
          checked={settings.closeToTray}
          onChange={(v) => update({ closeToTray: v })}
        />
      </div>

      <div className="card stack">
        <div className="section-title">Privacy</div>
        <ToggleRow
          label="Exclude Window From Screen Capture"
          hint={
            capabilities && !capabilities.screenCaptureExclusion
              ? "Not supported on this platform."
              : "May not work with every recording method."
          }
          checked={settings.excludeFromScreenCapture}
          disabled={capabilities !== null && !capabilities.screenCaptureExclusion}
          onChange={(v) => update({ excludeFromScreenCapture: v })}
        />
        <hr className="divider" />
        <ToggleRow
          label="Record Local Activity Statistics"
          hint="Never records what you typed, only that input occurred."
          checked={settings.recordActivityStatistics}
          onChange={(v) => update({ recordActivityStatistics: v })}
        />
      </div>

      <div className="card stack">
        <div className="section-title">Notifications &amp; Safety</div>
        <ToggleRow
          label="Notify When A Session Ends"
          checked={settings.notifyOnSessionEnd}
          onChange={(v) => update({ notifyOnSessionEnd: v })}
        />
        <hr className="divider" />
        <div>
          <div className="field-label" style={{ marginBottom: 6 }}>
            Emergency Stop Shortcut
          </div>
          <ShortcutRecorder
            value={settings.emergencyStopShortcut}
            onChange={(v) => update({ emergencyStopShortcut: v })}
          />
          <div className="field-hint">Immediately disables automated input from anywhere.</div>
        </div>
      </div>

      {capabilities && !capabilities.inputPermissionGranted && (
        <div className="card stack">
          <div className="section-title">Permissions</div>
          <div className="banner banner-warning">
            <AlertIcon />
            <span>
              Activity automation needs Accessibility access to control the mouse and keyboard.
            </span>
          </div>
          <button
            className="btn btn-secondary"
            onClick={() => commands.requestPermissions().catch((err) => setSaveError(String(err)))}
          >
            Grant Accessibility Access
          </button>
        </div>
      )}

      <div className="card stack">
        <div className="row">
          <div className="section-title" style={{ marginBottom: 0 }}>
            Activity Profiles
          </div>
          <div className="list-row-actions">
            <button className="btn btn-secondary btn-sm" onClick={onImportProfile}>
              Import
            </button>
            <button className="btn btn-secondary btn-sm" onClick={() => onEditProfile(null)}>
              New Profile
            </button>
          </div>
        </div>
        {profiles.map((profile, i) => (
          <div key={profile.id}>
            {i > 0 && <hr className="divider" />}
            <div className="list-row">
              <div>
                <div className="field-label">{profile.name}</div>
                <div className="field-hint">
                  {profile.builtIn ? "Built-in" : "Custom"} · {formatActivityRates(profile)}
                </div>
              </div>
              <div className="list-row-actions">
                <button
                  className="btn btn-secondary btn-sm"
                  onClick={() => onExportProfile(profile.id)}
                >
                  Export
                </button>
                <button className="btn btn-secondary btn-sm" onClick={() => onEditProfile(profile)}>
                  {profile.builtIn ? "View" : "Edit"}
                </button>
                {!profile.builtIn && (
                  <button
                    className="btn btn-danger btn-sm"
                    disabled={profile.id === activeProfileId}
                    title={
                      profile.id === activeProfileId ? "In use by the running session" : undefined
                    }
                    onClick={() => onDeleteProfile(profile.id)}
                  >
                    Delete
                  </button>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>

      {templates.length > 0 && (
        <div className="card stack">
          <div className="section-title">Session Templates</div>
          {templates.map((template, i) => (
            <div key={template.id}>
              {i > 0 && <hr className="divider" />}
              <div className="list-row">
                <div className="field-label">{template.name}</div>
                <button
                  className="btn btn-danger btn-sm"
                  onClick={() => onDeleteTemplate(template.id)}
                >
                  Delete
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {history.length > 0 && (
        <div className="card stack">
          <div className="row">
            <div className="section-title" style={{ marginBottom: 0 }}>
              Session History
            </div>
            <button className="btn btn-secondary btn-sm" onClick={onClearHistory}>
              Clear History
            </button>
          </div>
          {[...history].reverse().map((entry, i) => (
            <div key={entry.id}>
              {i > 0 && <hr className="divider" />}
              <div className="list-row">
                <div>
                  <div className="field-label">{formatDateTime(entry.endedAtUnixSecs)}</div>
                  <div className="field-hint">
                    {formatDuration(entry.durationSecs)} · {formatDuration(entry.stats.activeSecs)}{" "}
                    Active · {formatDuration(entry.stats.inactiveSecs)} Inactive ·{" "}
                    {entry.stats.automatedEventCount} Automated Events
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {capabilities && capabilities.notes.length > 0 && (
        <div className="card stack">
          <div className="section-title">Platform Notes</div>
          {capabilities.notes.map((note, i) => (
            <div key={i} className="field-hint">
              {note}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
