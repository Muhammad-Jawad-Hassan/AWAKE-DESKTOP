import { describe, expect, it } from "vitest";

import { shortcutFromEvent, shortcutPartLabel } from "./shortcut";

const press = (
  code: string,
  mods: Partial<Record<"ctrl" | "meta" | "alt" | "shift", boolean>> = {},
) => ({
  code,
  ctrlKey: !!mods.ctrl,
  metaKey: !!mods.meta,
  altKey: !!mods.alt,
  shiftKey: !!mods.shift,
});

describe("shortcutFromEvent", () => {
  it("keeps Ctrl and Cmd distinct", () => {
    expect(shortcutFromEvent(press("KeyK", { ctrl: true, shift: true }))).toBe(
      "Control+Shift+KeyK",
    );
    expect(shortcutFromEvent(press("KeyK", { meta: true }))).toBe("Super+KeyK");
  });

  it("uses the physical key, so Option combos stay parseable", () => {
    expect(shortcutFromEvent(press("KeyE", { alt: true }))).toBe("Alt+KeyE");
  });

  it("rejects plain or Shift-only typing", () => {
    expect(shortcutFromEvent(press("KeyA"))).toBeNull();
    expect(shortcutFromEvent(press("KeyA", { shift: true }))).toBeNull();
  });

  it("allows function keys on their own", () => {
    expect(shortcutFromEvent(press("F13"))).toBe("F13");
  });

  it("ignores modifier-only presses and unknown keys", () => {
    expect(shortcutFromEvent(press("ShiftLeft", { shift: true }))).toBeNull();
    expect(shortcutFromEvent(press("IntlBackslash", { ctrl: true }))).toBeNull();
  });
});

describe("shortcutPartLabel", () => {
  it("names modifiers per platform", () => {
    expect(shortcutPartLabel("Super", true)).toBe("Cmd");
    expect(shortcutPartLabel("Super", false)).toBe("Win");
    expect(shortcutPartLabel("CommandOrControl", false)).toBe("Ctrl");
    expect(shortcutPartLabel("Alt", true)).toBe("Option");
  });

  it("uses the printed key from the layout when known", () => {
    const azerty = new Map([["KeyQ", "a"]]);
    expect(shortcutPartLabel("KeyQ", false, azerty)).toBe("A");
    expect(shortcutPartLabel("KeyW", false, azerty)).toBe("W");
  });

  it("strips code prefixes from keys", () => {
    expect(shortcutPartLabel("KeyK", true)).toBe("K");
    expect(shortcutPartLabel("Digit1", true)).toBe("1");
    expect(shortcutPartLabel("ArrowUp", true)).toBe("↑");
    expect(shortcutPartLabel("Escape", true)).toBe("Esc");
  });
});
