import type { LapSummary } from '@/api/generated/models';
import { useGetSessionDetail } from '@/api/generated/sessions/sessions';
import { DeleteSessionDialog } from '@/components/delete-session-dialog';
import { Badge } from '@/components/ui/badge';
import { Breadcrumbs } from '@/components/ui/breadcrumbs';
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
import { formatDateTime, formatLapTime } from '@/lib/format';
import { Trash2 } from 'lucide-react';
import { useState } from 'react';
import { useNavigate, useParams } from 'react-router';

export function SessionDetailPage() {
  const { sessionId } = useParams<{ sessionId: string }>();
  const navigate = useNavigate();
  const { data: response, isLoading, error } = useGetSessionDetail(sessionId || '');
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);

  const session = response;

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Session Details</h2>
          <p className="text-muted-foreground">Loading session information...</p>
        </div>
        <Card>
          <LoadingState message="Loading session..." />
        </Card>
      </div>
    );
  }

  if (error) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Session Details</h2>
          <p className="text-muted-foreground">Error loading session</p>
        </div>
        <Card>
          <ErrorState error={error instanceof Error ? error : new Error('Failed to load session')} />
        </Card>
      </div>
    );
  }

  if (!session) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Session Details</h2>
          <p className="text-muted-foreground">Session not found</p>
        </div>
        <Card>
          <EmptyState message="Session not found" />
        </Card>
      </div>
    );
  }

  const laps = session.laps ?? [];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <Breadcrumbs
            items={[
              { label: 'Dashboard', href: '/dashboard' },
              { label: 'Sessions', href: '/sessions' },
              { label: session.track_name },
            ]}
            className="mb-3"
          />
          <h2 className="text-3xl font-bold tracking-tight text-foreground">
            {session.track_name}
          </h2>
          {session.track_config_name && (
            <p className="text-xl text-muted-foreground">{session.track_config_name}</p>
          )}
        </div>
        <div className="flex items-center gap-3">
          <Badge variant="info">{laps.length} Laps</Badge>
          <Button
            variant="outline"
            size="sm"
            className="text-muted-foreground hover:text-destructive hover:border-destructive"
            onClick={() => setShowDeleteDialog(true)}
          >
            <Trash2 className="h-4 w-4 mr-2" />
            Delete Session
          </Button>
        </div>
      </div>

      {/* Session Info */}
      <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Car</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-semibold text-foreground">{session.car_name}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Track Type</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-semibold text-white capitalize">{session.track_type}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Date</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-semibold text-foreground">{formatDateTime(session.created_at)}</p>
          </CardContent>
        </Card>
      </div>

      {/* Laps Table */}
      <Card>
        <CardHeader>
          <CardTitle>Laps</CardTitle>
          <CardDescription>Click on a lap to view detailed telemetry</CardDescription>
        </CardHeader>
        <CardContent>
          {laps.length === 0 ? (
            <EmptyState message="No laps recorded in this session" />
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Lap</TableHead>
                    <TableHead>Time</TableHead>
                    <TableHead>Valid</TableHead>
                    <TableHead>Metrics</TableHead>
                    <TableHead>Recorded</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {laps.map((lap: LapSummary) => (
                    <TableRow
                      key={lap.lap_id}
                      onClick={() => navigate(`/lap/${lap.lap_id}`)}
                    >
                      <TableCell>
                        <span className="font-medium text-foreground">Lap {lap.lap_number}</span>
                      </TableCell>
                      <TableCell>
                        <span className="font-mono text-foreground">
                          {lap.lap_time ? formatLapTime(lap.lap_time) : '--:--.---'}
                        </span>
                      </TableCell>
                      <TableCell>
                        <Badge variant={lap.is_valid ? 'success' : 'danger'}>
                          {lap.is_valid ? 'Valid' : 'Invalid'}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        <Badge variant={lap.has_metrics ? 'info' : 'default'}>
                          {lap.has_metrics ? 'Analyzed' : 'Pending'}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        <span className="text-sm text-muted-foreground">
                          {formatDateTime(lap.created_at)}
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

      <DeleteSessionDialog
        session={{
          id: session.session_id,
          trackName: session.track_name,
          carName: session.car_name,
          lapCount: laps.length,
        }}
        open={showDeleteDialog}
        onOpenChange={setShowDeleteDialog}
        onDeleted={() => navigate('/sessions')}
      />
    </div>
  );
}
