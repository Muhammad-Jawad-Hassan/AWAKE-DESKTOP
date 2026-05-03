import { invoke } from "@tauri-apps/api/core";

import type {
  ActivityProfile,
  AppSettings,
  HistoryEntry,
  PlatformCapabilities,
  SessionConfig,
  SessionSnapshot,
  SessionStats,
  SessionTemplate,
  TestActivityResult,
} from "./types";

export const commands = {
  getSnapshot: () => invoke<SessionSnapshot>("get_snapshot"),
  getLastSessionConfig: () => invoke<SessionConfig | null>("get_last_session_config"),
  getSessionStats: () => invoke<SessionStats | null>("get_session_stats"),
  listSessionHistory: () => invoke<HistoryEntry[]>("list_session_history"),
  clearSessionHistory: () => invoke<void>("clear_session_history"),
  startSession: (config: SessionConfig) => invoke<SessionSnapshot>("start_session", { config }),
  stopSession: () => invoke<SessionSnapshot>("stop_session"),
  extendSession: (extraSecs: number) => invoke<SessionSnapshot>("extend_session", { extraSecs }),
  pauseActivity: () => invoke<SessionSnapshot>("pause_activity"),
  resumeActivity: () => invoke<SessionSnapshot>("resume_activity"),
  emergencyStop: () => invoke<SessionSnapshot>("emergency_stop"),
  listProfiles: () => invoke<ActivityProfile[]>("list_profiles"),
  saveProfile: (profile: ActivityProfile) => invoke<void>("save_profile", { profile }),
  deleteProfile: (id: string) => invoke<void>("delete_profile", { id }),
  testActivity: (profile: ActivityProfile) =>
    invoke<TestActivityResult[]>("test_activity", { profile }),
  exportProfile: (id: string) => invoke<boolean>("export_profile", { id }),
  importProfile: () => invoke<ActivityProfile | null>("import_profile"),
  listTemplates: () => invoke<SessionTemplate[]>("list_templates"),
  saveTemplate: (template: SessionTemplate) => invoke<void>("save_template", { template }),
  deleteTemplate: (id: string) => invoke<void>("delete_template", { id }),
  getSettings: () => invoke<AppSettings>("get_settings"),
  updateSettings: (settings: AppSettings) => invoke<void>("update_settings", { settings }),
  getCapabilities: () => invoke<PlatformCapabilities>("get_capabilities"),
  requestPermissions: () => invoke<void>("request_permissions"),
};
