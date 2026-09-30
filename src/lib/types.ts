// Mirrors the Rust serde shapes.

export type SessionState = "idle" | "active" | "completed" | "stopped" | "failed";

export type ActivityKind =
  "mouse_movement" | "keyboard_input" | "gesture_horizontal" | "gesture_vertical";

export interface TestActivityProgress {
  kind: ActivityKind;
  index: number;
  total: number;
}

export interface TestActivityResult {
  kind: ActivityKind;
  performed: boolean;
}

export interface SessionConfig {
  duration: number;
  keepSystemAwake: boolean;
  keepDisplayAwake: boolean;
  inactivityThreshold: number;
  activityProfileId: string | null;
}

export interface SessionSnapshot {
  state: SessionState;
  remainingSecs: number;
  elapsedSecs: number;
  durationSecs: number;
  keepSystemAwake: boolean;
  keepDisplayAwake: boolean;
  activityProfileId: string | null;
  userInactive: boolean;
  idleSecs: number;
  inactivityThresholdSecs: number;
  activityPaused: boolean;
  lastActivity: ActivityKind | null;
  lastActivitySecsAgo: number | null;
  activityWarning: string | null;
  stats: SessionStats | null;
}

export interface MouseConfig {
  enabled: boolean;
  perMinute: number;
}

export interface KeyboardConfig {
  enabled: boolean;
  key: string;
  perMinute: number;
}

export interface GestureConfig {
  horizontal: boolean;
  vertical: boolean;
  perMinute: number;
}

export interface SafetySettings {
  avoidScreenCornersPx: number;
  maxActionsPerMinute: number;
}

export interface ActivityProfile {
  id: string;
  name: string;
  inactivityThreshold: number;
  mouse: MouseConfig;
  keyboard: KeyboardConfig;
  gestures: GestureConfig;
  safety: SafetySettings;
  builtIn: boolean;
}

export interface AppSettings {
  startMinimized: boolean;
  launchAtLogin: boolean;
  closeToTray: boolean;
  excludeFromScreenCapture: boolean;
  notifyOnSessionEnd: boolean;
  emergencyStopShortcut: string;
  recordActivityStatistics: boolean;
}

export interface SessionTemplate {
  id: string;
  name: string;
  config: SessionConfig;
}

export interface SessionStats {
  activeSecs: number;
  inactiveSecs: number;
  automatedEventCount: number;
  longestInactiveSecs: number;
  currentInactiveStreakSecs: number;
}

export interface HistoryEntry {
  id: string;
  endedAtUnixSecs: number;
  durationSecs: number;
  activityProfileId: string | null;
  stats: SessionStats;
}

export interface Limits {
  safeKeys: string[];
  randomKey: string;
  maxPerMinute: number;
  maxSafetyCap: number;
  maxSessionSecs: number;
}

export interface PlatformCapabilities {
  os: string;
  powerManagement: boolean;
  idleDetection: boolean;
  inputSimulation: boolean;
  inputPermissionGranted: boolean;
  screenCaptureExclusion: boolean;
  notes: string[];
}
