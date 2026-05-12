import { useEffect, useState } from "react";

// Mirrors parse_key in src-tauri/src/platform/input.rs, not the accelerator
// vocabulary in ShortcutRecorder: this simulates one key press, not a chord.
const NAMED_KEYS: Record<string, string> = {
  " ": "Space",
  Control: "Control",
  Shift: "Shift",
  Alt: "Alt",
  Meta: "Meta",
  Enter: "Enter",
  Tab: "Tab",
};

const KEY_LABELS: Record<string, string> = {
  Space: "Space",
};

function normalizeAutomationKey(key: string): string | null {
  const named = NAMED_KEYS[key];
  if (named) return named;
  if (/^F([1-9]|1[0-5])$/.test(key)) return key;
  if (key.length === 1) return key.toUpperCase();
  return null;
}

interface KeyRecorderProps {
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
}

export function KeyRecorder({ value, onChange, disabled }: KeyRecorderProps) {
  const [recording, setRecording] = useState(false);

  useEffect(() => {
    if (!recording) return;

    function handleKeyDown(e: KeyboardEvent) {
      e.preventDefault();
      e.stopPropagation();

      if (e.key === "Escape") {
        setRecording(false);
        return;
      }
      const key = normalizeAutomationKey(e.key);
      if (!key) return;
      onChange(key);
      setRecording(false);
    }

    window.addEventListener("keydown", handleKeyDown, true);
    return () => window.removeEventListener("keydown", handleKeyDown, true);
  }, [recording, onChange]);

  return (
    <button
      type="button"
      className={`shortcut-recorder ${recording ? "shortcut-recorder-active" : ""}`}
      disabled={disabled}
      onClick={() => setRecording(true)}
      onBlur={() => setRecording(false)}
    >
      {recording ? (
        <span className="shortcut-recorder-hint">Press a key… (Esc to cancel)</span>
      ) : (
        <kbd className="key-badge">{KEY_LABELS[value] ?? value}</kbd>
      )}
    </button>
  );
}
