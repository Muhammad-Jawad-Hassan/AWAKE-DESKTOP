import type { ActivityProfile, Limits } from "./types";

export type RatedActivity = "mouse" | "keyboard" | "gestures";

const RATED: RatedActivity[] = ["mouse", "keyboard", "gestures"];

export function isEnabled(profile: ActivityProfile, activity: RatedActivity): boolean {
  switch (activity) {
    case "mouse":
      return profile.mouse.enabled;
    case "keyboard":
      return profile.keyboard.enabled;
    case "gestures":
      return profile.gestures.horizontal || profile.gestures.vertical;
  }
}

function otherEnabledTotal(profile: ActivityProfile, activity: RatedActivity): number {
  return RATED.filter((a) => a !== activity && isEnabled(profile, a)).reduce(
    (sum, a) => sum + profile[a].perMinute,
    0,
  );
}

/** Sum of enabled activities' per-minute rates; mirrors `validate` in profiles.rs. */
export function enabledRateTotal(profile: ActivityProfile): number {
  return RATED.filter((a) => isEnabled(profile, a)).reduce(
    (sum, a) => sum + profile[a].perMinute,
    0,
  );
}

/** Highest rate `activity` can take without pushing the total past the cap. */
export function maxRateFor(
  profile: ActivityProfile,
  activity: RatedActivity,
  maxPerMinute: number,
): number {
  const room = profile.safety.maxActionsPerMinute - otherEnabledTotal(profile, activity);
  return Math.max(1, Math.min(maxPerMinute, room));
}

/**
 * Re-fits a just-enabled activity under the cap: raises the cap up to its maximum,
 * then lowers the activity's rate if it still doesn't fit.
 */
export function fitUnderCap(
  profile: ActivityProfile,
  activity: RatedActivity,
  limits: Limits,
): ActivityProfile {
  const total = enabledRateTotal(profile);
  const cap = Math.max(profile.safety.maxActionsPerMinute, Math.min(limits.maxSafetyCap, total));
  const fitted = { ...profile, safety: { ...profile.safety, maxActionsPerMinute: cap } };
  const rate = Math.max(
    1,
    Math.min(profile[activity].perMinute, cap - otherEnabledTotal(fitted, activity)),
  );
  return { ...fitted, [activity]: { ...profile[activity], perMinute: rate } };
}
