import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import { listen } from "@tauri-apps/api/event";
import { api, type AuthStatus } from "@/lib/invoke";

interface SessionContextValue {
  status: AuthStatus | null;
  loading: boolean;
  refresh: () => Promise<void>;
  unlock: (password: string) => Promise<void>;
  setup: (password: string) => Promise<void>;
  lock: () => Promise<void>;
}

const SessionContext = createContext<SessionContextValue | null>(null);

export function SessionProvider({ children }: { children: React.ReactNode }) {
  const [status, setStatus] = useState<AuthStatus | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    const next = await api.authStatus();
    setStatus(next);
  }, []);

  useEffect(() => {
    refresh().finally(() => setLoading(false));
  }, [refresh]);

  useEffect(() => {
    const unsubs: Array<() => void> = [];
    void (async () => {
      unsubs.push(
        await listen("session-locked", () => {
          setStatus((s) => (s ? { ...s, unlocked: false } : s));
        }),
      );
      unsubs.push(
        await listen("session-unlocked", () => {
          void refresh();
        }),
      );
    })();
    return () => unsubs.forEach((fn) => fn());
  }, [refresh]);

  const unlock = useCallback(
    async (password: string) => {
      await api.authUnlock(password);
      await refresh();
    },
    [refresh],
  );

  const setup = useCallback(
    async (password: string) => {
      await api.authSetup(password);
      await refresh();
    },
    [refresh],
  );

  const lock = useCallback(async () => {
    await api.authLock();
    await refresh();
  }, [refresh]);

  const value = useMemo(
    () => ({ status, loading, refresh, unlock, setup, lock }),
    [status, loading, refresh, unlock, setup, lock],
  );

  return (
    <SessionContext.Provider value={value}>{children}</SessionContext.Provider>
  );
}

export function useSession() {
  const ctx = useContext(SessionContext);
  if (!ctx) throw new Error("useSession must be used within SessionProvider");
  return ctx;
}
