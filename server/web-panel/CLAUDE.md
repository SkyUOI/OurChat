# CLAUDE.md - Web Panel

This file provides guidance to Claude Code (claude.ai/code) when working with the **web-panel** portion of this repository.

## Overview

The web-panel is a Vue 3-based administration interface for managing the OurChat Rust server. It provides real-time monitoring, user management, configuration, and system logs through a modern web interface. The panel communicates with the Rust backend via gRPC-Web protocol.

## Quick Reference

### Most Common Commands

```bash
# Install dependencies
npm install

# Development server with hot reload
npm run dev

# Production build
npm run build

# Type checking
npm run type-check

# Unit tests
npm run test:unit

# End-to-end tests
npm run test:e2e

# Linting and formatting
npm run lint
npm run format
```

### Critical Notes
- Uses **Vue 3** with Composition API, **TypeScript**, and **Vite**
- **gRPC-Web** for communication with Rust backend (proxy via `/backend`)
- **Element Plus** UI component library with **Tailwind CSS**
- Base path `/panel` for reverse proxy configuration
- See sections below for detailed guidance

## Technology Stack

### Core Frameworks
- **Vue 3** (v3.5.22) with Composition API (`<script setup>` syntax)
- **TypeScript** (~5.8.3) for type safety
- **Vite** (v6.4.1) as build tool and dev server
- **Pinia** (v3.0.3) for state management
- **Vue Router** (v4.6.3) for routing
- **Vue I18n** (v11.1.10) for internationalization

### UI Components & Styling
- **Element Plus** (v2.11.7) UI component library
- **Tailwind CSS** (v4.1.16) via `@tailwindcss/vite` plugin
- **Element Plus Icons** for iconography
- **ECharts** (v5.5.1+) for data visualization
- **vue-echarts** (v7.0.3+) Vue 3 wrapper for ECharts

### gRPC Integration
- **@protobuf-ts/runtime** (v2.11.1) - Protobuf TypeScript runtime
- **@protobuf-ts/grpcweb-transport** (v2.11.1) - gRPC-Web transport
- **@improbable-eng/grpc-web** (v0.15.0) - gRPC-Web client

### Testing
- **Vitest** (v3.2.4) for unit testing
- **Playwright** (v1.56.1) for end-to-end testing

### Code Quality
- **ESLint** (v9.39.0) with Vue/TypeScript configurations
- **Prettier** (v3.5.3) for code formatting
- **Vue TSC** (v2.2.12) for type checking

## Project Structure

```
server/web-panel/
├── src/
│   ├── api/                    # Generated gRPC client code (from protobuf)
│   │   ├── google/            # Google protobuf definitions
│   │   │   ├── timestamp.ts   # Timestamp type for time-based APIs
│   │   │   └── duration.ts    # Duration type for interval-based APIs
│   │   └── service/           # OurChat service definitions
│   │       ├── auth/          # Authentication services
│   │       ├── basic/         # Basic services (server, support, etc.)
│   │       ├── ourchat/       # OurChat core services
│   │       └── server_manage/ # Server management services
│   │           └── monitoring/  # Metrics monitoring APIs
│   │               └── v1/monitoring.ts  # MonitoringMetrics types
│   ├── assets/                # Static assets
│   ├── components/            # Vue components
│   │   ├── icons/            # Icon components
│   │   ├── ConfigManager.vue
│   │   ├── Header.vue
│   │   ├── ResourceMonitor.vue
│   │   ├── Sidebar.vue
│   │   └── SystemOverview.vue
│   ├── locales/               # Internationalization files
│   │   ├── en.json
│   │   └── zh.json
│   ├── router/               # Vue Router configuration
│   │   └── index.ts
│   ├── stores/               # Pinia stores
│   │   └── grpc.ts          # gRPC connection store
│   ├── views/               # Route views/pages
│   │   ├── ConfigView.vue
│   │   ├── DashboardView.vue
│   │   ├── LoginView.vue
│   │   ├── LogsView.vue
│   │   ├── MonitorView.vue
│   │   ├── ServicesView.vue
│   │   └── UsersView.vue
│   ├── App.vue              # Root component
│   ├── main.ts             # Application entry point
│   └── shims-vue.d.ts      # TypeScript declarations
├── public/                  # Public assets
│   └── favicon.png
├── dist/                    # Build output directory
├── e2e/                    # End-to-end tests
│   ├── vue.spec.ts
│   └── tsconfig.json
└── Configuration files:
    ├── package.json
    ├── vite.config.ts
    ├── tsconfig.json
    ├── tsconfig.app.json
    ├── tsconfig.node.json
    ├── tsconfig.vitest.json
    ├── playwright.config.ts
    ├── vitest.config.ts
    ├── eslint.config.ts
    ├── .prettierrc.json
    └── .gitignore
```

