export function LivePage() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold tracking-tight text-foreground">Live Session</h2>
        <p className="text-muted-foreground">
          Real-time telemetry and lap monitoring
        </p>
      </div>

      <div className="flex items-center justify-center rounded-lg border border-border bg-card p-8">
        <div className="text-center space-y-2">
          <div className="inline-block h-4 w-4 rounded-full bg-muted animate-pulse" />
          <p className="text-muted-foreground">
            Waiting for active session...
          </p>
        </div>
      </div>
    </div>
  );
}
