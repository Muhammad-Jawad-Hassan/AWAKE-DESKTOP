import { describe, expect, it } from "vitest";

import { enabledRateTotal, fitUnderCap, maxRateFor } from "./profile";
import type { ActivityProfile, Limits } from "./types";

const limits: Limits = {
  safeKeys: ["Shift"],
  randomKey: "Random",
  maxPerMinute: 60,
  maxSafetyCap: 60,
  maxSessionSecs: 86400,
};

const base: ActivityProfile = {
  id: "p",
  name: "P",
  inactivityThreshold: 60,
  mouse: { enabled: true, perMinute: 2 },
  keyboard: { enabled: true, key: "Shift", perMinute: 3 },
  gestures: { horizontal: false, vertical: false, perMinute: 4 },
  safety: { avoidScreenCornersPx: 24, maxActionsPerMinute: 6 },
  builtIn: false,
};

describe("enabledRateTotal", () => {
  it("counts only enabled activities", () => {
    expect(enabledRateTotal(base)).toBe(5);
  });

  it("counts scroll once for both axes", () => {
    const profile = { ...base, gestures: { ...base.gestures, horizontal: true, vertical: true } };
    expect(enabledRateTotal(profile)).toBe(9);
  });
});

describe("maxRateFor", () => {
  it("leaves room only up to the cap", () => {
    expect(maxRateFor(base, "mouse", 60)).toBe(3);
    expect(maxRateFor(base, "keyboard", 60)).toBe(4);
  });

  it("ignores the activity's own stale rate when it's disabled", () => {
    expect(maxRateFor(base, "gestures", 60)).toBe(1);
  });

  it("never exceeds the per-activity maximum or drops below one", () => {
    const roomy = { ...base, safety: { ...base.safety, maxActionsPerMinute: 60 } };
    expect(maxRateFor(roomy, "mouse", 10)).toBe(10);
    const tight = { ...base, safety: { ...base.safety, maxActionsPerMinute: 1 } };
    expect(maxRateFor(tight, "mouse", 60)).toBe(1);
  });
});

describe("fitUnderCap", () => {
  it("raises the cap to fit a re-enabled activity", () => {
    const profile = { ...base, gestures: { ...base.gestures, vertical: true } };
    const fitted = fitUnderCap(profile, "gestures", limits);
    expect(fitted.safety.maxActionsPerMinute).toBe(9);
    expect(fitted.gestures.perMinute).toBe(4);
  });

  it("lowers the rate when the cap is already at its maximum", () => {
    const profile: ActivityProfile = {
      ...base,
      mouse: { enabled: true, perMinute: 58 },
      keyboard: { ...base.keyboard, enabled: false },
      gestures: { ...base.gestures, vertical: true, perMinute: 20 },
      safety: { ...base.safety, maxActionsPerMinute: 60 },
    };
    const fitted = fitUnderCap(profile, "gestures", limits);
    expect(fitted.safety.maxActionsPerMinute).toBe(60);
    expect(fitted.gestures.perMinute).toBe(2);
    expect(enabledRateTotal(fitted)).toBe(60);
  });

  it("leaves a profile that already fits alone", () => {
    expect(fitUnderCap(base, "mouse", limits)).toEqual(base);
  });
});
