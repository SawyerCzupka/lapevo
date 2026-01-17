import { useCompareLaps } from '@/api/generated/metrics/metrics';
import type {
  BrakingZoneComparison,
  CornerComparison,
  LapSummary,
  SessionSummary,
} from '@/api/generated/models';
import { useGetSessionDetail, useGetSessionsList } from '@/api/generated/sessions/sessions';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { EmptyState, LoadingState } from '@/components/ui/loading-states';
import { formatDelta, formatLapTime, getDeltaColor } from '@/lib/format';
import { useState } from 'react';
import { useSearchParams } from 'react-router';

export function ComparePage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const [selectedSession, setSelectedSession] = useState<string | null>(null);

  const lap1 = searchParams.get('lap1') ?? '';
  const lap2 = searchParams.get('lap2') ?? '';

  // Fetch sessions for selection
  const { data: sessionsResponse, isLoading: sessionsLoading } = useGetSessionsList();
  const sessions: SessionSummary[] = sessionsResponse?.sessions ?? [];

  // Fetch selected session details to get laps
  const { data: sessionResponse } = useGetSessionDetail(
    selectedSession ?? '',
    { query: { enabled: !!selectedSession } }
  );
  const sessionLaps: LapSummary[] = sessionResponse?.laps ?? [];

  // Fetch comparison data when both laps are selected
  const { data: comparison, isLoading: comparisonLoading, error: comparisonError } =
    useCompareLaps(
      { lap_id_1: lap1, lap_id_2: lap2 },
      { query: { enabled: !!lap1 && !!lap2 } }
    );

  const handleLapSelect = (lapNum: 1 | 2, lapId: string) => {
    const newParams = new URLSearchParams(searchParams);
    newParams.set(`lap${lapNum}`, lapId);
    setSearchParams(newParams);
  };

  // If no laps selected, show the selection UI
  if (!lap1 || !lap2) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Compare Laps</h2>
          <p className="text-muted-foreground">Select two laps to compare their performance</p>
        </div>

        <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
          {/* Baseline Lap Selection */}
          <Card>
            <CardHeader>
              <CardTitle>Baseline Lap</CardTitle>
              <CardDescription>Select the reference lap for comparison</CardDescription>
            </CardHeader>
            <CardContent>
              {lap1 ? (
                <div className="p-4 border rounded-lg bg-teal-500/10 border-teal-500/30">
                  <p className="font-medium text-foreground">Lap Selected</p>
                  <p className="text-sm text-muted-foreground">{lap1}</p>
                  <button
                    onClick={() => handleLapSelect(1, '')}
                    className="mt-2 text-sm text-teal-400 hover:text-teal-300"
                  >
                    Change selection
                  </button>
                </div>
              ) : (
                <LapSelector
                  sessions={sessions}
                  sessionsLoading={sessionsLoading}
                  selectedSession={selectedSession}
                  onSessionSelect={setSelectedSession}
                  sessionLaps={sessionLaps}
                  onLapSelect={(lapId) => handleLapSelect(1, lapId)}
                />
              )}
            </CardContent>
          </Card>

          {/* Comparison Lap Selection */}
          <Card>
            <CardHeader>
              <CardTitle>Comparison Lap</CardTitle>
              <CardDescription>Select the lap to compare against baseline</CardDescription>
            </CardHeader>
            <CardContent>
              {lap2 ? (
                <div className="p-4 border rounded-lg bg-green-500/10 border-green-500/30">
                  <p className="font-medium text-foreground">Lap Selected</p>
                  <p className="text-sm text-muted-foreground">{lap2}</p>
                  <button
                    onClick={() => handleLapSelect(2, '')}
                    className="mt-2 text-sm text-green-400 hover:text-green-300"
                  >
                    Change selection
                  </button>
                </div>
              ) : (
                <LapSelector
                  sessions={sessions}
                  sessionsLoading={sessionsLoading}
                  selectedSession={selectedSession}
                  onSessionSelect={setSelectedSession}
                  sessionLaps={sessionLaps}
                  onLapSelect={(lapId) => handleLapSelect(2, lapId)}
                />
              )}
            </CardContent>
          </Card>
        </div>
      </div>
    );
  }

  // Show comparison results
  if (comparisonLoading) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Compare Laps</h2>
          <p className="text-muted-foreground">Loading comparison data...</p>
        </div>
        <Card>
          <LoadingState message="Comparing laps..." />
        </Card>
      </div>
    );
  }

  if (comparisonError || !comparison) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Compare Laps</h2>
          <p className="text-muted-foreground">Unable to compare laps</p>
        </div>
        <Card>
          <CardContent className="py-8">
            <div className="text-center">
              <p className="mb-4 text-muted-foreground">
                Could not compare these laps. Make sure both laps have metrics computed.
              </p>
              <button
                onClick={() => {
                  setSearchParams(new URLSearchParams());
                }}
                className="text-teal-400 hover:text-teal-300"
              >
                Select different laps
              </button>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  const summary = comparison.summary;
  const brakingComparisons = comparison.braking_zone_comparisons ?? [];
  const cornerComparisons = comparison.corner_comparisons ?? [];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Lap Comparison</h2>
          <p className="text-muted-foreground">
            Baseline vs Comparison lap analysis
          </p>
        </div>
        <button
          onClick={() => setSearchParams(new URLSearchParams())}
          className="text-sm text-muted-foreground hover:text-foreground"
        >
          Compare different laps
        </button>
      </div>

      {/* Summary */}
      <Card>
        <CardHeader>
          <CardTitle>Summary</CardTitle>
          <CardDescription>Overall lap time and performance deltas</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 gap-6 md:grid-cols-3">
            {/* Lap Time Comparison */}
            <div className="p-4 text-center rounded-lg bg-muted/50">
              <p className="mb-2 text-sm text-muted-foreground">Lap Time Delta</p>
              <p className={`text-3xl font-bold ${getDeltaColor(summary.lap_time_delta ?? 0)}`}>
                {summary.lap_time_delta != null ? formatDelta(summary.lap_time_delta) : 'N/A'}
              </p>
              <div className="mt-2 text-sm">
                <span className="text-teal-400">
                  {summary.baseline_lap_time ? formatLapTime(summary.baseline_lap_time) : '--:--.---'}
                </span>
                <span className="mx-2 text-muted-foreground">vs</span>
                <span className="text-green-400">
                  {summary.comparison_lap_time ? formatLapTime(summary.comparison_lap_time) : '--:--.---'}
                </span>
              </div>
            </div>

            {/* Max Speed Delta */}
            <div className="p-4 text-center rounded-lg bg-muted/50">
              <p className="mb-2 text-sm text-muted-foreground">Max Speed Delta</p>
              <p className={`text-3xl font-bold ${getDeltaColor(-(summary.max_speed_delta ?? 0))}`}>
                {summary.max_speed_delta != null ? `${(summary.max_speed_delta * 3.6).toFixed(1)} km/h` : 'N/A'}
              </p>
            </div>

            {/* Corner Speed Delta */}
            <div className="p-4 text-center rounded-lg bg-muted/50">
              <p className="mb-2 text-sm text-muted-foreground">Avg Corner Speed Delta</p>
              <p className={`text-3xl font-bold ${getDeltaColor(-(summary.average_corner_speed_delta ?? 0))}`}>
                {summary.average_corner_speed_delta != null
                  ? `${(summary.average_corner_speed_delta * 3.6).toFixed(1)} km/h`
                  : 'N/A'}
              </p>
            </div>
          </div>

          <div className="flex justify-center gap-8 mt-4 text-sm text-muted-foreground">
            <span>
              Braking Zones: {summary.matched_braking_zones}/{summary.total_braking_zones_baseline} matched
            </span>
            <span>
              Corners: {summary.matched_corners}/{summary.total_corners_baseline} matched
            </span>
          </div>
        </CardContent>
      </Card>

      {/* Braking Zone Comparisons */}
      {brakingComparisons.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Braking Zone Comparisons</CardTitle>
            <CardDescription>Zone-by-zone braking performance deltas</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
              {brakingComparisons.map((zone: BrakingZoneComparison, index: number) => (
                <div
                  key={index}
                  className="p-4 border border-border rounded-lg bg-muted/50"
                >
                  <div className="flex items-center justify-between mb-3">
                    <span className="text-lg font-semibold text-foreground">Zone {zone.zone_index + 1}</span>
                    {zone.matched_zone_index != null ? (
                      <Badge variant="success" className="text-xs">Matched</Badge>
                    ) : (
                      <Badge variant="default" className="text-xs">Unmatched</Badge>
                    )}
                  </div>
                  {zone.matched_zone_index != null ? (
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Entry Speed</span>
                        <span className={getDeltaColor(-(zone.braking_point_speed_delta ?? 0))}>
                          {zone.braking_point_speed_delta != null
                            ? `${(zone.braking_point_speed_delta * 3.6).toFixed(1)} km/h`
                            : '--'}
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Min Speed</span>
                        <span className={getDeltaColor(-(zone.minimum_speed_delta ?? 0))}>
                          {zone.minimum_speed_delta != null
                            ? `${(zone.minimum_speed_delta * 3.6).toFixed(1)} km/h`
                            : '--'}
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Duration</span>
                        <span className={getDeltaColor(zone.braking_duration_delta ?? 0)}>
                          {zone.braking_duration_delta != null
                            ? formatDelta(zone.braking_duration_delta)
                            : '--'}
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Efficiency</span>
                        <span className={getDeltaColor(-(zone.braking_efficiency_delta ?? 0))}>
                          {zone.braking_efficiency_delta != null
                            ? `${(zone.braking_efficiency_delta * 100).toFixed(1)}%`
                            : '--'}
                        </span>
                      </div>
                    </div>
                  ) : (
                    <p className="text-sm text-muted-foreground">No matching zone in comparison lap</p>
                  )}
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Corner Comparisons */}
      {cornerComparisons.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Corner Comparisons</CardTitle>
            <CardDescription>Corner-by-corner performance deltas</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
              {cornerComparisons.map((corner: CornerComparison, index: number) => (
                <div
                  key={index}
                  className="p-4 border border-border rounded-lg bg-muted/50"
                >
                  <div className="flex items-center justify-between mb-3">
                    <span className="text-lg font-semibold text-foreground">Corner {corner.corner_index + 1}</span>
                    {corner.matched_corner_index != null ? (
                      <Badge variant="success" className="text-xs">Matched</Badge>
                    ) : (
                      <Badge variant="default" className="text-xs">Unmatched</Badge>
                    )}
                  </div>
                  {corner.matched_corner_index != null ? (
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Entry Speed</span>
                        <span className={getDeltaColor(-(corner.turn_in_speed_delta ?? 0))}>
                          {corner.turn_in_speed_delta != null
                            ? `${(corner.turn_in_speed_delta * 3.6).toFixed(1)} km/h`
                            : '--'}
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Apex Speed</span>
                        <span className={getDeltaColor(-(corner.apex_speed_delta ?? 0))}>
                          {corner.apex_speed_delta != null
                            ? `${(corner.apex_speed_delta * 3.6).toFixed(1)} km/h`
                            : '--'}
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Exit Speed</span>
                        <span className={getDeltaColor(-(corner.exit_speed_delta ?? 0))}>
                          {corner.exit_speed_delta != null
                            ? `${(corner.exit_speed_delta * 3.6).toFixed(1)} km/h`
                            : '--'}
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Time in Corner</span>
                        <span className={getDeltaColor(corner.time_in_corner_delta ?? 0)}>
                          {corner.time_in_corner_delta != null
                            ? formatDelta(corner.time_in_corner_delta)
                            : '--'}
                        </span>
                      </div>
                    </div>
                  ) : (
                    <p className="text-sm text-muted-foreground">No matching corner in comparison lap</p>
                  )}
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

// Lap selector component
interface LapSelectorProps {
  sessions: SessionSummary[];
  sessionsLoading: boolean;
  selectedSession: string | null;
  onSessionSelect: (sessionId: string) => void;
  sessionLaps: LapSummary[];
  onLapSelect: (lapId: string) => void;
}

function LapSelector({
  sessions,
  sessionsLoading,
  selectedSession,
  onSessionSelect,
  sessionLaps,
  onLapSelect,
}: LapSelectorProps) {
  if (sessionsLoading) {
    return <LoadingState message="Loading sessions..." />;
  }

  if (sessions.length === 0) {
    return <EmptyState message="No sessions available" />;
  }

  return (
    <div className="space-y-4">
      {/* Session Selector */}
      <div>
        <label className="block mb-2 text-sm text-muted-foreground">Select Session</label>
        <select
          value={selectedSession ?? ''}
          onChange={(e) => onSessionSelect(e.target.value)}
          className="w-full px-4 py-2 text-foreground bg-muted border border-border rounded-md focus:outline-none focus:ring-2 focus:ring-teal-500"
        >
          <option value="">Choose a session...</option>
          {sessions.map((session) => (
            <option key={session.session_id} value={session.session_id}>
              {session.track_name} - {session.car_name}
            </option>
          ))}
        </select>
      </div>

      {/* Lap Selector */}
      {selectedSession && sessionLaps.length > 0 && (
        <div>
          <label className="block mb-2 text-sm text-muted-foreground">Select Lap</label>
          <div className="space-y-2 overflow-y-auto max-h-48">
            {sessionLaps.filter(lap => lap.has_metrics).map((lap) => (
              <button
                key={lap.lap_id}
                onClick={() => onLapSelect(lap.lap_id)}
                className="w-full p-3 text-left transition-colors bg-muted border border-border rounded-lg hover:bg-muted/80 hover:border-border"
              >
                <div className="flex items-center justify-between">
                  <span className="font-medium text-foreground">Lap {lap.lap_number}</span>
                  <span className="font-mono text-muted-foreground">
                    {lap.lap_time ? formatLapTime(lap.lap_time) : '--:--.---'}
                  </span>
                </div>
              </button>
            ))}
          </div>
          {sessionLaps.filter(lap => lap.has_metrics).length === 0 && (
            <p className="text-sm text-muted-foreground">No laps with computed metrics in this session</p>
          )}
        </div>
      )}

      {selectedSession && sessionLaps.length === 0 && (
        <p className="text-sm text-muted-foreground">No laps recorded in this session</p>
      )}
    </div>
  );
}
