import { useGetSessionsList } from '@/api/generated/sessions/sessions';
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
import { formatRelativeTime } from '@/lib/format';
import { useAuth } from '@/providers/auth-provider';
import { useNavigate } from 'react-router';

export function DashboardPage() {
  const navigate = useNavigate();
  const { user } = useAuth();
  const { data: response, isLoading, error } = useGetSessionsList();

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Dashboard</h2>
          <p className="text-muted-foreground">Loading your racing data...</p>
        </div>
        <Card>
          <LoadingState message="Loading dashboard..." />
        </Card>
      </div>
    );
  }

  if (error) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">Dashboard</h2>
        </div>
        <Card>
          <ErrorState error={error instanceof Error ? error : new Error('Failed to load dashboard')} />
        </Card>
      </div>
    );
  }

  const sessions = response?.sessions ?? [];
  const recentSessions = sessions.slice(0, 5);
  const totalSessions = response?.total ?? sessions.length;
  const totalLaps = sessions.reduce((acc, s) => acc + s.lap_count, 0);
  const uniqueTracks = new Set(sessions.map((s) => s.track_name)).size;
  const uniqueCars = new Set(sessions.map((s) => s.car_name)).size;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-foreground">
            Welcome back{user?.display_name ? `, ${user.display_name}` : ''}
          </h2>
          <p className="text-muted-foreground">Here's your racing overview</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => navigate('/compare')}>
            Compare Laps
          </Button>
          <Button onClick={() => navigate('/live')}>Live Session</Button>
        </div>
      </div>

      {/* Quick Stats Grid */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Total Sessions</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-3xl font-bold text-foreground">{totalSessions}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Total Laps</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-3xl font-bold text-foreground">{totalLaps}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Tracks Driven</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-3xl font-bold text-foreground">{uniqueTracks}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">Cars Used</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-3xl font-bold text-foreground">{uniqueCars}</p>
          </CardContent>
        </Card>
      </div>

      {/* Recent Sessions */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>Recent Sessions</CardTitle>
            <CardDescription>Your last 5 racing sessions</CardDescription>
          </div>
          <Button variant="ghost" onClick={() => navigate('/sessions')}>
            View All
          </Button>
        </CardHeader>
        <CardContent>
          {recentSessions.length === 0 ? (
            <EmptyState message="No sessions yet. Start racing to see your data here!" />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Track</TableHead>
                  <TableHead>Car</TableHead>
                  <TableHead>Laps</TableHead>
                  <TableHead>When</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {recentSessions.map((session) => (
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
                    <TableCell className="text-foreground">{session.car_name}</TableCell>
                    <TableCell>
                      <Badge variant="default">{session.lap_count}</Badge>
                    </TableCell>
                    <TableCell>
                      <span className="text-sm text-muted-foreground">
                        {formatRelativeTime(session.created_at)}
                      </span>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
