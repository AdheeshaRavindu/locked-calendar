import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { Loader2 } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { LoadingState } from "@/components/ui/loading-state";
import { api, type SyncStatus } from "@/lib/invoke";
import { formatRelativeTime } from "@/lib/dates";
import { formatUserError } from "@/lib/errors";
import { useSession } from "@/hooks/useSession";

function syncStatusBadge(status: SyncStatus | null) {
  if (!status) return { label: "Loading…", variant: "muted" as const };
  if (status.in_progress) return { label: "Syncing", variant: "accent" as const };
  if (status.connected) return { label: "Connected", variant: "success" as const };
  return { label: "Not connected", variant: "muted" as const };
}

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
  const [syncStatus, setSyncStatus] = useState<SyncStatus | null>(null);
  const [syncBusy, setSyncBusy] = useState(false);
  const [syncSuccess, setSyncSuccess] = useState<string | null>(null);
  const [syncError, setSyncError] = useState<string | null>(null);
  const [disconnectOpen, setDisconnectOpen] = useState(false);

  const refreshSyncStatus = useCallback(async () => {
    const status = await api.syncStatus();
    setSyncStatus(status);
  }, []);

  useEffect(() => {
    void api.authGetLockTimeout().then((secs) => {
      setTimeoutMinutes(Math.round(secs / 60));
    });
    void refreshSyncStatus();
  }, [refreshSyncStatus]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<SyncStatus>("sync-status-changed", (event) => {
      setSyncStatus(event.payload);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
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
      setPasswordError(formatUserError(err, "Could not change password."));
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
      setExportError(formatUserError(err, "Export failed."));
    } finally {
      setExporting(false);
    }
  }

  function clearSyncFeedback() {
    setSyncSuccess(null);
    setSyncError(null);
  }

  async function handleSyncConnect() {
    clearSyncFeedback();
    setSyncBusy(true);
    try {
      await api.syncConnect();
      await refreshSyncStatus();
      setSyncSuccess("Connected to Google Drive.");
    } catch (err) {
      setSyncError(formatUserError(err, "Could not connect to Google Drive."));
    } finally {
      setSyncBusy(false);
    }
  }

  async function handleSyncDisconnect() {
    clearSyncFeedback();
    setSyncBusy(true);
    try {
      await api.syncDisconnect();
      await refreshSyncStatus();
      setSyncSuccess("Disconnected from Google Drive.");
      setDisconnectOpen(false);
    } catch (err) {
      setSyncError(formatUserError(err, "Could not disconnect."));
    } finally {
      setSyncBusy(false);
    }
  }

  async function handleSyncNow() {
    clearSyncFeedback();
    setSyncBusy(true);
    try {
      const result = await api.syncNow();
      await refreshSyncStatus();
      setSyncSuccess(
        `Synced ${formatRelativeTime(result.last_sync_at)}. Applied ${result.merged.notes_applied} remote update(s).`,
      );
    } catch (err) {
      setSyncError(formatUserError(err, "Sync failed."));
    } finally {
      setSyncBusy(false);
    }
  }

  const badge = syncStatusBadge(syncStatus);

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
        <div className="flex flex-wrap items-center gap-2">
          <h3 className="font-medium">Cloud sync</h3>
          <Badge variant={badge.variant}>{badge.label}</Badge>
          {syncStatus?.in_progress && (
            <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
          )}
        </div>
        <p className="text-sm text-muted-foreground">
          Connect your Google account to sync encrypted notes to your Drive. Use the same master
          password on every device.
        </p>
        {!syncStatus ? (
          <LoadingState label="Loading sync status…" className="py-6" />
        ) : (
          <>
            {syncStatus.connected && syncStatus.last_sync_at && (
              <p className="text-sm text-muted-foreground">
                Last sync: {formatRelativeTime(syncStatus.last_sync_at)}
                <span className="ml-1 text-xs">
                  ({new Date(syncStatus.last_sync_at).toLocaleString()})
                </span>
              </p>
            )}
            {syncStatus.error && (
              <p className="text-sm text-destructive">
                {formatUserError(syncStatus.error)}
              </p>
            )}
            {syncSuccess && <p className="text-sm text-accent">{syncSuccess}</p>}
            {syncError && <p className="text-sm text-destructive">{syncError}</p>}
            <div className="flex flex-wrap gap-2">
              {!syncStatus.connected ? (
                <Button onClick={() => void handleSyncConnect()} disabled={syncBusy}>
                  {syncBusy ? "Connecting…" : "Connect Google account"}
                </Button>
              ) : (
                <>
                  <Button
                    onClick={() => void handleSyncNow()}
                    disabled={syncBusy || syncStatus.in_progress}
                  >
                    {syncBusy || syncStatus.in_progress ? "Syncing…" : "Sync now"}
                  </Button>
                  <Button
                    variant="secondary"
                    onClick={() => setDisconnectOpen(true)}
                    disabled={syncBusy}
                  >
                    Disconnect
                  </Button>
                </>
              )}
            </div>
          </>
        )}
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

      <Dialog open={disconnectOpen} onOpenChange={setDisconnectOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Disconnect Google account?</DialogTitle>
            <DialogDescription>
              Sync tokens will be removed from this device. Your journal on Google Drive is not
              deleted.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="secondary" onClick={() => setDisconnectOpen(false)}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={() => void handleSyncDisconnect()}>
              Disconnect
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
