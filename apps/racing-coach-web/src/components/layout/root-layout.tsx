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
    <div className="min-h-screen bg-gray-950">
      {/* Header */}
      <header className="border-b border-gray-800 bg-gray-900">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="flex h-16 items-center justify-between">
            <div className="flex items-center gap-8">
              <Link to="/dashboard" className="flex items-center gap-2 group">
                <img src="/favicon.svg" alt="LapEvo" className="w-7 h-7 group-hover:scale-105 transition-transform" />
                <span className="text-xl font-bold text-white">LapEvo</span>
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
                        ? 'bg-gray-800 text-white'
                        : 'text-gray-400 hover:bg-gray-800/50 hover:text-white'
                    )}
                  >
                    {item.name}
                  </Link>
                ))}

                {/* Admin section */}
                {isAdmin && (
                  <>
                    <div className="flex items-center mx-3">
                      <div className="h-6 w-px bg-gray-700" />
                      <span className="ml-3 text-xs font-medium uppercase tracking-wider text-gray-500">
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
                            ? 'bg-gray-800 text-white'
                            : 'text-gray-400 hover:bg-gray-800/50 hover:text-white'
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
              <span className="text-sm text-gray-400">
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
