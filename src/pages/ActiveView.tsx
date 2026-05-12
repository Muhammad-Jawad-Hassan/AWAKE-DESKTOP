import { useEffect, useState } from "react";

import { CountdownRing } from "@/components/CountdownRing";
import { AlertIcon, PauseIcon, PlayIcon, StopIcon } from "@/components/Icons";
import { StatusPill } from "@/components/StatusPill";
import { commands } from "@/lib/commands";
import { formatActivityKind, formatClock, formatMinSec, formatRelative } from "@/lib/format";
import type { ActivityProfile, SessionSnapshot, SessionStats } from "@/lib/types";

interface ActiveViewProps {
  snapshot: SessionSnapshot;
  profiles: ActivityProfile[];
}

export function ActiveView({ snapshot, profiles }: ActiveViewProps) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [stats, setStats] = useState<SessionStats | null>(null);
  const profile = profiles.find((p) => p.id === snapshot.activityProfileId);
  const progress =
    snapshot.durationSecs > 0 ? 1 - snapshot.remainingSecs / snapshot.durationSecs : 0;

  useEffect(() => {
    let cancelled = false;
    commands.getSessionStats().then((s) => {
      if (!cancelled) setStats(s);
    });
    return () => {
      cancelled = true;
    };
  }, [snapshot.remainingSecs]);

  async function run(action: () => Promise<unknown>) {
    setBusy(true);
    setError(null);
    try {
      await action();
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  const inactivityProgress =
    snapshot.inactivityThresholdSecs > 0
      ? Math.min(1, snapshot.idleSecs / snapshot.inactivityThresholdSecs)
      : 0;

  return (
    <div className="view">
      <div className="countdown">
        <CountdownRing remainingSecs={snapshot.remainingSecs} progress={progress} />
        <div className="countdown-label">Session Active</div>
      </div>

      <div className="extend-row">
        <button
          className="chip"
          style={{ flex: 1 }}
          disabled={busy}
          onClick={() => run(() => commands.extendSession(30 * 60))}
        >
          +30m
        </button>
        <button
          className="chip"
          style={{ flex: 1 }}
          disabled={busy}
          onClick={() => run(() => commands.extendSession(60 * 60))}
        >
          +1h
        </button>
      </div>

      <div className="card stack">
        <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
          {snapshot.keepSystemAwake && <StatusPill label="System Awake" tone="active" />}
          {snapshot.keepDisplayAwake && <StatusPill label="Display Awake" tone="active" />}
          {profile && (
            <StatusPill
              label={snapshot.activityPaused ? "Activity Paused" : `Activity: ${profile.name}`}
              tone={snapshot.activityPaused ? "default" : "active"}
            />
          )}
          <StatusPill
            label={snapshot.userInactive ? "User Inactive" : "User Active"}
            tone={snapshot.userInactive ? "success" : "default"}
          />
        </div>

        {profile && (
          <>
            <hr className="divider" />
            <div className="field-hint">
              {snapshot.userInactive
                ? `Inactive for ${formatMinSec(snapshot.idleSecs)}.`
                : `Idle ${formatMinSec(snapshot.idleSecs)} / ${formatMinSec(
                    snapshot.inactivityThresholdSecs,
                  )} until automation.`}
              {snapshot.lastActivity &&
                snapshot.lastActivitySecsAgo !== null &&
                ` Last activity: ${formatActivityKind(snapshot.lastActivity)} (${formatRelative(
                  snapshot.lastActivitySecsAgo,
                )}).`}
            </div>
            {!snapshot.userInactive && (
              <div className="mini-progress-track">
                <div
                  className="mini-progress-fill"
                  style={{ width: `${inactivityProgress * 100}%` }}
                />
              </div>
            )}
          </>
        )}
      </div>

      {stats && (
        <div className="card stack">
          <div className="section-title">Session Statistics</div>
          <div className="row">
            <span className="field-hint">Active / Inactive</span>
            <span className="field-label">
              {formatClock(stats.activeSecs)} / {formatClock(stats.inactiveSecs)}
            </span>
          </div>
          <div className="row">
            <span className="field-hint">Automated Events</span>
            <span className="field-label">{stats.automatedEventCount}</span>
          </div>
          <div className="row">
            <span className="field-hint">Longest Inactive Streak</span>
            <span className="field-label">{formatClock(stats.longestInactiveSecs)}</span>
          </div>
        </div>
      )}

      {error && (
        <div className="banner banner-danger">
          <AlertIcon />
          <span>{error}</span>
        </div>
      )}

      <div className="stack">
        {profile && (
          <button
            className="btn btn-secondary"
            disabled={busy}
            onClick={() =>
              run(snapshot.activityPaused ? commands.resumeActivity : commands.pauseActivity)
            }
          >
            {snapshot.activityPaused ? <PlayIcon /> : <PauseIcon />}
            {snapshot.activityPaused ? "Resume Activity" : "Pause Activity"}
          </button>
        )}
        <button
          className="btn btn-danger"
          disabled={busy}
          onClick={() => run(commands.stopSession)}
        >
          <StopIcon />
          Stop Session
        </button>
        {profile && !snapshot.activityPaused && (
          <button
            className="btn btn-link"
            style={{ alignSelf: "center" }}
            disabled={busy}
            onClick={() => run(commands.emergencyStop)}
          >
            <AlertIcon size={12} />
            Emergency Stop Automation
          </button>
        )}
      </div>
    </div>
  );
}
