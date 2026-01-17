import { Link, Outlet } from 'react-router';

export function AuthLayout() {
  return (
    <div className="min-h-screen bg-background flex flex-col">
      {/* Simple header with logo */}
      <header className="p-6">
        <Link to="/" className="flex items-center gap-2 w-fit group">
          <img src="/favicon.svg" alt="LapEvo" className="w-8 h-8 group-hover:scale-105 transition-transform" />
          <span className="text-xl font-bold text-foreground">LapEvo</span>
        </Link>
      </header>

      {/* Centered content */}
      <main className="flex-1 flex items-center justify-center px-4">
        <div className="w-full max-w-md">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
