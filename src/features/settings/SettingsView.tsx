import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { api } from "@/lib/invoke";
import { useSession } from "@/hooks/useSession";

export function SettingsView() {
  const { lock } = useSession();
  const [timeoutMinutes, setTimeoutMinutes] = useState(10);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    void api.authGetLockTimeout().then((secs) => {
      setTimeoutMinutes(Math.round(secs / 60));
    });
  }, []);

  const saveTimeout = async () => {
    await api.authSetLockTimeout(timeoutMinutes * 60);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="mx-auto max-w-lg space-y-8 p-8">
      <h2 className="text-2xl font-semibold">Settings</h2>

      <section className="space-y-4 rounded-xl border border-border bg-card/50 p-6">
        <h3 className="font-medium">Auto-lock</h3>
        <p className="text-sm text-muted-foreground">
          Lock the app after a period of inactivity.
        </p>
        <div className="flex items-end gap-3">
          <div className="space-y-2">
            <Label htmlFor="timeout">Timeout (minutes)</Label>
            <Input
              id="timeout"
              type="number"
              min={1}
              max={120}
              value={timeoutMinutes}
              onChange={(e) => setTimeoutMinutes(Number(e.target.value))}
            />
          </div>
          <Button onClick={() => void saveTimeout()}>Save</Button>
        </div>
        {saved && <p className="text-sm text-accent">Saved.</p>}
      </section>

      <section className="space-y-4 rounded-xl border border-border bg-card/50 p-6">
        <h3 className="font-medium">Security</h3>
        <p className="text-sm text-muted-foreground">
          Notes are encrypted on disk with AES-256-GCM. Your master password cannot be recovered
          if lost.
        </p>
        <Button variant="secondary" onClick={() => void lock()}>
          Lock now
        </Button>
      </section>
    </div>
  );
}
