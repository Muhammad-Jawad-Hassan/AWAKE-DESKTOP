import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

import { commands } from "@/lib/commands";
import type { SessionSnapshot } from "@/lib/types";

export function useSessionSnapshot() {
  const [snapshot, setSnapshot] = useState<SessionSnapshot | null>(null);

  useEffect(() => {
    let cancelled = false;

    commands.getSnapshot().then((s) => {
      if (!cancelled) setSnapshot(s);
    });

    const unlisten = listen<SessionSnapshot>("session://update", (event) => {
      setSnapshot(event.payload);
    });

    return () => {
      cancelled = true;
      unlisten.then((f) => f());
    };
  }, []);

  return snapshot;
}
