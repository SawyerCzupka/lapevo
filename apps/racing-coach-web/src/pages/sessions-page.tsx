import { useGetSessionsList } from '@/api/generated/sessions/sessions';
import { Badge } from '@/components/ui/badge';
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
import { useState } from 'react';
import { useNavigate } from 'react-router';

export function SessionsPage() {
  const navigate = useNavigate();
  const { data: response, isLoading, error } = useGetSessionsList();
  const [filter, setFilter] = useState('');

  // Extract sessions from response
  const sessions = response?.sessions;



  if (error) {
    console.log(error);
    console.log(JSON.stringify(error))
  }

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Sessions</h2>
          <p className="text-muted-foreground">View and analyze your racing sessions</p>
        </div>
        <Card>
          <LoadingState message="Loading sessions..." />
        </Card>
      </div>
    );
  }

  if (error) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Sessions</h2>
          <p className="text-muted-foreground">View and analyze your racing sessions</p>
        </div>
        <Card>
          <ErrorState error={error instanceof Error ? error : new Error('Failed to load sessions')} />
        </Card>
      </div>
    );
  }

  if (!sessions || sessions.length === 0) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Sessions</h2>
          <p className="text-muted-foreground">View and analyze your racing sessions</p>
        </div>
        <Card>
          <EmptyState message="No sessions found. Start racing to see your data here!" />
        </Card>
      </div>
    );
  }

  const filteredSessions = sessions.filter(
    (session) =>
      session.track_name.toLowerCase().includes(filter.toLowerCase()) ||
      session.car_name.toLowerCase().includes(filter.toLowerCase())
  );

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Sessions</h2>
          <p className="text-muted-foreground">View and analyze your racing sessions</p>
        </div>
        <Badge variant="info">{sessions.length} Total Sessions</Badge>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Recent Sessions</CardTitle>
          <CardDescription>Click on a session to view laps and telemetry</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="mb-4">
            <input
              type="text"
              placeholder="Filter by track or car..."
              className="w-full px-4 py-2 text-foreground placeholder-muted-foreground bg-muted border border-border rounded-md focus:outline-none focus:ring-2 focus:ring-teal-500"
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
            />
          </div>

          <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Track</TableHead>
                  <TableHead>Car</TableHead>
                  <TableHead>Laps</TableHead>
                  <TableHead>Date</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredSessions.map((session) => (
                  <TableRow
                    key={session.session_id}
                    onClick={() => navigate(`/session/${session.session_id}`)}
                  >
                    <TableCell>
                      <div>
                        <div className="font-medium text-foreground">{session.track_name}</div>
                        {session.track_config_name && (
                          <div className="text-sm text-muted-foreground">{session.track_config_name}</div>
                        )}
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="text-foreground">{session.car_name}</div>
                    </TableCell>
                    <TableCell>
                      <Badge variant="default">{session.lap_count} laps</Badge>
                    </TableCell>
                    <TableCell>
                      <div className="text-sm text-muted-foreground">
                        {formatDateTime(session.created_at)}
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>

          {filteredSessions.length === 0 && filter && (
            <div className="py-8 text-center">
              <p className="text-muted-foreground">No sessions match your filter</p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
