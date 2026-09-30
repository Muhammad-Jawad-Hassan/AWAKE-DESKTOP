import { describe, expect, it } from "vitest";

import {
  formatActivityKind,
  formatActivityRates,
  formatClock,
  formatDuration,
  formatMinSec,
  formatRelative,
} from "./format";

describe("formatClock", () => {
  it("pads hours, minutes and seconds to two digits", () => {
    expect(formatClock(5)).toBe("00:00:05");
    expect(formatClock(65)).toBe("00:01:05");
    expect(formatClock(3661)).toBe("01:01:01");
  });

  it("handles zero", () => {
    expect(formatClock(0)).toBe("00:00:00");
  });

  it("does not roll hours over into a third digit incorrectly", () => {
    expect(formatClock(23 * 3600 + 59 * 60 + 59)).toBe("23:59:59");
  });
});

describe("formatRelative", () => {
  it("reports very recent times as 'just now'", () => {
    expect(formatRelative(0)).toBe("just now");
    expect(formatRelative(4)).toBe("just now");
  });

  it("reports seconds for under a minute", () => {
    expect(formatRelative(5)).toBe("5s ago");
    expect(formatRelative(59)).toBe("59s ago");
  });

  it("reports minutes for under an hour", () => {
    expect(formatRelative(60)).toBe("1m ago");
    expect(formatRelative(125)).toBe("2m ago");
  });

  it("reports hours beyond that", () => {
    expect(formatRelative(3600)).toBe("1h ago");
    expect(formatRelative(7200)).toBe("2h ago");
  });
});

describe("formatActivityKind", () => {
  it("converts snake_case backend kinds into title case", () => {
    expect(formatActivityKind("mouse_movement")).toBe("Mouse Movement");
    expect(formatActivityKind("keyboard_input")).toBe("Key Tap");
    expect(formatActivityKind("gesture_horizontal")).toBe("Horizontal Scroll");
    expect(formatActivityKind("gesture_vertical")).toBe("Vertical Scroll");
  });

  it("handles a single word", () => {
    expect(formatActivityKind("click")).toBe("Click");
  });
});

describe("formatDuration", () => {
  it("shows whole units without padding", () => {
    expect(formatDuration(3600)).toBe("1h");
    expect(formatDuration(60)).toBe("1m");
    expect(formatDuration(45)).toBe("45s");
  });

  it("shows the two most significant units", () => {
    expect(formatDuration(3597)).toBe("59m 57s");
    expect(formatDuration(3660)).toBe("1h 1m");
    expect(formatDuration(3601)).toBe("1h");
    expect(formatDuration(90)).toBe("1m 30s");
  });

  it("shows zero and bad input as 0s", () => {
    expect(formatDuration(0)).toBe("0s");
    expect(formatDuration(-5)).toBe("0s");
  });
});

describe("formatMinSec", () => {
  it("pads seconds but not minutes", () => {
    expect(formatMinSec(5)).toBe("0:05");
    expect(formatMinSec(65)).toBe("1:05");
    expect(formatMinSec(600)).toBe("10:00");
  });

  it("handles zero", () => {
    expect(formatMinSec(0)).toBe("0:00");
  });
});

describe("formatActivityRates", () => {
  const base = {
    id: "p",
    name: "P",
    inactivityThreshold: 60,
    mouse: { enabled: true, perMinute: 2 },
    keyboard: { enabled: true, key: "Shift", perMinute: 1 },
    gestures: { horizontal: false, vertical: false, perMinute: 3 },
    safety: { avoidScreenCornersPx: 24, maxActionsPerMinute: 6 },
    builtIn: false,
  };

  it("lists only enabled activities with their rates", () => {
    expect(formatActivityRates(base)).toBe("Mouse 2/min · Keys 1/min");
  });

  it("includes scroll when either axis is on", () => {
    const profile = { ...base, gestures: { ...base.gestures, vertical: true } };
    expect(formatActivityRates(profile)).toBe("Mouse 2/min · Keys 1/min · Scroll 3/min");
  });

  it("says so when nothing is enabled", () => {
    const profile = {
      ...base,
      mouse: { ...base.mouse, enabled: false },
      keyboard: { ...base.keyboard, enabled: false },
    };
    expect(formatActivityRates(profile)).toBe("No activity");
  });
});