## Setup and Development

### Prerequisites
- Node.js (version compatible with package.json)
- npm or pnpm (package manager)

### Installation
```bash
cd /home/limuy/OurChat/server/web-panel
npm install  # or pnpm install
```

### Development Server
```bash
npm run dev
```
- Starts Vite dev server on `http://localhost:5173/panel`
- Hot module replacement enabled
- Proxy configuration routes `/backend` to `http://localhost:7777` (Rust server)

### Type Checking
```bash
npm run type-check  # Uses vue-tsc for .vue file type checking
```

## Integration with Rust Server

### Communication Protocol
The web-panel uses **gRPC-Web** to communicate with the Rust backend. The protocol is configured via:

1. **Proxy Setup**: `vite.config.ts` proxies `/backend` requests to `http://localhost:7777`
2. **Generated Clients**: TypeScript gRPC clients are generated from `.proto` files in `/home/limuy/OurChat/service/`
3. **Authentication**: JWT tokens stored in `localStorage` with route guards

### Available gRPC Services
- **AuthService** - User authentication and registration
- **BasicService** - Server information and support
- **OurChatService** - Core chat functionality
- **ServerManageService** - Server management operations
  - `GetMonitoringMetrics` - Current server metrics
  - `GetHistoricalMetrics` - Historical metrics for time ranges
  - `GetConfig`/`SetConfig` - Configuration management
  - `ListUsers`/`BanUser`/`UnbanUser` - User management
  - `ListSessions` - Session management

### Authentication Flow
1. Login at `/login` route
2. JWT token obtained from Rust server
3. Token stored in `localStorage`
4. Route guards redirect to login if token missing

## Metrics Monitoring UI

The web-panel includes a comprehensive metrics monitoring interface (`MonitorView.vue`) that displays real-time and historical server metrics.

### Key Features
- **Real-time charts** for connections, message throughput, CPU, and memory usage
- **Historical metrics** with configurable time ranges (1h, 6h, 24h, 7d)
- **Auto-refresh** every 30 seconds
- **Export functionality** to download metrics as JSON
- **System metrics toggle** to include/exclude CPU, memory, disk metrics
- **Full i18n support** for English and Chinese

### Technology Stack for Charts
- **ECharts** (v5.5.1+) - Charting library
- **vue-echarts** (v7.0.3+) - Vue 3 wrapper for ECharts

### gRPC APIs Used
The metrics UI uses two gRPC endpoints from `ServerManageService`:

1. **GetMonitoringMetrics** - Fetches current server metrics
   - `includeSystemMetrics`: Include CPU, memory, disk usage
   - `includeTokioMetrics`: Include Tokio runtime metrics
   - Returns: `MonitoringMetrics` with active connections, users, sessions, etc.

2. **GetHistoricalMetrics** - Fetches historical metrics for charts
   - `startTime`/`endTime`: Time range (as `google.protobuf.Timestamp`)
   - `interval`: Aggregation interval (as `google.protobuf.Duration`)
   - Returns: Array of `MetricDataPoint` with timestamped metrics

### Implementation Details

**Chart Configuration:**
```typescript
// ECharts is configured with tree-shaking for optimal bundle size
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart } from 'echarts/charts'
import { GridComponent, TooltipComponent, LegendComponent } from 'echarts/components'

use([CanvasRenderer, LineChart, GridComponent, TooltipComponent, LegendComponent])
```

**Protobuf Types:**
- `Timestamp` from `@/api/google/protobuf/timestamp` - Use `Timestamp.fromDate()` for conversion
- `Duration` from `@/api/google/protobuf/duration` - Use `Duration.create()` for construction

**Auto-refresh Pattern:**
```typescript
onMounted(() => {
  refreshMetrics()
  refreshInterval = setInterval(refreshMetrics, 30000) // 30 seconds
})

onUnmounted(() => {
  if (refreshInterval) clearInterval(refreshInterval)
})
```

### Modifying Metrics Display

**To add a new metric chart:**
1. Add chart option computed property in `MonitorView.vue`
2. Create `<v-chart>` component with the option
3. Map historical data points to the chart series

**To change refresh interval:**
- Modify the `setInterval` duration in `onMounted()` hook

**To add new time range options:**
- Add `<el-option>` to the time range `<el-select>` with value in seconds

### File Locations
- `src/views/MonitorView.vue` - Main metrics view component
- `src/api/service/server_manage/monitoring/v1/monitoring.ts` - Generated protobuf types
- `src/locales/en.json` & `src/locales/zh.json` - Translation keys (prefixed with metrics-related keys)

## Build and Deployment

