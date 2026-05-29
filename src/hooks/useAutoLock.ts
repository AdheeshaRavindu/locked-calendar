import { useEffect } from "react";
import { api } from "@/lib/invoke";
import { useSession } from "@/hooks/useSession";

const ACTIVITY_EVENTS = [
  "mousedown",
  "keydown",
  "touchstart",
  "scroll",
] as const;

const TOUCH_INTERVAL_MS = 30_000;

export function useAutoLock(enabled: boolean) {
  const { lock, status } = useSession();

  useEffect(() => {
    if (!enabled || !status?.unlocked) return;

    const touch = () => {
      void api.authTouchSession().catch(() => {
        void lock();
      });
    };

    touch();

    const interval = window.setInterval(touch, TOUCH_INTERVAL_MS);
    for (const event of ACTIVITY_EVENTS) {
      window.addEventListener(event, touch, { passive: true });
    }

    return () => {
      window.clearInterval(interval);
      for (const event of ACTIVITY_EVENTS) {
        window.removeEventListener(event, touch);
      }
    };
  }, [enabled, lock, status?.unlocked]);
}
