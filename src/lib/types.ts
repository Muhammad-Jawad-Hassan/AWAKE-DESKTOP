// Mirrors the serde JSON shapes produced by src-tauri/src/core and runtime.rs.

export type SessionState = "idle" | "active" | "completed" | "stopped" | "failed";

export type ActivityKind =
  "mouse_movement" | "mouse_click" | "keyboard_input" | "gesture_horizontal" | "gesture_vertical";

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
}

export type MouseButtonName = "left" | "right" | "middle";
export type ClickKind = "single" | "double";

export interface MouseConfig {
  enabled: boolean;
  movement: boolean;
  button: MouseButtonName;
  click: ClickKind;
  randomize: boolean;
}

export interface KeyboardConfig {
  enabled: boolean;
  key: string;
  modifiers: string[];
  randomize: boolean;
}

export interface GestureConfig {
  horizontal: boolean;
  vertical: boolean;
  custom: string | null;
}

export interface SafetySettings {
  avoidScreenCornersPx: number;
  maxActionsPerMinute: number;
}

export interface ActivityProfile {
  id: string;
  name: string;
  inactivityThreshold: number;
  minDelay: number;
  maxDelay: number;
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

export interface PlatformCapabilities {
  os: string;
  powerManagement: boolean;
  idleDetection: boolean;
  inputSimulation: boolean;
  screenCaptureExclusion: boolean;
  notes: string[];
}
