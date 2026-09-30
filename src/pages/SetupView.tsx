import { useEffect, useMemo, useRef, useState } from "react";

import { AlertIcon, PowerIcon } from "@/components/Icons";
import { Stepper } from "@/components/Stepper";
import { ToggleRow } from "@/components/ToggleRow";
import { commands } from "@/lib/commands";
import { INACTIVITY_THRESHOLD_OPTIONS } from "@/lib/constants";
import { formatActivityRates, formatDuration } from "@/lib/format";
import type {
  ActivityProfile,
  Limits,
  PlatformCapabilities,
  SessionSnapshot,
  SessionTemplate,
} from "@/lib/types";

const PRESETS = [
  { label: "30m", hours: 0, minutes: 30 },
  { label: "1h", hours: 1, minutes: 0 },
  { label: "4h", hours: 4, minutes: 0 },
  { label: "8h", hours: 8, minutes: 0 },
];

const ENDED_STATES = new Set(["completed", "stopped", "failed"]);

interface SetupViewProps {
  profiles: ActivityProfile[];
  templates: SessionTemplate[];
  capabilities: PlatformCapabilities | null;
  limits: Limits;
  snapshot: SessionSnapshot;
  endedBannerDismissed: boolean;
  onDismissEndedBanner: () => void;
  onTemplateSaved: () => void;
}

