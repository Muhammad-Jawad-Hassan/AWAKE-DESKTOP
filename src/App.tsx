import { useCallback, useEffect, useRef, useState } from "react";

import { PowerIcon } from "@/components/Icons";
import { ActiveView } from "@/pages/ActiveView";
import { SetupView } from "@/pages/SetupView";
import { useSessionSnapshot } from "@/hooks/useSessionSnapshot";
import { commands } from "@/lib/commands";
import type { ActivityProfile, PlatformCapabilities, SessionTemplate } from "@/lib/types";

export function App() {
  const snapshot = useSessionSnapshot();
  const [profiles, setProfiles] = useState<ActivityProfile[]>([]);
  const [templates, setTemplates] = useState<SessionTemplate[]>([]);
  const [capabilities, setCapabilities] = useState<PlatformCapabilities | null>(null);
  const [endedBannerDismissed, setEndedBannerDismissed] = useState(false);

  const refetchTemplates = useCallback(() => {
    commands.listTemplates().then(setTemplates);
  }, []);

  useEffect(() => {
    commands.listProfiles().then(setProfiles);
    refetchTemplates();
    commands.getCapabilities().then(setCapabilities);
  }, [refetchTemplates]);

  const wasActive = useRef(false);
  useEffect(() => {
    if (wasActive.current && snapshot?.state !== "active") {
      setEndedBannerDismissed(false);
    }
    wasActive.current = snapshot?.state === "active";
  }, [snapshot?.state]);

  return (
    <div className="app">
      <div className="titlebar">
        <span style={{ width: 28 }} />
        <h1>
          <span className="app-title">
            <span className="app-title-icon">
              <PowerIcon size={15} />
            </span>
            Awake
          </span>
        </h1>
        <span style={{ width: 28 }} />
      </div>

      {!snapshot ? (
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
          onStarted={() => setEndedBannerDismissed(true)}
          onTemplateSaved={refetchTemplates}
        />
      )}
    </div>
  );
}
