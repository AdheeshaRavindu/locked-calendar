import { useState } from "react";
import { Lock } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useSession } from "@/hooks/useSession";
import { cn } from "@/lib/utils";

export function LockScreen() {
  const { unlock } = useSession();
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [shake, setShake] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setLoading(true);
    try {
      await unlock(password);
      setPassword("");
    } catch {
      setError("Incorrect password. Try again.");
      setShake(true);
      setTimeout(() => setShake(false), 500);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex min-h-full items-center justify-center bg-background p-8">
      <Card variant="elevated" className={cn("w-full max-w-md", shake && "animate-shake")}>
        <CardHeader className="items-center pb-2 text-center">
          <div className="mb-4 flex h-14 w-14 items-center justify-center rounded-2xl bg-accent/12 text-accent">
            <Lock className="h-7 w-7" />
          </div>
          <CardTitle className="text-2xl">Locked Calendar</CardTitle>
          <CardDescription className="max-w-xs">
            Enter your master password to unlock your journal.
          </CardDescription>
        </CardHeader>
        <CardContent className="pt-2">
          <form onSubmit={handleSubmit} className="space-y-5">
            <div className="space-y-2">
              <Label htmlFor="password">Master password</Label>
              <Input
                id="password"
                type="password"
                autoFocus
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="••••••••"
              />
            </div>
            {error && <p className="text-sm text-destructive">{error}</p>}
            <Button type="submit" className="w-full" size="lg" disabled={loading || !password}>
              {loading ? "Unlocking…" : "Unlock"}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