export function SetupView({
  profiles,
  templates,
  capabilities,
  limits,
  snapshot,
  endedBannerDismissed,
  onDismissEndedBanner,
  onTemplateSaved,
}: SetupViewProps) {
  const [hours, setHours] = useState(1);
  const [minutes, setMinutes] = useState(0);
  const [keepSystemAwake, setKeepSystemAwake] = useState(true);
  const [keepDisplayAwake, setKeepDisplayAwake] = useState(false);
  const [activityEnabled, setActivityEnabled] = useState(false);
  const [profileId, setProfileId] = useState("");
  const [inactivityThreshold, setInactivityThreshold] = useState(300);
  const [starting, setStarting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [savingTemplate, setSavingTemplate] = useState(false);
  const [newTemplateName, setNewTemplateName] = useState("");

  const initialized = useRef(false);
  useEffect(() => {
    if (initialized.current || profiles.length === 0) return;
    initialized.current = true;

    const first = profiles[0]!;
    commands
      .getLastSessionConfig()
      .then((last) => {
        if (!last) {
          setProfileId(first.id);
          setInactivityThreshold(first.inactivityThreshold);
          return;
        }
        setHours(Math.floor(last.duration / 3600));
        setMinutes(Math.floor((last.duration % 3600) / 60));
        setKeepSystemAwake(last.keepSystemAwake);
        setKeepDisplayAwake(last.keepDisplayAwake);
        setInactivityThreshold(last.inactivityThreshold);
        const lastProfileExists = profiles.some((p) => p.id === last.activityProfileId);
        setActivityEnabled(lastProfileExists);
        setProfileId(lastProfileExists ? last.activityProfileId! : first.id);
      })
      .catch((err) => setError(`Couldn't restore your last session settings: ${String(err)}`));
  }, [profiles]);

  const maxHours = Math.floor(limits.maxSessionSecs / 3600);
  const fits = (h: number, m: number) => h * 3600 + m * 60 <= limits.maxSessionSecs;

  function incrementMinutes() {
    if (minutes < 59 && fits(hours, minutes + 1)) {
      setMinutes(minutes + 1);
    } else if (minutes === 59 && fits(hours + 1, 0)) {
      setMinutes(0);
      setHours(hours + 1);
    }
  }

  function decrementMinutes() {
    if (minutes > 0) {
      setMinutes(minutes - 1);
    } else if (hours > 0) {
      setMinutes(59);
      setHours(hours - 1);
    }
  }

  const durationSecs = hours * 3600 + minutes * 60;
  const selectedProfile = useMemo(
    () => profiles.find((p) => p.id === profileId),
    [profiles, profileId],
  );
  const inputUnsupported =
    capabilities !== null && (!capabilities.inputSimulation || !capabilities.idleDetection);
  const activityOn = activityEnabled && !inputUnsupported;

  function selectProfile(id: string) {
    setProfileId(id);
    const profile = profiles.find((p) => p.id === id);
    if (profile) setInactivityThreshold(profile.inactivityThreshold);
  }

  function applyTemplate(template: SessionTemplate) {
    const { config } = template;
    setHours(Math.floor(config.duration / 3600));
    setMinutes(Math.floor((config.duration % 3600) / 60));
    setKeepSystemAwake(config.keepSystemAwake);
    setKeepDisplayAwake(config.keepDisplayAwake);
    setInactivityThreshold(config.inactivityThreshold);
    setActivityEnabled(config.activityProfileId !== null);
    setError(null);
    if (config.activityProfileId) {
      if (profiles.some((p) => p.id === config.activityProfileId)) {
        setProfileId(config.activityProfileId);
      } else {
        setActivityEnabled(false);
        setError(
          `The activity profile saved in "${template.name}" no longer exists, so activity automation is off.`,
        );
      }
    }
  }

  async function handleSaveTemplate() {
    const name = newTemplateName.trim();
    if (!name) return;
    setError(null);
    try {
      await commands.saveTemplate({
        id: `template-${crypto.randomUUID()}`,
        name,
        config: {
          duration: durationSecs,
          keepSystemAwake,
          keepDisplayAwake,
          inactivityThreshold,
          activityProfileId: activityOn ? profileId || null : null,
        },
      });
      setNewTemplateName("");
      setSavingTemplate(false);
      onTemplateSaved();
    } catch (err) {
      setError(String(err));
    }
  }

  function handleCancelTemplate() {
    setNewTemplateName("");
    setSavingTemplate(false);
  }

  async function startSession(config: Parameters<typeof commands.startSession>[0]) {
    setError(null);
    setStarting(true);
    try {
      await commands.startSession(config);
    } catch (err) {
      setError(String(err));
    } finally {
      setStarting(false);
    }
  }

  function handleStart() {
    return startSession({
      duration: durationSecs,
      keepSystemAwake,
      keepDisplayAwake,
      inactivityThreshold,
      activityProfileId: activityOn ? profileId || null : null,
    });
  }

  function handleStartAgain() {
    onDismissEndedBanner();
    return startSession({
      duration: snapshot.durationSecs,
      keepSystemAwake: snapshot.keepSystemAwake,
      keepDisplayAwake: snapshot.keepDisplayAwake,
      inactivityThreshold: snapshot.inactivityThresholdSecs,
      activityProfileId: snapshot.activityProfileId,
    });
  }

  const canStart = (keepSystemAwake || keepDisplayAwake || activityOn) && durationSecs > 0;

  const showEndedBanner = ENDED_STATES.has(snapshot.state) && !endedBannerDismissed;

  return (
    <div className="view">
      {showEndedBanner && (
        <div className="card stack">
          <div className="list-row">
            <div>
              <div className="field-label">
                Session {snapshot.state === "completed" ? "Finished" : "Ended"}
              </div>
              <div className="field-hint">Ran for {formatDuration(snapshot.elapsedSecs)}.</div>
            </div>
            <div className="list-row-actions">
              <button className="btn btn-secondary btn-sm" onClick={onDismissEndedBanner}>
                Dismiss
              </button>
              <button
                className="btn btn-primary btn-sm"
                disabled={starting}
                onClick={handleStartAgain}
              >
                Start Again
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="card stack">
        <div className="section-title">Duration</div>
        <div className="duration-presets">
          {PRESETS.map((preset) => (
            <button
              key={preset.label}
              className={`chip ${hours === preset.hours && minutes === preset.minutes ? "chip-selected" : ""}`}
              onClick={() => {
                setHours(preset.hours);
                setMinutes(preset.minutes);
              }}
            >
              {preset.label}
            </button>
          ))}
        </div>
        <div className="time-panel">
          <div className="time-panel-unit">
            <div className="time-panel-unit-label">Hours</div>
            <Stepper
              value={hours}
              onChange={setHours}
              min={0}
              max={maxHours}
              label="hours"
              canIncrement={fits(hours + 1, minutes)}
            />
          </div>
          <div className="time-panel-sep">:</div>
          <div className="time-panel-unit">
            <div className="time-panel-unit-label">Minutes</div>
            <Stepper
              value={minutes}
              onChange={setMinutes}
              onIncrement={incrementMinutes}
              onDecrement={decrementMinutes}
              label="minutes"
              canIncrement={fits(hours, minutes + 1) || (minutes === 59 && fits(hours + 1, 0))}
              canDecrement={minutes > 0 || hours > 0}
            />
          </div>
        </div>
      </div>

      <div className="card stack">
        <ToggleRow
          label="Keep System Awake"
          hint="Prevents idle sleep for the session."
          checked={keepSystemAwake}
          onChange={setKeepSystemAwake}
        />
        <hr className="divider" />
        <ToggleRow
          label="Keep Display Awake"
          hint="Also prevents the screen from turning off."
          checked={keepDisplayAwake}
          onChange={setKeepDisplayAwake}
        />
      </div>

      <div className="card stack">
        <ToggleRow
          label="Activity Automation"
          hint="Simulate input after a period of inactivity."
          checked={activityEnabled}
          onChange={setActivityEnabled}
          disabled={inputUnsupported}
        />
        {activityOn && capabilities && !capabilities.inputPermissionGranted && (
          <div className="banner banner-warning">
            <AlertIcon />
            <span>Grant Accessibility access in Settings before starting.</span>
          </div>
        )}

        {inputUnsupported && (
          <div className="banner banner-warning">
            <AlertIcon />
            <span>
              Activity automation isn&apos;t available on this system. See Platform Notes in
              Settings.
            </span>
          </div>
        )}

        {activityOn && (
          <>
            <hr className="divider" />
            <div>
              <label className="field-label" htmlFor="setup-profile" style={{ marginBottom: 6 }}>
                Profile
              </label>
              <select
                id="setup-profile"
                className="input"
                value={profileId}
                onChange={(e) => selectProfile(e.target.value)}
              >
                {profiles.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                  </option>
                ))}
              </select>
              {selectedProfile && (
                <div className="field-hint">{formatActivityRates(selectedProfile)}</div>
              )}
            </div>
            <div>
              <label className="field-label" htmlFor="setup-threshold" style={{ marginBottom: 6 }}>
                Inactivity Threshold
              </label>
              <select
                id="setup-threshold"
                className="input"
                value={inactivityThreshold}
                onChange={(e) => setInactivityThreshold(Number(e.target.value))}
              >
                {INACTIVITY_THRESHOLD_OPTIONS.map((secs) => (
                  <option key={secs} value={secs}>
                    {formatDuration(secs)}
                  </option>
                ))}
              </select>
            </div>
          </>
        )}
      </div>

      <div className="card stack">
        <div className="row">
          <div className="section-title" style={{ marginBottom: 0 }}>
            Templates
          </div>
          {!savingTemplate && (
            <button className="btn btn-secondary btn-sm" onClick={() => setSavingTemplate(true)}>
              Save Current
            </button>
          )}
        </div>
        {templates.length === 0 && !savingTemplate && (
          <div className="field-hint">No saved templates yet.</div>
        )}
        {templates.map((template, i) => (
          <div key={template.id}>
            {i > 0 && <hr className="divider" />}
            <div className="list-row">
              <div className="field-label">{template.name}</div>
              <button className="btn btn-secondary btn-sm" onClick={() => applyTemplate(template)}>
                Load
              </button>
            </div>
          </div>
        ))}
        {savingTemplate && (
          <>
            {templates.length > 0 && <hr className="divider" />}
            <div style={{ display: "flex", gap: 6 }}>
              <input
                className="input"
                placeholder="Template name"
                aria-label="Template name"
                value={newTemplateName}
                autoFocus
                onChange={(e) => setNewTemplateName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleSaveTemplate();
                  if (e.key === "Escape") handleCancelTemplate();
                }}
              />
              <button
                className="btn btn-secondary btn-sm"
                disabled={!newTemplateName.trim()}
                onClick={handleSaveTemplate}
              >
                Save
              </button>
              <button className="btn btn-secondary btn-sm" onClick={handleCancelTemplate}>
                Cancel
              </button>
            </div>
          </>
        )}
      </div>

      {error && (
        <div className="banner banner-danger">
          <AlertIcon />
          <span>{error}</span>
        </div>
      )}

      <button className="btn btn-primary" disabled={!canStart || starting} onClick={handleStart}>
        <PowerIcon />
        {starting ? "Starting…" : "Start Session"}
      </button>
    </div>
  );
}
