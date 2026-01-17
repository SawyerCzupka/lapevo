import type { CornerSegmentCreate, CornerSegmentResponse } from '@/api/generated/models';
import {
  useGetLapTelemetry,
  useGetSessionDetail,
  useGetSessionsList,
} from '@/api/generated/sessions/sessions';
import {
  useCreateCornerSegments,
  useGetTrackBoundary,
  useListCornerSegments,
} from '@/api/generated/tracks/tracks';
import { CornerSegmentList } from '@/components/tracks/corner-segment-list';
import { TrackMapWithCorners } from '@/components/tracks/track-map-with-corners';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ErrorState, LoadingState } from '@/components/ui/loading-states';
import { useCallback, useMemo, useState } from 'react';
import { useNavigate, useParams } from 'react-router';


/**
 * Draft corner state for editing before saving.
 */
export interface CornerDraft {
  id: string | null; // null for new corners, string UUID for existing
  start_distance: number;
  end_distance: number;
  isNew: boolean;
  isModified: boolean;
}

/**
 * Convert server corners to draft format.
 */
function cornersToDrafts(corners: CornerSegmentResponse[]): CornerDraft[] {
  return corners.map((c) => ({
    id: c.id,
    start_distance: c.start_distance,
    end_distance: c.end_distance,
    isNew: false,
    isModified: false,
  }));
}

/**
 * Check if two corner segments overlap.
 */
function cornersOverlap(a: CornerDraft, b: CornerDraft): boolean {
  return a.start_distance < b.end_distance && b.start_distance < a.end_distance;
}

/**
 * Find overlapping corners (returns pairs of indices).
 */
function findOverlappingCorners(corners: CornerDraft[]): [number, number][] {
  const overlaps: [number, number][] = [];
  for (let i = 0; i < corners.length; i++) {
    for (let j = i + 1; j < corners.length; j++) {
      if (cornersOverlap(corners[i], corners[j])) {
        overlaps.push([i, j]);
      }
    }
  }
  return overlaps;
}

