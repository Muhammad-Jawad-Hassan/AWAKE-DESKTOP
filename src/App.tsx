import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import { BackIcon, GearIcon, PowerIcon } from "@/components/Icons";
import { ActiveView } from "@/pages/ActiveView";
import { ProfileEditorView } from "@/pages/ProfileEditorView";
import { SettingsView } from "@/pages/SettingsView";
import { SetupView } from "@/pages/SetupView";
import { useSessionSnapshot } from "@/hooks/useSessionSnapshot";
import { commands } from "@/lib/commands";
import type {
  ActivityProfile,
  AppSettings,
  HistoryEntry,
  PlatformCapabilities,
  SessionTemplate,
} from "@/lib/types";

type View = "main" | "settings" | "profile-editor";

export function App() {
  const snapshot = useSessionSnapshot();
  const [view, setView] = useState<View>("main");
  const [editingProfile, setEditingProfile] = useState<ActivityProfile | null>(null);
  const [profiles, setProfiles] = useState<ActivityProfile[]>([]);
  const [templates, setTemplates] = useState<SessionTemplate[]>([]);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [capabilities, setCapabilities] = useState<PlatformCapabilities | null>(null);
  const [settingsError, setSettingsError] = useState<string | null>(null);

  const refetchProfiles = useCallback(() => {
    commands.listProfiles().then(setProfiles);
  }, []);

  const refetchTemplates = useCallback(() => {
    commands.listTemplates().then(setTemplates);
  }, []);

  const refetchHistory = useCallback(() => {
    commands.listSessionHistory().then(setHistory);
  }, []);

  useEffect(() => {
    refetchProfiles();
    refetchTemplates();
    refetchHistory();
    commands.getSettings().then(setSettings);
    commands.getCapabilities().then(setCapabilities);

    const unlisten = listen<string>("navigate", (event) => {
      if (event.payload === "settings") setView("settings");
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, [refetchProfiles, refetchTemplates, refetchHistory]);

  async function handleDeleteProfile(id: string) {
    setSettingsError(null);
    try {
      await commands.deleteProfile(id);
    } catch (err) {
      setSettingsError(String(err));
    }
    refetchProfiles();
  }

  async function handleImportProfile() {
    setSettingsError(null);
    try {
      const imported = await commands.importProfile();
      if (imported) refetchProfiles();
    } catch (err) {
      setSettingsError(String(err));
    }
  }

  async function handleExportProfile(id: string) {
    setSettingsError(null);
    try {
      await commands.exportProfile(id);
    } catch (err) {
      setSettingsError(String(err));
    }
  }

  async function handleDeleteTemplate(id: string) {
    setSettingsError(null);
    try {
      await commands.deleteTemplate(id);
    } catch (err) {
      setSettingsError(String(err));
    }
    refetchTemplates();
  }

  async function handleClearHistory() {
    setSettingsError(null);
    try {
      await commands.clearSessionHistory();
    } catch (err) {
      setSettingsError(String(err));
    }
    refetchHistory();
  }

  const [endedBannerDismissed, setEndedBannerDismissed] = useState(false);
  const wasActive = useRef(false);
  useEffect(() => {
    if (wasActive.current && snapshot?.state !== "active") {
      refetchHistory();
      setEndedBannerDismissed(false);
    }
    wasActive.current = snapshot?.state === "active";
  }, [snapshot?.state, refetchHistory]);

  function closeProfileEditor() {
    setEditingProfile(null);
    setView("settings");
    refetchProfiles();
  }

  const showBack = view !== "main";

  return (
    <div className="app">
      <div className="titlebar">
        {showBack ? (
          <button
            className="icon-btn"
            onClick={() => (view === "profile-editor" ? closeProfileEditor() : setView("main"))}
            aria-label="Back"
          >
            <BackIcon />
          </button>
        ) : (
          <button className="icon-btn" onClick={() => setView("settings")} aria-label="Settings">
            <GearIcon />
          </button>
        )}
        <h1>
          {showBack ? (
            view === "profile-editor" ? (
              editingProfile ? (
                "Profile"
              ) : (
                "New Profile"
              )
            ) : (
              "Settings"
            )
          ) : (
            <span className="app-title">
              <span className="app-title-icon">
                <PowerIcon size={15} />
              </span>
              Awake
            </span>
          )}
        </h1>
        <span style={{ width: 28 }} />
      </div>

      {view === "settings" && settings && (
        <SettingsView
          settings={settings}
          onSettingsChange={setSettings}
          profiles={profiles}
          templates={templates}
          history={history}
          capabilities={capabilities}
          error={settingsError}
          onEditProfile={(p) => {
            setEditingProfile(p);
            setView("profile-editor");
          }}
          onDeleteProfile={handleDeleteProfile}
          onImportProfile={handleImportProfile}
          onExportProfile={handleExportProfile}
          onDeleteTemplate={handleDeleteTemplate}
          onClearHistory={handleClearHistory}
        />
      )}

      {view === "profile-editor" && (
        <ProfileEditorView profile={editingProfile} onDone={closeProfileEditor} />
      )}

      {view === "main" &&
        (!snapshot ? (
          <div className="view">
            <p className="muted">Loading…</p>
          </div>
        ) : snapshot.state === "active" ? (
          <ActiveView snapshot={snapshot} profiles={profiles} />
        ) : (
          <SetupView
            profiles={profiles}
            templates={templates}
            capabilities={capabilities}
            snapshot={snapshot}
            endedBannerDismissed={endedBannerDismissed}
            onDismissEndedBanner={() => setEndedBannerDismissed(true)}
            onStarted={() => setView("main")}
            onTemplateSaved={refetchTemplates}
          />
        ))}
    </div>
  );
}
