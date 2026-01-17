import { useCallback } from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Slider } from '@/components/ui/slider';
import type { CornerDraft } from '@/pages/corner-segment-editor-page';

interface CornerSegmentListProps {
  corners: CornerDraft[];
  selectedCornerId: string | null;
  overlappingPairs: [number, number][];
  onSelectCorner: (id: string | null) => void;
  onDeleteCorner: (id: string) => void;
  onUpdateCorner: (id: string, startDistance: number, endDistance: number) => void;
  trackLength: number;
}

const MIN_CORNER_WIDTH = 10; // Minimum 10m corner width
const SLIDER_PADDING = 150; // Padding around corner for slider range (meters)

export function CornerSegmentList({
  corners,
  selectedCornerId,
  overlappingPairs,
  onSelectCorner,
  onDeleteCorner,
  onUpdateCorner,
  trackLength,
}: CornerSegmentListProps) {
  // Get indices of corners that are involved in overlaps
  const overlappingIndices = new Set<number>();
  overlappingPairs.forEach(([i, j]) => {
    overlappingIndices.add(i);
    overlappingIndices.add(j);
  });

  // Handle slider value change - updates in real-time
  const handleSliderChange = useCallback(
    (cornerId: string, values: number[]) => {
      const [start, end] = values;
      onUpdateCorner(cornerId, start, end);
    },
    [onUpdateCorner]
  );

  // Handle number input change - updates in real-time
  const handleInputChange = useCallback(
    (
      cornerId: string,
      field: 'start' | 'end',
      value: string,
      currentStart: number,
      currentEnd: number
    ) => {
      const numValue = parseFloat(value);
      if (isNaN(numValue)) return;

      // Clamp to track length
      const clampedValue = Math.max(0, Math.min(trackLength, numValue));

      if (field === 'start') {
        // Ensure start doesn't exceed end - MIN_CORNER_WIDTH
        const newStart = Math.min(clampedValue, currentEnd - MIN_CORNER_WIDTH);
        onUpdateCorner(cornerId, Math.max(0, newStart), currentEnd);
      } else {
        // Ensure end doesn't go below start + MIN_CORNER_WIDTH
        const newEnd = Math.max(clampedValue, currentStart + MIN_CORNER_WIDTH);
        onUpdateCorner(cornerId, currentStart, Math.min(trackLength, newEnd));
      }
    },
    [trackLength, onUpdateCorner]
  );

  // Handle keyboard adjustments for fine-tuning
  const handleKeyDown = useCallback(
    (
      e: React.KeyboardEvent,
      cornerId: string,
      field: 'start' | 'end',
      currentStart: number,
      currentEnd: number
    ) => {
      const step = e.shiftKey ? 10 : 1; // Hold shift for 10m steps
      let delta = 0;

      if (e.key === 'ArrowUp' || e.key === 'ArrowRight') {
        delta = step;
      } else if (e.key === 'ArrowDown' || e.key === 'ArrowLeft') {
        delta = -step;
      } else {
        return;
      }

      e.preventDefault();

      if (field === 'start') {
        const newStart = Math.max(0, Math.min(currentEnd - MIN_CORNER_WIDTH, currentStart + delta));
        onUpdateCorner(cornerId, newStart, currentEnd);
      } else {
        const newEnd = Math.max(currentStart + MIN_CORNER_WIDTH, Math.min(trackLength, currentEnd + delta));
        onUpdateCorner(cornerId, currentStart, newEnd);
      }
    },
    [trackLength, onUpdateCorner]
  );

  if (corners.length === 0) {
    return (
      <div className="text-center py-8 text-gray-500">
        <p className="mb-2">No corners defined</p>
        <p className="text-sm">Click on the centerline to add a corner</p>
      </div>
    );
  }

  return (
    <div className="space-y-2 max-h-[500px] overflow-y-auto">
      {corners.map((corner, idx) => {
        const isSelected = corner.id === selectedCornerId;
        const hasOverlap = overlappingIndices.has(idx);

        return (
          <div
            key={corner.id}
            className={`p-3 rounded-lg border cursor-pointer transition-colors ${
              isSelected
                ? 'bg-teal-900/30 border-teal-600'
                : 'bg-gray-800/50 border-gray-700 hover:border-gray-600'
            }`}
            onClick={() => onSelectCorner(corner.id)}
          >
            <div className="flex items-center justify-between mb-2">
              <span className="font-medium text-white">Corner {idx + 1}</span>
              <div className="flex items-center gap-2">
                {corner.isNew && <Badge variant="info">New</Badge>}
                {corner.isModified && <Badge variant="default">Modified</Badge>}
                {hasOverlap && <Badge variant="danger">Overlap</Badge>}
              </div>
            </div>

            {isSelected ? (
              // Expanded editing view when selected
              (() => {
                // Calculate bounded slider range centered around the corner
                const sliderMin = Math.max(0, corner.start_distance - SLIDER_PADDING);
                const sliderMax = Math.min(trackLength, corner.end_distance + SLIDER_PADDING);

                return (
              <div className="space-y-3" onClick={(e) => e.stopPropagation()}>
                {/* Range slider with bounded range for precision */}
                <div className="px-1">
                  <Slider
                    value={[corner.start_distance, corner.end_distance]}
                    min={sliderMin}
                    max={sliderMax}
                    step={1}
                    minStepsBetweenThumbs={MIN_CORNER_WIDTH}
                    onValueChange={(values) => handleSliderChange(corner.id!, values)}
                  />
                  <div className="flex justify-between text-xs text-gray-500 mt-1">
                    <span>{sliderMin.toFixed(0)}m</span>
                    <span>{sliderMax.toFixed(0)}m</span>
                  </div>
                </div>

                {/* Number inputs for precise adjustment */}
                <div className="flex gap-2 items-center">
                  <div className="flex-1">
                    <label className="text-xs text-gray-400 block mb-1">Start</label>
                    <input
                      type="number"
                      className="w-full bg-gray-900 border border-gray-600 rounded px-2 py-1 text-white text-sm focus:border-teal-500 focus:outline-none"
                      value={corner.start_distance.toFixed(0)}
                      onChange={(e) =>
                        handleInputChange(
                          corner.id!,
                          'start',
                          e.target.value,
                          corner.start_distance,
                          corner.end_distance
                        )
                      }
                      onKeyDown={(e) =>
                        handleKeyDown(
                          e,
                          corner.id!,
                          'start',
                          corner.start_distance,
                          corner.end_distance
                        )
                      }
                    />
                  </div>
                  <span className="text-gray-400 mt-5">-</span>
                  <div className="flex-1">
                    <label className="text-xs text-gray-400 block mb-1">End</label>
                    <input
                      type="number"
                      className="w-full bg-gray-900 border border-gray-600 rounded px-2 py-1 text-white text-sm focus:border-teal-500 focus:outline-none"
                      value={corner.end_distance.toFixed(0)}
                      onChange={(e) =>
                        handleInputChange(
                          corner.id!,
                          'end',
                          e.target.value,
                          corner.start_distance,
                          corner.end_distance
                        )
                      }
                      onKeyDown={(e) =>
                        handleKeyDown(
                          e,
                          corner.id!,
                          'end',
                          corner.start_distance,
                          corner.end_distance
                        )
                      }
                    />
                  </div>
                  <span className="text-gray-500 text-sm mt-5">m</span>
                </div>

                {/* Length display */}
                <div className="text-xs text-gray-500">
                  Length: {(corner.end_distance - corner.start_distance).toFixed(0)}m
                </div>

                {/* Delete button */}
                <Button
                  size="sm"
                  variant="outline"
                  className="text-red-400 hover:text-red-300 w-full"
                  onClick={() => {
                    if (confirm(`Delete Corner ${idx + 1}?`)) {
                      onDeleteCorner(corner.id!);
                    }
                  }}
                >
                  Delete Corner
                </Button>

                {/* Keyboard hint */}
                <div className="text-xs text-gray-600">
                  Tip: Use arrow keys to fine-tune (Shift + arrows for 10m steps)
                </div>
              </div>
                );
              })()
            ) : (
              // Collapsed view when not selected
              <div className="text-sm text-gray-400">
                {corner.start_distance.toFixed(0)}m - {corner.end_distance.toFixed(0)}m
                <span className="text-gray-600 ml-2">
                  ({(corner.end_distance - corner.start_distance).toFixed(0)}m)
                </span>
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
