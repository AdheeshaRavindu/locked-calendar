export function formatUserError(err: unknown, fallback = "Something went wrong. Please try again."): string {
  const raw =
    err instanceof Error
      ? err.message
      : typeof err === "string"
        ? err
        : String(err);

  const msg = raw.replace(/^Error:\s*/i, "").trim();

  if (/GOOGLE_OAUTH_CLIENT_ID|Google OAuth is not configured/i.test(msg)) {
    return "Cloud sync is not configured yet. The app publisher must enable Google OAuth before you can connect your account.";
  }
  if (/Incorrect password/i.test(msg)) {
    return "Incorrect password.";
  }
  if (/Invalid password/i.test(msg)) {
    return "Incorrect password.";
  }
  if (/Please unlock/i.test(msg)) {
    return "Unlock the app to continue.";
  }
  if (/Google Drive is not connected/i.test(msg)) {
    return "Connect Google Drive first.";
  }
  if (/changed on another device/i.test(msg)) {
    return "Sync conflict detected. Try syncing again to merge changes.";
  }
  if (/Sync is already in progress/i.test(msg)) {
    return "Sync is already running.";
  }
  if (/Could not open browser/i.test(msg)) {
    return "Could not open the browser for sign-in.";
  }

  return msg || fallback;
}
