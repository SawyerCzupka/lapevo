import { useMemo, useCallback, useRef } from 'react';
import Plot from 'react-plotly.js';
import type { TrackBoundaryResponse, LapTelemetryResponse } from '@/api/generated/models';
import type { CornerDraft } from '@/pages/corner-segment-editor-page';
import { distanceToGps, generateCornerPolygon, gpsToDistance } from '@/lib/track-utils';
import { plotLayout } from '@/lib/chart-colors';

interface TrackMapWithCornersProps {
  boundary: TrackBoundaryResponse;
  corners: CornerDraft[];
  selectedCornerId: string | null;
  lapTelemetry?: LapTelemetryResponse;
  onAddCorner: (startDistance: number, endDistance: number) => void;
  onSelectCorner: (id: string | null) => void;
}

// Teal-based color palette for corners
const CORNER_COLORS = [
  '#14b8a6', // teal-500
  '#2dd4bf', // teal-400
  '#5eead4', // teal-300
  '#99f6e4', // teal-200
  '#0d9488', // teal-600
  '#0f766e', // teal-700
  '#06b6d4', // cyan-500
  '#22d3ee', // cyan-400
];

export function TrackMapWithCorners({
  boundary,
  corners,
  selectedCornerId,
  lapTelemetry,
  onAddCorner,
  onSelectCorner,
}: TrackMapWithCornersProps) {
  const trackLength = boundary.track_length ?? 0;
  const plotRef = useRef<any>(null);

  // Calculate centerline
  const { centerLat, centerLon } = useMemo(() => {
    const lat = boundary.left_latitude.map(
      (l: number, i: number) => (l + boundary.right_latitude[i]) / 2
    );
    const lon = boundary.left_longitude.map(
      (l: number, i: number) => (l + boundary.right_longitude[i]) / 2
    );
    return { centerLat: lat, centerLon: lon };
  }, [boundary]);

  // Build plot traces
  const { traces, shapes } = useMemo(() => {
    const traceList: any[] = [];

    // Left boundary
    traceList.push({
      x: boundary.left_longitude,
      y: boundary.left_latitude,
      type: 'scatter' as const,
      mode: 'lines' as const,
      line: { color: '#ef4444', width: 2 },
      name: 'Left Boundary',
      hoverinfo: 'skip' as const,
    });

    // Right boundary
    traceList.push({
      x: boundary.right_longitude,
      y: boundary.right_latitude,
      type: 'scatter' as const,
      mode: 'lines' as const,
      line: { color: '#22c55e', width: 2 },
      name: 'Right Boundary',
      hoverinfo: 'skip' as const,
    });

    // Centerline visual (lines only, no interaction)
    traceList.push({
      x: centerLon,
      y: centerLat,
      type: 'scatter' as const,
      mode: 'lines' as const,
      line: { color: '#6b7280', width: 4, dash: 'dash' as const },
      name: 'Centerline',
      hoverinfo: 'skip' as const,
      opacity: 0.5,
    });

    // Invisible click targets at every grid point for precise corner placement
    traceList.push({
      x: centerLon,
      y: centerLat,
      type: 'scatter' as const,
      mode: 'markers' as const,
      marker: {
        size: 16, // Large hit area for easy clicking
        color: 'rgba(107, 114, 128, 0.1)', // Nearly invisible
        symbol: 'circle' as const,
      },
      name: 'Click to add corner',
      hovertemplate: 'Click to add corner<extra></extra>',
      hoverlabel: {
        bgcolor: '#1f2937',
        font: { color: '#ffffff', size: 12 },
      },
      showlegend: false,
    });

    // Start/Finish line
    const sfLeftLat = boundary.left_latitude[0];
    const sfLeftLon = boundary.left_longitude[0];
    const sfRightLat = boundary.right_latitude[0];
    const sfRightLon = boundary.right_longitude[0];
    traceList.push({
      x: [sfLeftLon, sfRightLon],
      y: [sfLeftLat, sfRightLat],
      type: 'scatter' as const,
      mode: 'lines' as const,
      line: { color: '#fbbf24', width: 3 },
      name: 'Start/Finish Line',
      hoverinfo: 'skip' as const,
    });

    // S/F marker
    const sfLat = (sfLeftLat + sfRightLat) / 2;
    const sfLon = (sfLeftLon + sfRightLon) / 2;
    traceList.push({
      x: [sfLon],
      y: [sfLat],
      type: 'scatter' as const,
      mode: 'markers+text' as const,
      marker: { size: 14, color: '#fbbf24', symbol: 'star' },
      text: ['S/F'],
      textposition: 'top center' as const,
      textfont: { color: '#ffffff', size: 12 },
      name: 'Start/Finish',
      hoverinfo: 'skip' as const,
    });

    // Lap telemetry overlay
    if (lapTelemetry && lapTelemetry.frames.length > 0) {
      traceList.push({
        x: lapTelemetry.frames.map((f) => f.longitude),
        y: lapTelemetry.frames.map((f) => f.latitude),
        type: 'scatter' as const,
        mode: 'lines' as const,
        line: { color: '#8b5cf6', width: 1.5 },
        opacity: 0.6,
        name: 'Reference Lap',
        hoverinfo: 'skip' as const,
      });
    }

    // Corner regions
    const shapeList: any[] = [];
    if (trackLength > 0) {
      corners.forEach((corner, idx) => {
        const isSelected = corner.id === selectedCornerId;
        const color = CORNER_COLORS[idx % CORNER_COLORS.length];

        // Generate polygon for corner region
        const polygon = generateCornerPolygon(
          corner.start_distance,
          corner.end_distance,
          boundary,
          15
        );

        // Add filled region trace
        // Set customdata for ALL polygon points so clicks anywhere on the polygon work
        traceList.push({
          x: polygon.longitudes,
          y: polygon.latitudes,
          type: 'scatter' as const,
          mode: 'lines' as const,
          fill: 'toself' as const,
          fillcolor: isSelected ? `${color}50` : `${color}30`,
          line: {
            color: isSelected ? color : `${color}80`,
            width: isSelected ? 2 : 1,
          },
          name: `Corner ${idx + 1}`,
          hovertemplate: `Corner ${idx + 1}<br>${corner.start_distance.toFixed(0)}m - ${corner.end_distance.toFixed(0)}m<extra></extra>`,
          customdata: polygon.longitudes.map(() => corner.id),
        });

        // Add start/end markers
        const startPoint = distanceToGps(corner.start_distance, boundary);
        const endPoint = distanceToGps(corner.end_distance, boundary);

        // Start marker (green triangle)
        traceList.push({
          x: [startPoint.longitude],
          y: [startPoint.latitude],
          type: 'scatter' as const,
          mode: 'markers' as const,
          marker: {
            size: isSelected ? 16 : 12,
            color: '#22c55e',
            symbol: 'triangle-right',
            line: { color: '#ffffff', width: 1 },
          },
          name: `C${idx + 1} Start`,
          hovertemplate: `Corner ${idx + 1} Start: ${corner.start_distance.toFixed(0)}m<extra></extra>`,
          customdata: [{ cornerId: corner.id, handle: 'start' }],
          showlegend: false,
        });

        // End marker (red triangle)
        traceList.push({
          x: [endPoint.longitude],
          y: [endPoint.latitude],
          type: 'scatter' as const,
          mode: 'markers' as const,
          marker: {
            size: isSelected ? 16 : 12,
            color: '#ef4444',
            symbol: 'triangle-left',
            line: { color: '#ffffff', width: 1 },
          },
          name: `C${idx + 1} End`,
          hovertemplate: `Corner ${idx + 1} End: ${corner.end_distance.toFixed(0)}m<extra></extra>`,
          customdata: [{ cornerId: corner.id, handle: 'end' }],
          showlegend: false,
        });
      });
    }

    return { traces: traceList, shapes: shapeList };
  }, [boundary, corners, selectedCornerId, centerLat, centerLon, lapTelemetry, trackLength]);

  // Handle click on plot
  const handleClick = useCallback(
    (event: any) => {
      if (!event.points || event.points.length === 0) {
        return;
      }
      const point = event.points[0];

      // Check if clicked on a corner region or marker (has customdata)
      if (point.customdata) {
        if (typeof point.customdata === 'string') {
          // Clicked on corner region
          onSelectCorner(point.customdata);
          return;
        } else if (point.customdata?.cornerId) {
          // Clicked on a handle marker - select the corner
          const { cornerId } = point.customdata as { cornerId: string; handle: 'start' | 'end' };
          onSelectCorner(cornerId);
          return;
        }
      }

      // Check if clicked on centerline click targets
      const traceName = point.data?.name || '';

      if (traceName.includes('add corner') && trackLength > 0) {
        // Get precise coordinates using native mouse event + Plotly axis conversion
        const nativeEvent = event.event as MouseEvent;

        if (nativeEvent && point.xaxis && point.yaxis) {
          const plotDiv = plotRef.current?.el;

          if (plotDiv) {
            const rect = plotDiv.getBoundingClientRect();
            const pixelX = nativeEvent.clientX - rect.left;
            const pixelY = nativeEvent.clientY - rect.top;

            // Use Plotly's p2d (pixel to data) conversion
            const lon = point.xaxis.p2d(pixelX - point.xaxis._offset);
            const lat = point.yaxis.p2d(pixelY - point.yaxis._offset);

            // Use gpsToDistance for precise interpolated distance
            const distance = gpsToDistance({ latitude: lat, longitude: lon }, boundary);

            const halfWidth = 25;
            const startDist = Math.max(0, distance - halfWidth);
            const endDist = Math.min(trackLength, distance + halfWidth);

            onAddCorner(startDist, endDist);
            return;
          }
        }

        // Fallback: use the clicked marker position (grid-point precision)
        const distance = gpsToDistance(
          { latitude: point.y, longitude: point.x },
          boundary
        );

        const halfWidth = 25;
        onAddCorner(
          Math.max(0, distance - halfWidth),
          Math.min(trackLength, distance + halfWidth)
        );
      }
    },
    [boundary, trackLength, onAddCorner, onSelectCorner]
  );

  // Layout configuration
  const layout = useMemo(
    () => ({
      uirevision: 'preserve-zoom',
      height: 600,
      paper_bgcolor: plotLayout.paper_bgcolor,
      plot_bgcolor: plotLayout.plot_bgcolor,
      font: plotLayout.font,
      xaxis: {
        uirevision: 'preserve-zoom',
        showticklabels: false,
        showgrid: false,
        zeroline: false,
      },
      yaxis: {
        uirevision: 'preserve-zoom',
        showticklabels: false,
        showgrid: false,
        zeroline: false,
        scaleanchor: 'x' as const,
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
      shapes,
    }),
    [shapes]
  );

  return (
    <Plot
      ref={plotRef}
      data={traces}
      layout={layout}
      config={{
        responsive: true,
        displayModeBar: true,
        displaylogo: false,
        modeBarButtonsToRemove: ['lasso2d', 'select2d'],
        scrollZoom: true,
      }}
      className="w-full"
      onClick={handleClick}
    />
  );
}
