import { isEnabled, type RatedActivity } from "@/lib/profile";
import type { ActivityProfile } from "@/lib/types";

export function formatClock(totalSecs: number): string {
  const h = Math.floor(totalSecs / 3600);
  const m = Math.floor((totalSecs % 3600) / 60);
  const s = Math.floor(totalSecs % 60);
  const pad = (n: number) => n.toString().padStart(2, "0");
  return `${pad(h)}:${pad(m)}:${pad(s)}`;
}

export function formatRelative(secs: number): string {
  if (secs < 5) return "just now";
  if (secs < 60) return `${secs}s ago`;
  const mins = Math.floor(secs / 60);
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  return `${hours}h ago`;
}

const ACTIVITY_NAMES: Record<string, string> = {
  mouse_movement: "Mouse Movement",
  keyboard_input: "Key Tap",
  gesture_horizontal: "Horizontal Scroll",
  gesture_vertical: "Vertical Scroll",
};

export function formatActivityKind(kind: string): string {
  if (ACTIVITY_NAMES[kind]) return ACTIVITY_NAMES[kind];
  return kind
    .split("_")
    .map((w) => w[0]!.toUpperCase() + w.slice(1))
    .join(" ");
}

const RATE_LABELS = { mouse: "Mouse", keyboard: "Keys", gestures: "Scroll" } as const;

/** Summarizes enabled activity rates, e.g. "Mouse 2/min · Keys 1/min". */
export function formatActivityRates(profile: ActivityProfile): string {
  const parts = (Object.keys(RATE_LABELS) as RatedActivity[])
    .filter((activity) => isEnabled(profile, activity))
    .map((activity) => `${RATE_LABELS[activity]} ${profile[activity].perMinute}/min`);
  return parts.length > 0 ? parts.join(" · ") : "No activity";
}

/** Two most significant units, e.g. "1h 5m", "59m 57s", "0s". */
export function formatDuration(totalSecs: number): string {
  const secs = Math.max(0, Math.floor(totalSecs));
  const parts = [
    [Math.floor(secs / 3600), "h"],
    [Math.floor((secs % 3600) / 60), "m"],
    [secs % 60, "s"],
  ] as const;
  const first = parts.findIndex(([value]) => value > 0);
  if (first === -1) return "0s";
  return parts
    .slice(first, first + 2)
    .filter(([value]) => value > 0)
    .map(([value, unit]) => `${value}${unit}`)
    .join(" ");
}

export function formatMinSec(totalSecs: number): string {
  const m = Math.floor(totalSecs / 60);
  const s = Math.floor(totalSecs % 60);
  return `${m}:${s.toString().padStart(2, "0")}`;
}

export function formatDateTime(unixSecs: number): string {
  return new Date(unixSecs * 1000).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
}
