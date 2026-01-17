import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { useAuth } from '@/providers/auth-provider';
import { Link, Outlet, useLocation } from 'react-router';

const mainNavigation = [
  { name: 'Dashboard', href: '/dashboard' },
  { name: 'Sessions', href: '/sessions' },
  { name: 'Live', href: '/live' },
  { name: 'Compare', href: '/compare' },
];

const adminNavigation = [
  { name: 'Tracks', href: '/tracks' },
];

export function RootLayout() {
  const location = useLocation();
  const { user, logout, isAdmin } = useAuth();

  const isActiveRoute = (href: string) => {
    if (href === '/dashboard') {
      return location.pathname === '/dashboard';
    }
    return location.pathname === href || location.pathname.startsWith(href + '/');
  };

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b border-border bg-card">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="flex h-16 items-center justify-between">
            <div className="flex items-center gap-8">
              <Link to="/dashboard" className="flex items-center gap-2 group">
                <img src="/favicon.svg" alt="LapEvo" className="w-7 h-7 group-hover:scale-105 transition-transform" />
                <span className="text-xl font-bold text-foreground">LapEvo</span>
              </Link>
              <nav className="flex items-center gap-1">
                {/* Main navigation */}
                {mainNavigation.map((item) => (
                  <Link
                    key={item.name}
                    to={item.href}
                    className={cn(
                      'px-3 py-2 rounded-md text-sm font-medium transition-colors',
                      isActiveRoute(item.href)
                        ? 'bg-teal-500/10 text-teal-400 border border-teal-500/20'
                        : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
                    )}
                  >
                    {item.name}
                  </Link>
                ))}

                {/* Admin section */}
                {isAdmin && (
                  <>
                    <div className="flex items-center mx-3">
                      <div className="h-6 w-px bg-border" />
                      <span className="ml-3 text-xs font-medium uppercase tracking-wider text-muted-foreground">
                        Admin
                      </span>
                    </div>
                    {adminNavigation.map((item) => (
                      <Link
                        key={item.name}
                        to={item.href}
                        className={cn(
                          'px-3 py-2 rounded-md text-sm font-medium transition-colors',
                          isActiveRoute(item.href)
                            ? 'bg-teal-500/10 text-teal-400 border border-teal-500/20'
                            : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
                        )}
                      >
                        {item.name}
                      </Link>
                    ))}
                  </>
                )}
              </nav>
            </div>
            <div className="flex items-center gap-4">
              <span className="text-sm text-muted-foreground">
                {user?.display_name || user?.email}
              </span>
              <Button variant="ghost" size="sm" onClick={logout}>
                Logout
              </Button>
            </div>
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
        <Outlet />
      </main>
    </div>
  );
}
