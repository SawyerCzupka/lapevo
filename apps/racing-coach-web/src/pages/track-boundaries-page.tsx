import { useListTrackBoundaries } from '@/api/generated/tracks/tracks';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { EmptyState, ErrorState, LoadingState } from '@/components/ui/loading-states';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { formatDateTime } from '@/lib/format';
import { useNavigate } from 'react-router';

export function TrackBoundariesPage() {
  const navigate = useNavigate();
  const { data: response, isLoading, error } = useListTrackBoundaries();

  const boundaries = response?.boundaries;

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Track Boundaries</h2>
          <p className="text-muted-foreground">Manage track boundary data for lateral position analysis</p>
        </div>
        <Card>
          <LoadingState message="Loading track boundaries..." />
        </Card>
      </div>
    );
  }

  if (error) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Track Boundaries</h2>
          <p className="text-muted-foreground">Manage track boundary data for lateral position analysis</p>
        </div>
        <Card>
          <ErrorState
            error={error instanceof Error ? error : new Error('Failed to load track boundaries')}
          />
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Track Boundaries</h2>
          <p className="text-muted-foreground">Manage track boundary data for lateral position analysis</p>
        </div>
        <div className="flex gap-4 items-center">
          <Badge variant="info">{boundaries?.length || 0} Tracks</Badge>
          <Button onClick={() => navigate('/tracks/upload')}>Upload IBT File</Button>
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Available Track Boundaries</CardTitle>
          <CardDescription>Click on a track to view the boundary map</CardDescription>
        </CardHeader>
        <CardContent>
          {!boundaries || boundaries.length === 0 ? (
            <EmptyState message="No track boundaries yet. Upload an IBT file to create one." />
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Track Name</TableHead>
                    <TableHead>Configuration</TableHead>
                    <TableHead>Grid Points</TableHead>
                    <TableHead>Created</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {boundaries.map((boundary) => (
                    <TableRow
                      key={boundary.id}
                      onClick={() => navigate(`/tracks/${boundary.id}`)}
                    >
                      <TableCell>
                        <span className="font-medium text-foreground">{boundary.track_name}</span>
                      </TableCell>
                      <TableCell>
                        <span className="text-muted-foreground">
                          {boundary.track_config_name || 'Default'}
                        </span>
                      </TableCell>
                      <TableCell>
                        <Badge variant="default">{boundary.grid_size} points</Badge>
                      </TableCell>
                      <TableCell>
                        <span className="text-sm text-muted-foreground">
                          {formatDateTime(boundary.created_at)}
                        </span>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
