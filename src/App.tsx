import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import { AlertIcon, BackIcon, GearIcon, PowerIcon } from "@/components/Icons";
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
  Limits,
  PlatformCapabilities,
  SessionTemplate,
} from "@/lib/types";

type View = "main" | "settings" | "profile-editor";

export function App() {
  const { snapshot, error: snapshotError } = useSessionSnapshot();
  const [view, setView] = useState<View>("main");
  const [editingProfile, setEditingProfile] = useState<ActivityProfile | null>(null);
  const [isNewProfile, setIsNewProfile] = useState(false);
  const [limits, setLimits] = useState<Limits | null>(null);
  const [notices, setNotices] = useState<string[]>([]);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [profiles, setProfiles] = useState<ActivityProfile[]>([]);
  const [templates, setTemplates] = useState<SessionTemplate[]>([]);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [capabilities, setCapabilities] = useState<PlatformCapabilities | null>(null);
  const [settingsError, setSettingsError] = useState<string | null>(null);

  const load = useCallback(<T,>(request: Promise<T>, apply: (value: T) => void) => {
    request.then(apply).catch((err) => setLoadError(`Couldn't load app data: ${String(err)}`));
  }, []);

  const refetchProfiles = useCallback(() => load(commands.listProfiles(), setProfiles), [load]);
  const refetchTemplates = useCallback(() => load(commands.listTemplates(), setTemplates), [load]);
  const refetchHistory = useCallback(() => load(commands.listSessionHistory(), setHistory), [load]);
  const refetchCapabilities = useCallback(
    () => load(commands.getCapabilities(), setCapabilities),
    [load],
  );

  useEffect(() => {
    refetchProfiles();
    refetchTemplates();
    refetchHistory();
    refetchCapabilities();
    load(commands.getSettings(), setSettings);
    load(commands.getLimits(), setLimits);
    load(commands.takeNotices(), (taken) => setNotices((current) => [...current, ...taken]));

    // Permissions may change in System Settings.
    window.addEventListener("focus", refetchCapabilities);
    const unlisten = listen<string>("navigate", (event) => {
      // Keep the profile editor draft.
      if (event.payload === "settings") {
        setView((current) => (current === "profile-editor" ? current : "settings"));
      }
    });
    return () => {
      window.removeEventListener("focus", refetchCapabilities);
      unlisten.then((f) => f());
    };
  }, [load, refetchProfiles, refetchTemplates, refetchHistory, refetchCapabilities]);

  async function openProfileEditor(profile: ActivityProfile | null) {
    setSettingsError(null);
    try {
      setEditingProfile(profile ?? (await commands.newProfile()));
      setIsNewProfile(profile === null);
      setView("profile-editor");
    } catch (err) {
      setSettingsError(String(err));
    }
  }

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
              isNewProfile ? (
                "New Profile"
              ) : (
                "Profile"
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

      {[...notices, ...[loadError, snapshotError].filter((e): e is string => !!e)].map(
        (message, i) => (
          <div
            key={i}
            className="banner banner-warning"
            role="alert"
            style={{ margin: "8px 12px 0" }}
          >
            <AlertIcon />
            <span>{message}</span>
          </div>
        ),
      )}

      {view === "settings" && settings && (
        <SettingsView
          settings={settings}
          onSettingsChange={setSettings}
          profiles={profiles}
          templates={templates}
          history={history}
          capabilities={capabilities}
          error={settingsError}
          activeProfileId={snapshot?.state === "active" ? snapshot.activityProfileId : null}
          onEditProfile={openProfileEditor}
          onDeleteProfile={handleDeleteProfile}
          onImportProfile={handleImportProfile}
          onExportProfile={handleExportProfile}
          onDeleteTemplate={handleDeleteTemplate}
          onClearHistory={handleClearHistory}
        />
      )}

      {view === "profile-editor" && editingProfile && limits && (
        <ProfileEditorView profile={editingProfile} limits={limits} onDone={closeProfileEditor} />
      )}

      {view === "main" && (!snapshot || !limits) && (
        <div className="view">
          <p className="muted">
            {loadError || snapshotError ? "Awake couldn't start. Try reopening it." : "Loading…"}
          </p>
        </div>
      )}

      {view === "main" && snapshot?.state === "active" && limits && (
        <ActiveView snapshot={snapshot} profiles={profiles} limits={limits} />
      )}

      {/* Kept mounted to preserve the form. */}
      {snapshot && snapshot.state !== "active" && limits && (
        <div style={{ display: view === "main" ? "contents" : "none" }}>
          <SetupView
            profiles={profiles}
            templates={templates}
            capabilities={capabilities}
            limits={limits}
            snapshot={snapshot}
            endedBannerDismissed={endedBannerDismissed}
            onDismissEndedBanner={() => setEndedBannerDismissed(true)}
            onTemplateSaved={refetchTemplates}
          />
        </div>
      )}
    </div>
  );
}
