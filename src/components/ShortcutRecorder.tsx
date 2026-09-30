import { useEffect, useState } from "react";

import { shortcutFromEvent, shortcutPartLabel } from "@/lib/shortcut";

const IS_MAC = navigator.userAgent.includes("Mac");

interface KeyboardLayoutApi {
  keyboard?: { getLayoutMap?: () => Promise<ReadonlyMap<string, string>> };
}

interface ShortcutRecorderProps {
  value: string;
  onChange: (value: string) => void;
}

export function ShortcutRecorder({ value, onChange }: ShortcutRecorderProps) {
  const [recording, setRecording] = useState(false);
  const [layout, setLayout] = useState<ReadonlyMap<string, string>>();

  useEffect(() => {
    (navigator as KeyboardLayoutApi).keyboard
      ?.getLayoutMap?.()
      .then(setLayout)
      .catch(() => {});
  }, []);

  useEffect(() => {
    if (!recording) return;

    function handleKeyDown(e: KeyboardEvent) {
      e.preventDefault();
      e.stopPropagation();
      const bare = !e.metaKey && !e.ctrlKey && !e.altKey && !e.shiftKey;
      if (e.code === "Escape" && bare) {
        setRecording(false);
        return;
      }
      const shortcut = shortcutFromEvent(e);
      if (!shortcut) return;
      onChange(shortcut);
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
      aria-label="Emergency stop shortcut"
      onClick={() => setRecording(true)}
      onBlur={() => setRecording(false)}
    >
      {recording ? (
        <span className="shortcut-recorder-hint">
          Press a combination with {IS_MAC ? "Cmd, Ctrl or Option" : "Ctrl, Win or Alt"}… (Esc to
          cancel)
        </span>
      ) : parts.length === 0 ? (
        <span className="shortcut-recorder-hint">Click to set a shortcut</span>
      ) : (
        parts.map((part, i) => (
          <span className="key-badge-group" key={`${part}-${i}`}>
            {i > 0 && <span className="key-plus">+</span>}
            <kbd className="key-badge">{shortcutPartLabel(part, IS_MAC, layout)}</kbd>
          </span>
        ))
      )}
    </button>
  );
}
