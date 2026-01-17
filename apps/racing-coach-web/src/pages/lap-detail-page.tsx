import { useGetLapMetrics } from '@/api/generated/metrics/metrics';
import type { BrakingMetrics, CornerMetrics } from '@/api/generated/models';
import { Badge } from '@/components/ui/badge';
import { Breadcrumbs } from '@/components/ui/breadcrumbs';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { EmptyState, LoadingState } from '@/components/ui/loading-states';
import { formatDistance, formatGForce, formatLapTime, formatSpeed } from '@/lib/format';
import { useParams } from 'react-router';

export function LapDetailPage() {
  const { lapId } = useParams<{ lapId: string }>();
  const { data: metrics, isLoading, error } = useGetLapMetrics(
    lapId ?? ''
  );

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Lap Details</h2>
          <p className="text-muted-foreground">Loading lap information...</p>
        </div>
        <Card>
          <LoadingState message="Loading lap metrics..." />
        </Card>
      </div>
    );
  }

  if (error) {
    return (
      <div className="space-y-6">
        <div>
          <Breadcrumbs
            items={[
              { label: 'Dashboard', href: '/dashboard' },
              { label: 'Sessions', href: '/sessions' },
              { label: 'Lap Analysis' },
            ]}
            className="mb-3"
          />
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Lap Details</h2>
          <p className="text-muted-foreground">No metrics available for this lap</p>
        </div>
        <Card>
          <CardContent className="py-8">
            <div className="text-center">
              <p className="mb-4 text-muted-foreground">
                Metrics have not been computed for this lap yet.
              </p>
              <p className="text-sm text-muted-foreground">
                Lap metrics are calculated after telemetry data is uploaded and processed.
              </p>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (!metrics) {
    return (
      <div className="space-y-6">
        <div>
          <Breadcrumbs
            items={[
              { label: 'Dashboard', href: '/dashboard' },
              { label: 'Sessions', href: '/sessions' },
              { label: 'Lap Analysis' },
            ]}
            className="mb-3"
          />
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Lap Details</h2>
          <p className="text-muted-foreground">Lap not found</p>
        </div>
        <Card>
          <EmptyState message="Lap not found" />
        </Card>
      </div>
    );
  }

  const brakingZones = metrics.braking_zones ?? [];
  const corners = metrics.corners ?? [];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <Breadcrumbs
            items={[
              { label: 'Dashboard', href: '/dashboard' },
              { label: 'Sessions', href: '/sessions' },
              { label: 'Lap Analysis' },
            ]}
            className="mb-3"
          />
          <h2 className="text-3xl font-bold tracking-tight text-foreground">
            Lap Analysis
          </h2>
          <p className="text-xl text-muted-foreground">
            {metrics.lap_time ? formatLapTime(metrics.lap_time) : 'Time not recorded'}
          </p>
        </div>
        <Badge variant="info">
          {corners.length} Corners | {brakingZones.length} Braking Zones
        </Badge>
      </div>

      {/* Summary Stats */}
      <div className="grid grid-cols-1 gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Max Speed</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold text-foreground">{formatSpeed(metrics.max_speed)}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Min Speed</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold text-foreground">{formatSpeed(metrics.min_speed)}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Avg Corner Speed</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold text-foreground">{formatSpeed(metrics.average_corner_speed)}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Lap Time</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="font-mono text-2xl font-bold text-foreground">
              {metrics.lap_time ? formatLapTime(metrics.lap_time) : '--:--.---'}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Braking Zones */}
      <Card>
        <CardHeader>
          <CardTitle>Braking Zones</CardTitle>
          <CardDescription>Analysis of each braking zone in the lap</CardDescription>
        </CardHeader>
        <CardContent>
          {brakingZones.length === 0 ? (
            <EmptyState message="No braking zones detected" />
          ) : (
            <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
              {brakingZones.map((zone: BrakingMetrics, index: number) => (
                <div
                  key={index}
                  className="p-4 border border-border rounded-lg bg-muted/50"
                >
                  <div className="flex items-center justify-between mb-3">
                    <span className="text-lg font-semibold text-foreground">Zone {index + 1}</span>
                    {zone.has_trail_braking && (
                      <Badge variant="success" className="text-xs">Trail Braking</Badge>
                    )}
                  </div>
                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Entry Speed</span>
                      <span className="text-foreground">{formatSpeed(zone.braking_point_speed)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Min Speed</span>
                      <span className="text-foreground">{formatSpeed(zone.minimum_speed)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Max Brake</span>
                      <span className="text-foreground">{(zone.max_brake_pressure * 100).toFixed(0)}%</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Duration</span>
                      <span className="text-white">{zone.braking_duration.toFixed(2)}s</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Efficiency</span>
                      <span className="text-foreground">{(zone.braking_efficiency * 100).toFixed(1)}%</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Distance</span>
                      <span className="text-foreground">{formatDistance(zone.braking_point_distance)}</span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Corners */}
      <Card>
        <CardHeader>
          <CardTitle>Corner Analysis</CardTitle>
          <CardDescription>Performance metrics for each corner</CardDescription>
        </CardHeader>
        <CardContent>
          {corners.length === 0 ? (
            <EmptyState message="No corners detected" />
          ) : (
            <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
              {corners.map((corner: CornerMetrics, index: number) => (
                <div
                  key={index}
                  className="p-4 border border-border rounded-lg bg-muted/50"
                >
                  <div className="flex items-center justify-between mb-3">
                    <span className="text-lg font-semibold text-foreground">Corner {index + 1}</span>
                    <Badge variant="default" className="text-xs">
                      {formatGForce(corner.max_lateral_g)}
                    </Badge>
                  </div>
                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Entry Speed</span>
                      <span className="text-foreground">{formatSpeed(corner.turn_in_speed)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Apex Speed</span>
                      <span className="text-foreground">{formatSpeed(corner.apex_speed)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Exit Speed</span>
                      <span className="text-foreground">{formatSpeed(corner.exit_speed)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Time in Corner</span>
                      <span className="text-foreground">{corner.time_in_corner.toFixed(2)}s</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Speed Lost</span>
                      <span className="text-red-400">{formatSpeed(corner.speed_loss)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Speed Gained</span>
                      <span className="text-green-400">{formatSpeed(corner.speed_gain)}</span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
