import { useEffect, useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { api } from "@/lib/invoke";
import { useSession } from "@/hooks/useSession";

export function SettingsView() {
  const { lock } = useSession();
  const [timeoutMinutes, setTimeoutMinutes] = useState(10);
  const [saved, setSaved] = useState(false);
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const [passwordSuccess, setPasswordSuccess] = useState(false);
  const [changingPassword, setChangingPassword] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [exportError, setExportError] = useState<string | null>(null);
  const [exportSuccess, setExportSuccess] = useState<string | null>(null);

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

  async function handleChangePassword(e: React.FormEvent) {
    e.preventDefault();
    setPasswordError(null);
    setPasswordSuccess(false);

    if (newPassword.length < 8) {
      setPasswordError("New password must be at least 8 characters.");
      return;
    }
    if (newPassword !== confirmPassword) {
      setPasswordError("New passwords do not match.");
      return;
    }

    setChangingPassword(true);
    try {
      await api.authChangePassword(currentPassword, newPassword);
      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
      setPasswordSuccess(true);
    } catch (err) {
      setPasswordError(String(err));
    } finally {
      setChangingPassword(false);
    }
  }

  async function handleExport() {
    setExportError(null);
    setExportSuccess(null);
    setExporting(true);
    try {
      const json = await api.exportNotesJson();
      const path = await save({
        defaultPath: `locked-calendar-backup-${new Date().toISOString().slice(0, 10)}.json`,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!path) {
        return;
      }
      await writeTextFile(path, json);
      setExportSuccess(`Backup saved to ${path}`);
    } catch (err) {
      setExportError(String(err));
    } finally {
      setExporting(false);
    }
  }

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
        <h3 className="font-medium">Change password</h3>
        <p className="text-sm text-muted-foreground">
          All notes are re-encrypted with your new password. This cannot be undone if you forget
          the new password.
        </p>
        <form onSubmit={(e) => void handleChangePassword(e)} className="space-y-3">
          <div className="space-y-2">
            <Label htmlFor="current-password">Current password</Label>
            <Input
              id="current-password"
              type="password"
              value={currentPassword}
              onChange={(e) => setCurrentPassword(e.target.value)}
              autoComplete="current-password"
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="new-password">New password</Label>
            <Input
              id="new-password"
              type="password"
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
              autoComplete="new-password"
              required
              minLength={8}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="confirm-password">Confirm new password</Label>
            <Input
              id="confirm-password"
              type="password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              autoComplete="new-password"
              required
              minLength={8}
            />
          </div>
          {passwordError && <p className="text-sm text-destructive">{passwordError}</p>}
          {passwordSuccess && (
            <p className="text-sm text-accent">
              Password changed. All notes were re-encrypted.
            </p>
          )}
          <Button type="submit" disabled={changingPassword}>
            {changingPassword ? "Changing…" : "Change password"}
          </Button>
        </form>
      </section>

      <section className="space-y-4 rounded-xl border border-border bg-card/50 p-6">
        <h3 className="font-medium">Export backup</h3>
        <p className="text-sm text-muted-foreground">
          Exports decrypted notes as JSON. Store the file securely.
        </p>
        {exportError && <p className="text-sm text-destructive">{exportError}</p>}
        {exportSuccess && <p className="text-sm text-accent">{exportSuccess}</p>}
        <Button variant="secondary" onClick={() => void handleExport()} disabled={exporting}>
          {exporting ? "Exporting…" : "Export backup"}
        </Button>
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
