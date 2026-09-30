import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import { commands } from "@/lib/commands";
import type { SessionSnapshot } from "@/lib/types";

export function useSessionSnapshot() {
  const [snapshot, setSnapshot] = useState<SessionSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    let sawEvent = false;

    const unlisten = listen<SessionSnapshot>("session://update", (event) => {
      sawEvent = true;
      setSnapshot(event.payload);
    });
    // Newer events win over this fetch.
    unlisten
      .then(() => commands.getSnapshot())
      .then((s) => {
        if (!cancelled && !sawEvent) setSnapshot(s);
      })
      .catch((err) => setError(`Couldn't load session state: ${String(err)}`));

    return () => {
      cancelled = true;
      unlisten.then((f) => f());
    };
  }, []);

  return { snapshot, error };
}
