/** Physical keys the global-shortcut parser accepts, by `KeyboardEvent.code`. */
const KEY_CODE =
  /^(Key[A-Z]|Digit[0-9]|F([1-9]|1[0-9]|2[0-4])|Arrow(Up|Down|Left|Right)|Escape|Space|Enter|Tab|Backspace|Delete|Insert|Home|End|PageUp|PageDown|Minus|Equal|BracketLeft|BracketRight|Backslash|Semicolon|Quote|Comma|Period|Slash|Backquote)$/;

type KeyInput = Pick<KeyboardEvent, "code" | "ctrlKey" | "metaKey" | "altKey" | "shiftKey">;

/**
 * Builds an accelerator like "Control+Shift+KeyK", or null if the combination is unusable.
 * Needs Ctrl, Cmd/Win or Alt unless the key is an F-key, so plain typing is never captured.
 */
export function shortcutFromEvent(e: KeyInput): string | null {
  if (!KEY_CODE.test(e.code)) return null;
  const modifiers = [
    e.ctrlKey && "Control",
    e.metaKey && "Super",
    e.altKey && "Alt",
    e.shiftKey && "Shift",
  ].filter((m): m is string => Boolean(m));
  const isFunctionKey = /^F\d+$/.test(e.code);
  const hasCommandModifier = e.ctrlKey || e.metaKey || e.altKey;
  if (!hasCommandModifier && !isFunctionKey) return null;
  return [...modifiers, e.code].join("+");
}

const ARROWS: Record<string, string> = { up: "↑", down: "↓", left: "←", right: "→" };

/** Human label for one accelerator token; `layout` maps key codes to printed keys. */
export function shortcutPartLabel(
  part: string,
  isMac: boolean,
  layout?: ReadonlyMap<string, string>,
): string {
  const printed = /^(Key|Digit)/.test(part) ? layout?.get(part) : undefined;
  if (printed) return printed.toUpperCase();
  const lower = part.toLowerCase();
  switch (lower) {
    case "control":
    case "ctrl":
      return "Ctrl";
    case "super":
    case "cmd":
    case "command":
    case "meta":
      return isMac ? "Cmd" : "Win";
    case "commandorcontrol":
    case "cmdorctrl":
      return isMac ? "Cmd" : "Ctrl";
    case "alt":
    case "option":
      return isMac ? "Option" : "Alt";
    case "shift":
      return "Shift";
    case "escape":
    case "esc":
      return "Esc";
  }
  const key = part.replace(/^(Key|Digit|Arrow)/, "");
  return ARROWS[key.toLowerCase()] ?? key;
}
