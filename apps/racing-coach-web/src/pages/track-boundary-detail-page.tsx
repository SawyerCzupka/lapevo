import { useDeleteTrackBoundary, useGetTrackBoundary } from '@/api/generated/tracks/tracks';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ErrorState, LoadingState } from '@/components/ui/loading-states';
import { formatDateTime } from '@/lib/format';
import { useQueryClient } from '@tanstack/react-query';
import { useMemo } from 'react';
import Plot from 'react-plotly.js';
import { useNavigate, useParams } from 'react-router';

export function TrackBoundaryDetailPage() {
  const { boundaryId } = useParams<{ boundaryId: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const { data: boundary, isLoading, error } = useGetTrackBoundary(boundaryId || '');
  const deleteMutation = useDeleteTrackBoundary();

  // Calculate centerline and plot data
  const plotData = useMemo(() => {
    if (!boundary) return null;

    // Calculate centerline as average of left and right
    const centerLat = boundary.left_latitude.map(
      (lat, i) => (lat + boundary.right_latitude[i]) / 2
    );
    const centerLon = boundary.left_longitude.map(
      (lon, i) => (lon + boundary.right_longitude[i]) / 2
    );

    // Start/Finish line position (first point)
    const sfLat = (boundary.left_latitude[0] + boundary.right_latitude[0]) / 2;
    const sfLon = (boundary.left_longitude[0] + boundary.right_longitude[0]) / 2;

    // Start/Finish line markers (perpendicular line across track)
    const sfLeftLat = boundary.left_latitude[0];
    const sfLeftLon = boundary.left_longitude[0];
    const sfRightLat = boundary.right_latitude[0];
    const sfRightLon = boundary.right_longitude[0];

    return {
      traces: [
        // Left boundary
        {
          x: boundary.left_longitude,
          y: boundary.left_latitude,
          type: 'scatter' as const,
          mode: 'lines' as const,
          line: { color: '#ef4444', width: 2 },
          name: 'Left Boundary',
          hovertemplate: 'Left: %{y:.6f}, %{x:.6f}<extra></extra>',
        },
        // Right boundary
        {
          x: boundary.right_longitude,
          y: boundary.right_latitude,
          type: 'scatter' as const,
          mode: 'lines' as const,
          line: { color: '#22c55e', width: 2 },
          name: 'Right Boundary',
          hovertemplate: 'Right: %{y:.6f}, %{x:.6f}<extra></extra>',
        },
        // Centerline
        {
          x: centerLon,
          y: centerLat,
          type: 'scatter' as const,
          mode: 'lines' as const,
          line: { color: '#6b7280', width: 1, dash: 'dash' as const },
          name: 'Centerline',
          hovertemplate: 'Center: %{y:.6f}, %{x:.6f}<extra></extra>',
        },
        // Start/Finish line (as a line segment)
        {
          x: [sfLeftLon, sfRightLon],
          y: [sfLeftLat, sfRightLat],
          type: 'scatter' as const,
          mode: 'lines' as const,
          line: { color: '#fbbf24', width: 3 },
          name: 'Start/Finish Line',
          hoverinfo: 'skip' as const,
        },
        // Start/Finish marker
        {
          x: [sfLon],
          y: [sfLat],
          type: 'scatter' as const,
          mode: 'markers+text' as const,
          marker: { size: 14, color: '#fbbf24', symbol: 'star' },
          text: ['S/F'],
          textposition: 'top center' as const,
          textfont: { color: '#ffffff', size: 12 },
          name: 'Start/Finish',
          hovertemplate: 'Start/Finish<extra></extra>',
        },
      ],
      layout: {
        uirevision: 'preserve-zoom',
        height: 700,
        paper_bgcolor: 'rgba(0,0,0,0)',
        plot_bgcolor: 'rgba(17, 24, 39, 0.5)',
        font: {
          color: '#ffffff',
          family: 'system-ui, -apple-system, sans-serif',
        },
        xaxis: {
          uirevision: 'preserve-zoom',
          showticklabels: false,
          showgrid: false,
          zeroline: false,
          title: '',
        },
        yaxis: {
          uirevision: 'preserve-zoom',
          showticklabels: false,
          showgrid: false,
          zeroline: false,
          scaleanchor: 'x',
          title: '',
        },
        legend: {
          x: 0,
          y: 1,
          bgcolor: 'rgba(0,0,0,0.7)',
          font: { color: '#ffffff' },
        },
        margin: { t: 20, b: 20, l: 20, r: 20 },
        hovermode: 'closest' as const,
        dragmode: 'pan' as const,
      },
    };
  }, [boundary]);

  const handleDelete = async () => {
    if (!boundaryId) return;

    if (window.confirm('Are you sure you want to delete this track boundary?')) {
      try {
        await deleteMutation.mutateAsync({ boundaryId });
        queryClient.invalidateQueries({ queryKey: ['/api/v1/tracks'] });
        navigate('/tracks');
      } catch (err) {
        console.error('Failed to delete track boundary:', err);
      }
    }
  };

  if (isLoading) {
    return (
      <div className="space-y-6">
        <Card>
          <LoadingState message="Loading track boundary..." />
        </Card>
      </div>
    );
  }

  if (error || !boundary) {
    return (
      <div className="space-y-6">
        <Card>
          <ErrorState
            error={error instanceof Error ? error : new Error('Track boundary not found')}
          />
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <button
            onClick={() => navigate('/tracks')}
            className="text-sm text-muted-foreground hover:text-foreground mb-2 flex items-center gap-1"
          >
            &larr; Back to Track Boundaries
          </button>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">{boundary.track_name}</h2>
          {boundary.track_config_name && (
            <p className="text-xl text-muted-foreground">{boundary.track_config_name}</p>
          )}
        </div>
        <div className="flex gap-4 items-center">
          <Badge variant="info">{boundary.grid_size} grid points</Badge>
          <Button
            variant="outline"
            onClick={() => navigate(`/tracks/${boundaryId}/corners`)}
          >
            Edit Corners
          </Button>
          <Button
            variant="outline"
            onClick={handleDelete}
            disabled={deleteMutation.isPending}
          >
            {deleteMutation.isPending ? 'Deleting...' : 'Delete'}
          </Button>
        </div>
      </div>

      {/* Metadata Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Track ID</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-semibold text-foreground">{boundary.track_id}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Left Boundary Frames</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-semibold text-foreground">{boundary.source_left_frames}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Right Boundary Frames
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-semibold text-foreground">{boundary.source_right_frames}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Created</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-lg font-semibold text-foreground">{formatDateTime(boundary.created_at)}</p>
          </CardContent>
        </Card>
      </div>

      {/* Track Map */}
      <Card>
        <CardHeader>
          <CardTitle>Track Boundary Map</CardTitle>
        </CardHeader>
        <CardContent>
          {plotData && (
            <Plot
              data={plotData.traces as any}
              layout={plotData.layout as any}
              config={{
                responsive: true,
                displayModeBar: true,
                displaylogo: false,
                modeBarButtonsToRemove: ['lasso2d', 'select2d'],
                scrollZoom: true,
              }}
              className="w-full"
            />
          )}
        </CardContent>
      </Card>
    </div>
  );
}
