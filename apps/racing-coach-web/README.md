# Racing Coach Web Dashboard

Modern web dashboard for analyzing racing telemetry data from iRacing sessions.

## Tech Stack

- **Framework**: Vite 7 + React 19 + TypeScript
- **Styling**: Tailwind CSS v4
- **UI Components**: shadcn/ui (Radix UI primitives)
- **State Management**: TanStack Query v5 (React Query)
- **Routing**: React Router v7
- **API Client**: Orval (auto-generated from OpenAPI spec)
- **Charts**: Plotly.js for interactive telemetry visualization
- **Forms**: React Hook Form + Zod validation

## Features

### Current Pages

- **Sessions**: List and browse all racing sessions
- **Session Detail**: View individual session with lap breakdown
- **Lap Analysis**: Detailed telemetry charts for single lap
- **Lap Comparison**: Side-by-side comparison of two laps
- **Live Session**: Real-time updates via WebSocket (coming soon)

### Planned Features

- Interactive telemetry charts (speed, throttle, brake, steering)
- Corner-by-corner performance breakdown
- Braking zone analysis
- Delta comparison overlays
- Tire temperature and wear visualization
- Real-time session monitoring

## Development

### Prerequisites

- Node.js 20+
- npm or similar package manager
- Racing Coach Server running on localhost:8000

### Local Setup

```bash
# Install dependencies
npm install

# Generate API client from OpenAPI spec
# (Make sure the server is running first!)
npm run generate:api

# Start dev server
npm run dev

# Visit http://localhost:3000
```

### Available Scripts

- `npm run dev` - Start development server (port 3000)
- `npm run build` - Build for production
- `npm run preview` - Preview production build
- `npm run lint` - Lint code with ESLint
- `npm run format` - Format code with Prettier
- `npm run generate:api` - Generate TypeScript API client from OpenAPI

## Production Deployment

The app is deployed to Cloudflare Workers via Wrangler. The Worker serves the static SPA build and proxies `/api/*` requests to the backend — the backend URL is never baked into the JS bundle.

```bash
# Deploy to Cloudflare Workers
npx wrangler deploy

# Preview locally using the Workers runtime (not Vite)
npx wrangler dev
```

### How the Proxy Works

The Worker (`worker/index.ts`) acts as a Backend for Frontend (BFF):

- Requests to `/api/*` are forwarded to `env.API_BASE_URL` (the backend server)
- All other requests are served from the static `dist/` build via the `ASSETS` binding
- `run_worker_first = ["/api/*"]` in `wrangler.toml` ensures API paths hit the Worker before the asset handler

This means the SPA makes same-origin requests (no CORS), and the backend URL is never exposed to the browser.

### Environment Variables

`API_BASE_URL` is set in `wrangler.toml` under `[vars]` (plaintext, visible in source). This is fine for a public-facing URL. For values that should be kept secret (e.g. an internal service token used to authenticate Worker→backend traffic), use Wrangler secrets — they are encrypted at rest and never appear in source:

```bash
npx wrangler secret put MY_SECRET
```

Secrets are accessed the same way as vars (`env.MY_SECRET`) in the Worker code.

### TypeScript Setup for the Worker

The `worker/` directory uses its own `tsconfig.worker.json`, referenced from the root `tsconfig.json`. This separation is necessary because the Workers runtime redefines globals like `Request`, `Response`, and `fetch` differently from the browser DOM types used by the React app — sharing a tsconfig would cause type conflicts.

`npx wrangler types` generates `worker-configuration.d.ts` at the project root. This file provides typed `Env` bindings (e.g. `ASSETS: Fetcher`, `API_BASE_URL: string`) and Workers runtime globals matched to your `compatibility_date`. Re-run it whenever bindings in `wrangler.toml` change:

```bash
npx wrangler types
```

`worker-configuration.d.ts` is included in `tsconfig.worker.json` and committed to source control.

## Architecture

The app uses Orval to auto-generate TypeScript types and React Query hooks from the FastAPI server's OpenAPI specification.

## License

See root LICENSE file.
