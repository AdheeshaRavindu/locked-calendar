import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
} from "react";

const STORAGE_KEY = "locked-calendar:focus-mode";

interface FocusModeContextValue {
  focusMode: boolean;
  toggleFocusMode: () => void;
  setFocusMode: (value: boolean) => void;
}

const FocusModeContext = createContext<FocusModeContextValue | null>(null);

function readStoredFocusMode(): boolean {
  try {
    return localStorage.getItem(STORAGE_KEY) === "true";
  } catch {
    return false;
  }
}

export function FocusModeProvider({ children }: { children: React.ReactNode }) {
  const [focusMode, setFocusModeState] = useState(readStoredFocusMode);

  const setFocusMode = useCallback((value: boolean) => {
    setFocusModeState(value);
    try {
      localStorage.setItem(STORAGE_KEY, String(value));
    } catch {
      // ignore storage errors
    }
  }, []);

  const toggleFocusMode = useCallback(() => {
    setFocusMode(!focusMode);
  }, [focusMode, setFocusMode]);

  const value = useMemo(
    () => ({ focusMode, toggleFocusMode, setFocusMode }),
    [focusMode, toggleFocusMode, setFocusMode],
  );

  return (
    <FocusModeContext.Provider value={value}>{children}</FocusModeContext.Provider>
  );
}

export function useFocusMode() {
  const ctx = useContext(FocusModeContext);
  if (!ctx) {
    throw new Error("useFocusMode must be used within FocusModeProvider");
  }
  return ctx;
}
