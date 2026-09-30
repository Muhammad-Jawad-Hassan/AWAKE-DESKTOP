import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import { listen } from "@tauri-apps/api/event";

import { AlertIcon } from "@/components/Icons";
import { Stepper } from "@/components/Stepper";
import { Toggle } from "@/components/Toggle";
import { commands } from "@/lib/commands";
import { INACTIVITY_THRESHOLD_OPTIONS } from "@/lib/constants";
import { formatActivityKind, formatDuration } from "@/lib/format";
import { enabledRateTotal, fitUnderCap, maxRateFor, type RatedActivity } from "@/lib/profile";
import type { ActivityProfile, GestureConfig, Limits, TestActivityProgress } from "@/lib/types";

type ScrollDirection = "vertical" | "horizontal" | "both";

const SCROLL_DIRECTIONS: { value: ScrollDirection; label: string }[] = [
  { value: "vertical", label: "Vertical" },
  { value: "horizontal", label: "Horizontal" },
  { value: "both", label: "Both" },
];

function scrollDirection({ horizontal, vertical }: GestureConfig): ScrollDirection {
  if (horizontal && vertical) return "both";
  return horizontal ? "horizontal" : "vertical";
}

interface SettingRowProps {
  label: string;
  htmlFor?: string;
  children: ReactNode;
}

function SettingRow({ label, htmlFor, children }: SettingRowProps) {
  return (
    <div className="row">
      {htmlFor ? (
        <label className="setting-label" htmlFor={htmlFor}>
          {label}
        </label>
      ) : (
        <span className="setting-label">{label}</span>
      )}
      {children}
    </div>
  );
}

let lastTestRun = 0;

interface ActivitySectionProps {
  title: string;
  description: string;
  enabled: boolean;
  readOnly: boolean;
  onToggle: (enabled: boolean) => void;
  children: ReactNode;
}

function ActivitySection({
  title,
  description,
  enabled,
  readOnly,
  onToggle,
  children,
}: ActivitySectionProps) {
  return (
    <div className="stack">
      <div className="row">
        <div>
          <div className="field-label">{title}</div>
          <div className="field-hint">{description}</div>
        </div>
        <Toggle label={title} checked={enabled} disabled={readOnly} onChange={onToggle} />
      </div>
      {enabled && children}
    </div>
  );
}

interface ProfileEditorViewProps {
  profile: ActivityProfile;
  limits: Limits;
  onDone: () => void;
}

