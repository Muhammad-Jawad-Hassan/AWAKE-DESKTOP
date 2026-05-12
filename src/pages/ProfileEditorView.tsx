import { useState } from "react";
import { listen } from "@tauri-apps/api/event";

import { AlertIcon } from "@/components/Icons";
import { KeyRecorder } from "@/components/KeyRecorder";
import { Toggle } from "@/components/Toggle";
import { commands } from "@/lib/commands";
import { INACTIVITY_THRESHOLD_OPTIONS } from "@/lib/constants";
import { formatActivityKind, formatDurationShort } from "@/lib/format";
import type { ActivityProfile, TestActivityProgress } from "@/lib/types";

function blankProfile(): ActivityProfile {
  return {
    id: `custom-${crypto.randomUUID()}`,
    name: "New profile",
    inactivityThreshold: 300,
    minDelay: 30,
    maxDelay: 180,
    mouse: { enabled: true, movement: true, button: "left", click: "single", randomize: true },
    keyboard: { enabled: false, key: "Shift", modifiers: [], randomize: true },
    gestures: { horizontal: false, vertical: false, custom: null },
    safety: { avoidScreenCornersPx: 24, maxActionsPerMinute: 6 },
    builtIn: false,
  };
}

interface ProfileEditorViewProps {
  profile: ActivityProfile | null;
  onDone: () => void;
}

export function ProfileEditorView({ profile, onDone }: ProfileEditorViewProps) {
  const [draft, setDraft] = useState<ActivityProfile>(profile ?? blankProfile());
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);
  const readOnly = draft.builtIn;

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
    const unlisten = await listen<TestActivityProgress>("test-activity://progress", (event) => {
      const { kind, index, total } = event.payload;
      setTestResult(`Testing ${formatActivityKind(kind)}… (${index + 1}/${total})`);
    });
    try {
      const results = await commands.testActivity(draft);
      const performed = results.filter((r) => r.performed).map((r) => r.kind);
      const skipped = results.filter((r) => !r.performed).map((r) => r.kind);
      let message =
        performed.length > 0 ? `Fired: ${performed.map(formatActivityKind).join(", ")}` : "";
      if (skipped.length > 0) {
        const skippedText = `skipped near screen corner: ${skipped.map(formatActivityKind).join(", ")}`;
        message = message ? `${message} (${skippedText})` : skippedText;
      }
      setTestResult(message);
    } catch (err) {
      setError(String(err));
    } finally {
      setTesting(false);
      unlisten();
    }
  }

  return (
    <div className="view">
      <div className="card stack">
        <div>
          <div className="field-label" style={{ marginBottom: 6 }}>
            Name
          </div>
          <input
            className="input"
            value={draft.name}
            disabled={readOnly}
            onChange={(e) => setDraft({ ...draft, name: e.target.value })}
          />
        </div>
        <div>
          <div className="field-label" style={{ marginBottom: 6 }}>
            Random Delay Range (Seconds)
          </div>
          <div style={{ display: "flex", gap: 8 }}>
            <input
              type="number"
              min={1}
              className="input"
              value={draft.minDelay}
              disabled={readOnly}
              onChange={(e) => setDraft({ ...draft, minDelay: Number(e.target.value) })}
            />
            <input
              type="number"
              min={1}
              className="input"
              value={draft.maxDelay}
              disabled={readOnly}
              onChange={(e) => setDraft({ ...draft, maxDelay: Number(e.target.value) })}
            />
          </div>
        </div>
        <div>
          <div className="field-label" style={{ marginBottom: 6 }}>
            Inactivity Threshold
          </div>
          <select
            className="input"
            value={draft.inactivityThreshold}
            disabled={readOnly}
            onChange={(e) => setDraft({ ...draft, inactivityThreshold: Number(e.target.value) })}
          >
            {INACTIVITY_THRESHOLD_OPTIONS.map((secs) => (
              <option key={secs} value={secs}>
                {formatDurationShort(secs)}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div className="card stack">
        <div className="row">
          <div className="field-label">Mouse</div>
          <Toggle
            checked={draft.mouse.enabled}
            disabled={readOnly}
            onChange={(v) => setDraft({ ...draft, mouse: { ...draft.mouse, enabled: v } })}
          />
        </div>
        {draft.mouse.enabled && (
          <>
            <hr className="divider" />
            <div className="row">
              <div className="field-hint">Movement</div>
              <Toggle
                checked={draft.mouse.movement}
                disabled={readOnly}
                onChange={(v) => setDraft({ ...draft, mouse: { ...draft.mouse, movement: v } })}
              />
            </div>
            <div className="row">
              <div className="field-hint">Button</div>
              <select
                className="input"
                style={{ width: "auto" }}
                value={draft.mouse.button}
                disabled={readOnly}
                onChange={(e) =>
                  setDraft({
                    ...draft,
                    mouse: {
                      ...draft.mouse,
                      button: e.target.value as ActivityProfile["mouse"]["button"],
                    },
                  })
                }
              >
                <option value="left">Left</option>
                <option value="right">Right</option>
                <option value="middle">Middle</option>
              </select>
            </div>
          </>
        )}
      </div>

      <div className="card stack">
        <div className="row">
          <div className="field-label">Keyboard</div>
          <Toggle
            checked={draft.keyboard.enabled}
            disabled={readOnly}
            onChange={(v) => setDraft({ ...draft, keyboard: { ...draft.keyboard, enabled: v } })}
          />
        </div>
        {draft.keyboard.enabled && (
          <>
            <hr className="divider" />
            <div>
              <div className="field-hint" style={{ marginBottom: 6 }}>
                Key
              </div>
              <KeyRecorder
                value={draft.keyboard.key}
                disabled={readOnly}
                onChange={(key) => setDraft({ ...draft, keyboard: { ...draft.keyboard, key } })}
              />
            </div>
          </>
        )}
      </div>

      <div className="card stack">
        <div className="field-label">Gestures</div>
        <div className="row">
          <div className="field-hint">Horizontal Scroll</div>
          <Toggle
            checked={draft.gestures.horizontal}
            disabled={readOnly}
            onChange={(v) => setDraft({ ...draft, gestures: { ...draft.gestures, horizontal: v } })}
          />
        </div>
        <div className="row">
          <div className="field-hint">Vertical Scroll</div>
          <Toggle
            checked={draft.gestures.vertical}
            disabled={readOnly}
            onChange={(v) => setDraft({ ...draft, gestures: { ...draft.gestures, vertical: v } })}
          />
        </div>
      </div>

      <div className="card stack">
        <div className="row">
          <div>
            <div className="field-label">Test Activity</div>
            <div className="field-hint">
              {testResult ??
                "Fire every enabled action, one after another, to preview this profile."}
            </div>
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
        <div className="banner banner-danger">
          <AlertIcon />
          <span>{error}</span>
        </div>
      )}

      <div className="stack">
        {!readOnly && (
          <button className="btn btn-primary btn-compact" disabled={saving} onClick={handleSave}>
            {saving ? "Saving…" : "Save Profile"}
          </button>
        )}
        <button
          className={`btn ${readOnly ? "btn-secondary" : "btn-link"}`}
          style={readOnly ? undefined : { alignSelf: "center" }}
          disabled={testing}
          onClick={onDone}
        >
          {readOnly ? "Close" : "Cancel"}
        </button>
      </div>
    </div>
  );
}
