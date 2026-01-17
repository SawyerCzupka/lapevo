import type { Lap } from '@/lib/types';
import { formatLapTime, formatRelativeTime } from '@/lib/format';
import { Badge } from './ui/badge';
import { Card, CardContent } from './ui/card';

interface LapCardProps {
  lap: Lap;
  onClick?: () => void;
  isSelected?: boolean;
}

export function LapCard({ lap, onClick, isSelected }: LapCardProps) {
  return (
    <div
      className={`cursor-pointer transition-all ${
        isSelected ? 'ring-2 ring-teal-500/20' : ''
      }`}
      onClick={onClick}
    >
      <Card className={isSelected ? 'border-teal-500' : ''}>
        <CardContent className="py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <div className="flex flex-col">
                <span className="text-sm text-gray-400">Lap {lap.lap_number}</span>
                <span className="text-2xl font-mono font-bold text-white">
                  {formatLapTime(lap.lap_time)}
                </span>
              </div>
            </div>

            <div className="flex items-center space-x-2">
              {lap.is_valid ? (
                <Badge variant="success">Valid</Badge>
              ) : (
                <Badge variant="danger">Invalid</Badge>
              )}
            </div>
          </div>

          <div className="mt-2 text-xs text-gray-500">
            {formatRelativeTime(lap.created_at)}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