export function ProfileEditorView({ profile, limits, onDone }: ProfileEditorViewProps) {
  const [draft, setDraft] = useState(profile);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);
  const testRunRef = useRef<number | null>(null);
  const mountedRef = useRef(true);
  const ids = { name: useId(), threshold: useId(), key: useId() };
  const readOnly = draft.builtIn;
  const rateTotal = enabledRateTotal(draft);
  const cap = draft.safety.maxActionsPerMinute;
  const scrolling = draft.gestures.horizontal || draft.gestures.vertical;
  const overCap = rateTotal > cap;
  const thresholdOptions = INACTIVITY_THRESHOLD_OPTIONS.includes(draft.inactivityThreshold)
    ? INACTIVITY_THRESHOLD_OPTIONS
    : [...INACTIVITY_THRESHOLD_OPTIONS, draft.inactivityThreshold].sort((a, b) => a - b);

  // Stop a running test on exit.
  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
      if (testRunRef.current !== null) void commands.cancelTestActivity(testRunRef.current);
    };
  }, []);

  function setActivityEnabled(
    activity: RatedActivity,
    update: (d: ActivityProfile) => ActivityProfile,
  ) {
    setDraft((d) => fitUnderCap(update(d), activity, limits));
  }

  function rateStepper(activity: RatedActivity, label: string) {
    const value = draft[activity].perMinute;
    const max = maxRateFor(draft, activity, limits.maxPerMinute);
    return (
      <Stepper
        inline
        value={value}
        onChange={(perMinute) =>
          setDraft((d) => ({ ...d, [activity]: { ...d[activity], perMinute } }))
        }
        min={1}
        max={max}
        suffix="/min"
        label={label}
        canIncrement={!readOnly && value < max}
        canDecrement={!readOnly && value > 1}
      />
    );
  }

  function setScrollDirection(direction: ScrollDirection) {
    setDraft((d) => ({
      ...d,
      gestures: {
        ...d.gestures,
        horizontal: direction !== "vertical",
        vertical: direction !== "horizontal",
      },
    }));
  }

  async function handleSave() {
    setError(null);
    setSaving(true);
    try {
      await commands.saveProfile(draft);
      onDone();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  }

  async function handleTestActivity() {
    setError(null);
    setTestResult(null);
    setTesting(true);
    const runId = ++lastTestRun;
    testRunRef.current = runId;
    let unlisten: (() => void) | undefined;
    try {
      unlisten = await listen<TestActivityProgress>("test-activity://progress", (event) => {
        const { kind, index, total } = event.payload;
        setTestResult(`Testing ${formatActivityKind(kind)}… (${index + 1}/${total})`);
      });
      if (!mountedRef.current) return;
      const results = await commands.testActivity(draft, runId);
      const performed = results.filter((r) => r.performed).map((r) => r.kind);
      const skipped = results.filter((r) => !r.performed).map((r) => r.kind);
      let message =
        performed.length > 0 ? `Fired: ${performed.map(formatActivityKind).join(", ")}` : "";
      if (skipped.length > 0) {
        const skippedText = `skipped near screen corner: ${skipped.map(formatActivityKind).join(", ")}`;
        message = message ? `${message} (${skippedText})` : skippedText;
      }
      setTestResult(message || "Stopped.");
    } catch (err) {
      setError(String(err));
    } finally {
      testRunRef.current = null;
      unlisten?.();
      if (mountedRef.current) setTesting(false);
    }
  }

  return (
    <div className="view">
      {readOnly && (
        <div className="banner">
          <AlertIcon />
          <span>Built-in profiles can&apos;t be changed. Create a new profile to customize.</span>
        </div>
      )}

      <div className="card stack">
        <div className="section-title">Profile</div>
        <div>
          <label
            className="setting-label"
            htmlFor={ids.name}
            style={{ display: "block", marginBottom: 6 }}
          >
            Name
          </label>
          <input
            id={ids.name}
            className="input"
            value={draft.name}
            disabled={readOnly}
            onChange={(e) => setDraft((d) => ({ ...d, name: e.target.value }))}
          />
        </div>
        <SettingRow label="Start after idle for" htmlFor={ids.threshold}>
          <select
            id={ids.threshold}
            className="input input-inline"
            value={draft.inactivityThreshold}
            disabled={readOnly}
            onChange={(e) =>
              setDraft((d) => ({ ...d, inactivityThreshold: Number(e.target.value) }))
            }
          >
            {thresholdOptions.map((secs) => (
              <option key={secs} value={secs}>
                {formatDuration(secs)}
              </option>
            ))}
          </select>
        </SettingRow>
      </div>

      <div className="card stack">
        <div className="section-title">Activities</div>

        <ActivitySection
          title="Mouse Movement"
          description="A small jiggle that returns the cursor to where it was."
          enabled={draft.mouse.enabled}
          readOnly={readOnly}
          onToggle={(enabled) =>
            setActivityEnabled("mouse", (d) => ({ ...d, mouse: { ...d.mouse, enabled } }))
          }
        >
          <SettingRow label="Frequency">
            {rateStepper("mouse", "mouse moves per minute")}
          </SettingRow>
        </ActivitySection>

        <hr className="divider" />

        <ActivitySection
          title="Key Taps"
          description="Taps a key that does nothing on its own, safe in any app."
          enabled={draft.keyboard.enabled}
          readOnly={readOnly}
          onToggle={(enabled) =>
            setActivityEnabled("keyboard", (d) => ({ ...d, keyboard: { ...d.keyboard, enabled } }))
          }
        >
          <SettingRow label="Key" htmlFor={ids.key}>
            <select
              id={ids.key}
              className="input input-inline"
              value={draft.keyboard.key}
              disabled={readOnly}
              onChange={(e) =>
                setDraft((d) => ({ ...d, keyboard: { ...d.keyboard, key: e.target.value } }))
              }
            >
              {limits.safeKeys.map((key) => (
                <option key={key} value={key}>
                  {key}
                </option>
              ))}
              <option value={limits.randomKey}>Random</option>
            </select>
          </SettingRow>
          <SettingRow label="Frequency">
            {rateStepper("keyboard", "key taps per minute")}
          </SettingRow>
        </ActivitySection>

        <hr className="divider" />

        <ActivitySection
          title="Scrolling"
          description="Scrolls a few notches in the window under the cursor."
          enabled={scrolling}
          readOnly={readOnly}
          onToggle={(enabled) =>
            setActivityEnabled("gestures", (d) => ({
              ...d,
              gestures: { ...d.gestures, vertical: enabled, horizontal: false },
            }))
          }
        >
          <SettingRow label="Direction">
            <div className="segmented" role="group" aria-label="Scroll direction">
              {SCROLL_DIRECTIONS.map(({ value, label }) => (
                <button
                  key={value}
                  type="button"
                  aria-pressed={scrollDirection(draft.gestures) === value}
                  disabled={readOnly}
                  onClick={() => setScrollDirection(value)}
                >
                  {label}
                </button>
              ))}
            </div>
          </SettingRow>
          <SettingRow label="Frequency">{rateStepper("gestures", "scrolls per minute")}</SettingRow>
        </ActivitySection>
      </div>

      <div className="card stack">
        <div className="section-title">Rate Limit</div>
        <SettingRow label="Max actions per minute">
          <Stepper
            inline
            value={cap}
            onChange={(v) =>
              setDraft((d) => ({ ...d, safety: { ...d.safety, maxActionsPerMinute: v } }))
            }
            min={Math.max(1, rateTotal)}
            max={limits.maxSafetyCap}
            suffix="/min"
            label="max actions per minute"
            canIncrement={!readOnly && cap < limits.maxSafetyCap}
            canDecrement={!readOnly && cap > Math.max(1, rateTotal)}
          />
        </SettingRow>
        <div>
          <div className="mini-progress-track" aria-hidden="true">
            <div
              className="mini-progress-fill"
              style={{ width: `${Math.min(100, (rateTotal / cap) * 100)}%` }}
            />
          </div>
          <div className="field-hint">
            {rateTotal} of {cap} used by enabled activities.
            {cap < limits.maxSafetyCap && " Raise the limit to allow more."}
          </div>
        </div>
      </div>

      <div className="card stack">
        <div className="section-title">Preview</div>
        <div className="row">
          <div className="field-hint" style={{ marginTop: 0 }} aria-live="polite">
            {testResult ?? "Run each enabled activity once, a few seconds apart."}
          </div>
          <button
            className="btn btn-secondary btn-sm"
            disabled={testing}
            onClick={handleTestActivity}
          >
            {testing ? "Testing…" : "Test Now"}
          </button>
        </div>
      </div>

      {error && (
        <div className="banner banner-danger" role="alert">
          <AlertIcon />
          <span>{error}</span>
        </div>
      )}

      <div className="stack">
        {!readOnly && (
          <button
            className="btn btn-primary btn-compact"
            disabled={saving || overCap}
            title={overCap ? "Lower a frequency or raise the rate limit first" : undefined}
            onClick={handleSave}
          >
            {saving ? "Saving…" : "Save Profile"}
          </button>
        )}
        <button
          className={`btn ${readOnly ? "btn-secondary" : "btn-link"}`}
          style={readOnly ? undefined : { alignSelf: "center" }}
          onClick={onDone}
        >
          {readOnly ? "Close" : "Cancel"}
        </button>
      </div>
    </div>
  );
}
