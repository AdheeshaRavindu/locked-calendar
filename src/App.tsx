import { BrowserRouter } from "react-router-dom";
import { FocusModeProvider } from "@/app/layout/FocusModeContext";
import { SessionProvider, useSession } from "@/hooks/useSession";
import { LockScreen } from "@/features/lock/LockScreen";
import { SetupPassword } from "@/features/lock/SetupPassword";
import { AppRoutes } from "@/app/routes";

function AuthGate() {
  const { status, loading } = useSession();

  if (loading || !status) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        Loading…
      </div>
    );
  }

  if (!status.initialized) {
    return <SetupPassword />;
  }

  if (!status.unlocked) {
    return <LockScreen />;
  }

  return <AppRoutes />;
}

export default function App() {
  return (
    <BrowserRouter>
      <SessionProvider>
        <FocusModeProvider>
          <AuthGate />
        </FocusModeProvider>
      </SessionProvider>
    </BrowserRouter>
  );
}
