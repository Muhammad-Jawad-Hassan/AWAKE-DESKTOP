import { describe, expect, it } from "vitest";

import {
  formatActivityKind,
  formatClock,
  formatDurationShort,
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
    expect(formatActivityKind("keyboard_input")).toBe("Keyboard Input");
    expect(formatActivityKind("gesture_horizontal")).toBe("Gesture Horizontal");
  });

  it("handles a single word", () => {
    expect(formatActivityKind("click")).toBe("Click");
  });
});

describe("formatDurationShort", () => {
  it("prefers hours when evenly divisible", () => {
    expect(formatDurationShort(3600)).toBe("1h");
    expect(formatDurationShort(7200)).toBe("2h");
  });

  it("prefers minutes when evenly divisible but not by an hour", () => {
    expect(formatDurationShort(60)).toBe("1m");
    expect(formatDurationShort(1800)).toBe("30m");
  });

  it("falls back to seconds otherwise", () => {
    expect(formatDurationShort(45)).toBe("45s");
    expect(formatDurationShort(90)).toBe("90s");
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