### Production Build
```bash
npm run build
```
- Runs type checking first (`vue-tsc --build`)
- Builds optimized assets to `/dist` directory
- Base path configured as `/panel` for reverse proxy

### Build Configuration (`vite.config.ts`)
- Base path: `/panel`
- Proxy: `/backend` → `http://localhost:7777`
- Aliases: `@` → `./src`
- Tailwind CSS via Vite plugin

### Static File Serving
The built `/dist` directory contains static files that can be served by:
- Nginx/Apache reverse proxy
- Rust HTTP server static file serving
- Any static file server

### Deployment Considerations
- Ensure Rust server is accessible at the proxy target
- Configure CORS if serving from different domain/port
- Set appropriate cache headers for static assets

## Testing

### Unit Tests (Vitest)
```bash
npm run test:unit
```
- Tests Vue components and composables
- Configuration in `vitest.config.ts`
- Uses `@vue/test-utils` for component testing

### End-to-End Tests (Playwright)
```bash
# Install browsers first
npx playwright install

# Run all E2E tests
npm run test:e2e

# Run specific browser
npm run test:e2e -- --project=chromium

# Debug mode
npm run test:e2e -- --debug
```
- Configuration in `playwright.config.ts`
- Tests complete user flows

### Linting and Formatting
```bash
npm run lint    # ESLint with auto-fix
npm run format  # Prettier formatting
```

## Protobuf Code Generation

When protobuf definitions change in `/home/limuy/OurChat/service/`, regenerate TypeScript clients:

```bash
# From project root
python script/generate_grpc_web.py
```

This script:
1. Scans `.proto` files in `service/` directory
2. Uses `pnpx protoc` with `@protobuf-ts/plugin`
3. Outputs generated code to `server/web-panel/src/api/`
4. Includes TypeScript definitions and gRPC client code

### Manual Generation (if script unavailable)
```bash
cd /home/limuy/OurChat
pnpx protoc service/*.proto \
  --ts_out=server/web-panel/src/api \
  --ts_opt eslint_disable
```

## Common Development Tasks

### Adding New Views
1. Create Vue component in `src/views/`
2. Add route in `src/router/index.ts`
3. Update sidebar navigation if needed (`src/components/Sidebar.vue`)

### Creating Components
1. Create `.vue` file in `src/components/`
2. Use Composition API with `<script setup>` syntax
3. Import Element Plus components as needed
4. Apply Tailwind CSS classes for styling

### Adding gRPC Service Calls
1. Ensure protobuf definitions are generated (`src/api/service/`)
2. Import generated client in component/store
3. Use `@protobuf-ts/grpcweb-transport` for transport
4. Handle async loading states and errors

### Internationalization
1. Add translation keys to `src/locales/en.json` and `src/locales/zh.json`
2. Use `$t('key')` in templates or `useI18n()` in scripts

## Troubleshooting

### Common Issues

**Development server not connecting to backend:**
- Ensure Rust server is running on `http://localhost:7777`
- Check proxy configuration in `vite.config.ts`
- Verify no CORS issues (proxy should handle)

**Type errors with `.vue` files:**
- Use `npm run type-check` to identify issues
- Ensure Volar extension is installed in VS Code
- Check `shims-vue.d.ts` for type declarations

**Build failures:**
- Run `npm run type-check` separately to see type errors
- Ensure all dependencies are installed (`npm install`)
- Check Node.js version compatibility

**gRPC client generation issues:**
- Run `python script/generate_grpc_web.py` from project root
- Ensure `pnpx` is available (install with `npm install -g pnpm`)
- Check protobuf file syntax

**Tests failing:**
- Unit tests: Check component mocking and async handling
- E2E tests: Ensure dev server is running before tests
- Playwright: Install browsers with `npx playwright install`

### Performance Optimization

**Bundle size:**
- Vite automatically code-splits routes
- Use dynamic imports for large components
- Monitor bundle with `vite-bundle-visualizer`

**Runtime performance:**
- Use Vue's `computed` and `watchEffect` efficiently
- Implement virtual scrolling for large lists
- Debounce frequent gRPC calls

## Important Notes

- The panel is designed to be served under `/panel` path
- Authentication state is preserved in `localStorage`
- All gRPC calls go through `/backend` proxy
- Element Plus components require proper import in `main.ts`
- Internationalization supports English (en) and Chinese (zh)
- Tailwind CSS uses JIT mode via Vite plugin

## Related Documentation

- [Vue 3 Documentation](https://vuejs.org/)
- [Vite Documentation](https://vite.dev/)
- [Element Plus Documentation](https://element-plus.org/)
- [gRPC-Web Documentation](https://github.com/grpc/grpc-web)
- [Tailwind CSS Documentation](https://tailwindcss.com/)

Refer to the main project CLAUDE.md for server-side integration details.