export function CornerSegmentEditorPage() {
  const { boundaryId } = useParams<{ boundaryId: string }>();
  const navigate = useNavigate();

  // Server state
  const {
    data: boundary,
    isLoading: boundaryLoading,
    error: boundaryError,
  } = useGetTrackBoundary(boundaryId || '');

  const {
    data: serverCorners,
    isLoading: cornersLoading,
    error: cornersError,
  } = useListCornerSegments(boundaryId || '');

  const createCornersMutation = useCreateCornerSegments();

  // Local draft state
  const [corners, setCorners] = useState<CornerDraft[]>([]);
  const [initialized, setInitialized] = useState(false);
  const [selectedCornerId, setSelectedCornerId] = useState<string | null>(null);

  // Lap overlay state
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);
  const [selectedLapId, setSelectedLapId] = useState<string | null>(null);

  // Sessions for lap selection
  const { data: sessions } = useGetSessionsList();
  const trackSessions = useMemo(() => {
    if (!sessions || !boundary) return [];
    return sessions.sessions.filter((s) => s.track_id === boundary.track_id);
  }, [sessions, boundary]);

  // Fetch session details (including laps) when a session is selected
  const { data: selectedSessionDetail } = useGetSessionDetail(
    selectedSessionId || '',
    { query: { enabled: !!selectedSessionId } }
  );

  // Lap telemetry for overlay
  const { data: lapTelemetry } = useGetLapTelemetry(
    selectedSessionId || '',
    selectedLapId || '',
    { query: { enabled: !!selectedSessionId && !!selectedLapId } }
  );

  // Initialize corners from server data
  if (!initialized && serverCorners && !cornersLoading) {
    setCorners(cornersToDrafts(serverCorners.corners));
    setInitialized(true);
  }

  // Computed state
  const hasUnsavedChanges = useMemo(
    () => corners.some((c) => c.isNew || c.isModified),
    [corners]
  );

  const overlappingCorners = useMemo(() => findOverlappingCorners(corners), [corners]);
  const hasOverlaps = overlappingCorners.length > 0;

  // Handlers
  const handleAddCorner = useCallback(
    (startDistance: number, endDistance: number) => {
      const newCorner: CornerDraft = {
        id: `new-${Date.now()}`,
        start_distance: Math.min(startDistance, endDistance),
        end_distance: Math.max(startDistance, endDistance),
        isNew: true,
        isModified: false,
      };
      setCorners((prev) => {
        // Sort by start_distance after adding
        const updated = [...prev, newCorner].sort(
          (a, b) => a.start_distance - b.start_distance
        );
        return updated;
      });
      setSelectedCornerId(newCorner.id);
    },
    []
  );

  const handleUpdateCorner = useCallback(
    (id: string, startDistance: number, endDistance: number) => {
      setCorners((prev) =>
        prev
          .map((c) =>
            c.id === id
              ? {
                  ...c,
                  start_distance: Math.min(startDistance, endDistance),
                  end_distance: Math.max(startDistance, endDistance),
                  isModified: !c.isNew,
                }
              : c
          )
          .sort((a, b) => a.start_distance - b.start_distance)
      );
    },
    []
  );

  const handleDeleteCorner = useCallback((id: string) => {
    setCorners((prev) => prev.filter((c) => c.id !== id));
    setSelectedCornerId(null);
  }, []);

  const handleSelectCorner = useCallback((id: string | null) => {
    setSelectedCornerId(id);
  }, []);

  const handleReset = useCallback(() => {
    if (serverCorners) {
      setCorners(cornersToDrafts(serverCorners.corners));
      setSelectedCornerId(null);
    }
  }, [serverCorners]);

  const handleSave = useCallback(async () => {
    if (!boundaryId) return;

    const cornerData: CornerSegmentCreate[] = corners.map((c) => ({
      start_distance: c.start_distance,
      end_distance: c.end_distance,
    }));

    try {
      await createCornersMutation.mutateAsync({
        boundaryId,
        data: { corners: cornerData },
      });
      // Update local state to reflect saved state
      setCorners((prev) =>
        prev.map((c) => ({ ...c, isNew: false, isModified: false }))
      );
    } catch (err) {
      console.error('Failed to save corner segments:', err);
    }
  }, [boundaryId, corners, createCornersMutation]);

  // Loading/error states
  if (boundaryLoading || cornersLoading) {
    return (
      <div className="space-y-6">
        <Card>
          <LoadingState message="Loading track boundary..." />
        </Card>
      </div>
    );
  }

  if (boundaryError || !boundary) {
    return (
      <div className="space-y-6">
        <Card>
          <ErrorState
            error={
              boundaryError instanceof Error
                ? boundaryError
                : new Error('Track boundary not found')
            }
          />
        </Card>
      </div>
    );
  }

  if (cornersError) {
    return (
      <div className="space-y-6">
        <Card>
          <ErrorState
            error={
              cornersError instanceof Error
                ? cornersError
                : new Error('Failed to load corner segments')
            }
          />
        </Card>
      </div>
    );
  }

  const trackLength = boundary.track_length ?? 0;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <button
            onClick={() => navigate(`/tracks/${boundaryId}`)}
            className="text-sm text-muted-foreground hover:text-foreground mb-2 flex items-center gap-1"
          >
            &larr; Back to Track Boundary
          </button>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">
            Corner Segment Editor
          </h2>
          <p className="text-xl text-muted-foreground">
            {boundary.track_name}
            {boundary.track_config_name && ` - ${boundary.track_config_name}`}
          </p>
        </div>
        <div className="flex gap-4 items-center">
          {trackLength > 0 && (
            <Badge variant="info">{(trackLength / 1000).toFixed(2)} km</Badge>
          )}
          <Badge variant={hasUnsavedChanges ? 'danger' : 'default'}>
            {hasUnsavedChanges ? 'Unsaved Changes' : 'Saved'}
          </Badge>
          <Button
            variant="outline"
            onClick={handleReset}
            disabled={!hasUnsavedChanges}
          >
            Reset
          </Button>
          <Button
            onClick={handleSave}
            disabled={!hasUnsavedChanges || createCornersMutation.isPending}
          >
            {createCornersMutation.isPending ? 'Saving...' : 'Save All'}
          </Button>
        </div>
      </div>

      {/* Overlap warning */}
      {hasOverlaps && (
        <Card className="border-yellow-600 bg-yellow-900/20">
          <CardContent className="py-3">
            <p className="text-yellow-400 text-sm">
              Warning: {overlappingCorners.length} overlapping corner pair(s) detected.
              Overlaps are allowed but may affect analysis accuracy.
            </p>
          </CardContent>
        </Card>
      )}

      {/* Main content */}
      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        {/* Track map (3/4 width on large screens) */}
        <div className="lg:col-span-3">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center justify-between">
                <span>Track Map</span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              {/* Usage instructions */}
              <div className="bg-muted/50 border border-border rounded-lg p-3 mb-4">
                <p className="text-muted-foreground text-sm">
                  <strong>Click anywhere on the track centerline</strong> to add a
                  new corner. Fine-tune distances in the sidebar. Scroll to zoom,
                  drag to pan.
                </p>
              </div>

              {/* Lap overlay selector */}
              <div className="flex gap-4 mb-4">
                <div className="flex-1">
                  <label className="block text-sm text-muted-foreground mb-1">
                    Reference Lap (optional)
                  </label>
                  <div className="flex gap-2">
                    <select
                      className="flex-1 bg-muted border border-border rounded px-3 py-2 text-foreground text-sm"
                      value={selectedSessionId || ''}
                      onChange={(e) => {
                        setSelectedSessionId(e.target.value || null);
                        setSelectedLapId(null);
                      }}
                    >
                      <option value="">Select session...</option>
                      {trackSessions.map((s) => (
                        <option key={s.session_id} value={s.session_id}>
                          {s.car_name} - {new Date(s.created_at).toLocaleDateString()}
                        </option>
                      ))}
                    </select>
                    {selectedSessionId && (
                      <select
                        className="w-32 bg-muted border border-border rounded px-3 py-2 text-foreground text-sm"
                        value={selectedLapId || ''}
                        onChange={(e) => setSelectedLapId(e.target.value || null)}
                      >
                        <option value="">Lap...</option>
                        {selectedSessionDetail?.laps.map((lap) => (
                          <option key={lap.lap_id} value={lap.lap_id}>
                            Lap {lap.lap_number}
                          </option>
                        ))}
                      </select>
                    )}
                  </div>
                </div>
              </div>

              <TrackMapWithCorners
                boundary={boundary}
                corners={corners}
                selectedCornerId={selectedCornerId}
                lapTelemetry={lapTelemetry}
                onAddCorner={handleAddCorner}
                onSelectCorner={handleSelectCorner}
              />
            </CardContent>
          </Card>
        </div>

        {/* Corner list sidebar (1/4 width) */}
        <div className="lg:col-span-1">
          <Card className="sticky top-4">
            <CardHeader>
              <CardTitle className="flex items-center justify-between">
                <span>Corners ({corners.length})</span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <CornerSegmentList
                corners={corners}
                selectedCornerId={selectedCornerId}
                overlappingPairs={overlappingCorners}
                onSelectCorner={handleSelectCorner}
                onDeleteCorner={handleDeleteCorner}
                onUpdateCorner={handleUpdateCorner}
                trackLength={trackLength}
              />
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
