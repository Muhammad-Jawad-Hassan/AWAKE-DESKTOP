import { useEffect, useState } from "react";

const MODIFIER_LABELS: Record<string, string> = {
  commandorcontrol: "Ctrl",
  cmdorctrl: "Ctrl",
  command: "Cmd",
  cmd: "Cmd",
  control: "Ctrl",
  ctrl: "Ctrl",
  alt: "Alt",
  option: "Alt",
  shift: "Shift",
  super: "Super",
  meta: "Meta",
};

const KEY_LABELS: Record<string, string> = {
  escape: "Esc",
  enter: "Enter",
  return: "Enter",
  space: "Space",
  tab: "Tab",
  backspace: "Backspace",
  delete: "Delete",
  up: "↑",
  down: "↓",
  left: "←",
  right: "→",
};

function partLabel(part: string): string {
  const lower = part.toLowerCase();
  return (
    MODIFIER_LABELS[lower] ?? KEY_LABELS[lower] ?? (part.length === 1 ? part.toUpperCase() : part)
  );
}

function normalizeKey(key: string): string | null {
  if (key.length === 1) return key.toUpperCase();
  const map: Record<string, string> = {
    Escape: "Escape",
    Enter: "Return",
    " ": "Space",
    Tab: "Tab",
    Backspace: "Backspace",
    Delete: "Delete",
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
  };
  return map[key] ?? (/^F\d{1,2}$/.test(key) ? key : null);
}

interface ShortcutRecorderProps {
  value: string;
  onChange: (value: string) => void;
}

export function ShortcutRecorder({ value, onChange }: ShortcutRecorderProps) {
  const [recording, setRecording] = useState(false);

  useEffect(() => {
    if (!recording) return;

    function handleKeyDown(e: KeyboardEvent) {
      e.preventDefault();
      e.stopPropagation();

      if (e.key === "Escape" && !e.metaKey && !e.ctrlKey && !e.altKey && !e.shiftKey) {
        setRecording(false);
        return;
      }
      if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;

      const key = normalizeKey(e.key);
      if (!key) return;

      const parts: string[] = [];
      if (e.metaKey || e.ctrlKey) parts.push("CommandOrControl");
      if (e.shiftKey) parts.push("Shift");
      if (e.altKey) parts.push("Alt");
      parts.push(key);

      if (parts.length < 2) return;
      onChange(parts.join("+"));
      setRecording(false);
    }

    window.addEventListener("keydown", handleKeyDown, true);
    return () => window.removeEventListener("keydown", handleKeyDown, true);
  }, [recording, onChange]);

  const parts = value.split("+").filter(Boolean);

  return (
    <button
      type="button"
      className={`shortcut-recorder ${recording ? "shortcut-recorder-active" : ""}`}
      onClick={() => setRecording(true)}
      onBlur={() => setRecording(false)}
    >
      {recording ? (
        <span className="shortcut-recorder-hint">Press a key combination… (Esc to cancel)</span>
      ) : parts.length === 0 ? (
        <span className="shortcut-recorder-hint">Click to set a shortcut</span>
      ) : (
        parts.map((part, i) => (
          <span className="key-badge-group" key={`${part}-${i}`}>
            {i > 0 && <span className="key-plus">+</span>}
            <kbd className="key-badge">{partLabel(part)}</kbd>
          </span>
        ))
      )}
    </button>
  );
}
