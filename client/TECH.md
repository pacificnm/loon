# Loon Client — Technical Plan

Stack, dependencies, build pipeline, and platform integration. Planning only — no code yet.

---

## Stack decisions

| Layer | Choice | Rationale |
|-------|--------|-----------|
| Framework | **React 18** | Team familiarity, ecosystem |
| Build | **Vite 5** | Fast HMR for dev; static `dist/` for webOS |
| Language | **TypeScript 5** | API type safety |
| Routing | **React Router 6** | Screen stack + focus state in `location.state` |
| Focus | **@noriginmedia/norigin-spatial-navigation** | Proven TV spatial nav; spike in W0 |
| Styling | **CSS Modules** | Scoped styles, no runtime CSS-in-JS overhead on TV |
| Data fetching | **TanStack Query v5** | Cache browse data, stale-while-revalidate on home return |
| HTTP | `fetch` wrapper | No axios — smaller bundle |
| Tests | **Vitest** + Testing Library | Unit + component |
| webOS platform | **webOSTV.js** (optional W1) | Back button, platform events |

**Not using:** Electron, Enact, Flutter for webOS (Flutter is [new on webOS](https://webostv.developer.lge.com/) but React matches existing Loon docs and web stack).

---

## Dependencies (draft `package.json`)

```json
{
  "name": "loon-webos",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "test": "vitest run",
    "package:webos": "npm run build && ares-package dist -o build"
  },
  "dependencies": {
    "@noriginmedia/norigin-spatial-navigation": "^2.0.0",
    "@tanstack/react-query": "^5.0.0",
    "react": "^18.3.0",
    "react-dom": "^18.3.0",
    "react-router-dom": "^6.26.0"
  },
  "devDependencies": {
    "@testing-library/react": "^16.0.0",
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.0",
    "typescript": "^5.5.0",
    "vite": "^5.4.0",
    "vitest": "^2.0.0"
  }
}
```

`webostvjs` added in W1 if native Back handling needed:

```bash
npm install webostvjs
```

---

## Environment & config

| Key | Storage | Notes |
|-----|---------|-------|
| `loon_server_url` | `localStorage` | e.g. `http://192.168.88.205:3000` |
| `loon_recent_searches` | `localStorage` | JSON array, max 5 |
| `VITE_LOON_SERVER` | build-time `.env` | Dev default only |

```typescript
// src/config.ts
export function getServerUrl(): string | null {
  return localStorage.getItem('loon_server_url')
    ?? import.meta.env.VITE_LOON_SERVER
    ?? null;
}

export function artworkUrl(path: string | undefined, server: string): string | undefined {
  if (!path) return undefined;
  if (path.startsWith('http')) return path;
  return `${server}${path}`;
}
```

Server returns relative artwork paths (`/api/artwork/...`) — always prepend base URL.

---

## API client shape

```typescript
// src/api/client.ts — conceptual
class LoonApi {
  constructor(private baseUrl: string) {}

  async health(): Promise<HealthResponse>
  async browse(): Promise<BrowseResponse>
  async movie(slug: string): Promise<MovieDetail>
  async search(q: string, limit?: number): Promise<SearchResponse>
  async setProgress(slug: string, position: number, duration?: number): Promise<void>
  async setFavorite(slug: string, favorite?: boolean): Promise<void>
  async libraryStatus(): Promise<LibraryStatusResponse>
}
```

Errors: parse `{ error: { code, message } }` envelope from server.

React Query keys:

| Key | Endpoint |
|-----|----------|
| `['browse']` | `/api/browse` |
| `['movie', slug]` | `/api/movies/:slug` |
| `['search', q]` | `/api/search` |
| `['health']` | `/api/health` |

Invalidate `browse` + `movie` after player exit (progress may change continue watching).

---

## `appinfo.json` (draft)

Per [LG appinfo.json reference](https://webostv.developer.lge.com/develop/references/appinfo-json):

```json
{
  "id": "com.pacificnm.loon",
  "version": "0.1.0",
  "vendor": "Pacific NM",
  "type": "web",
  "main": "index.html",
  "title": "Loon",
  "icon": "icon.png",
  "largeIcon": "icon-large.png",
  "bgColor": "#090B10",
  "iconColor": "#7DD3FC",
  "handlesRelaunch": true,
  "deeplinking": false,
  "supportTouchMode": "full",
  "requiredPermissions": [
    "network.operation",
    "network.query"
  ]
}
```

Icons: 80×80 and 130×130 PNG. Design in W1.

---

## Vite config notes

```typescript
// vite.config.ts — conceptual
export default defineConfig({
  plugins: [react()],
  base: './',           // required for webOS file:// loading
  build: {
    outDir: 'dist',
    target: 'es2020',  // webOS Chromium baseline
  },
  server: {
    host: true,        // LAN dev against real server
  },
});
```

`base: './'` is critical — absolute `/assets/` paths break in packaged `.ipk`.

---

## webOS platform integration

### Back button

```typescript
// Prefer webOSTV.js when available
if (window.webOS?.platformBack) {
  window.webOS.platformBack(() => handleBack());
}
// Fallback: keydown 461
```

See [webOS Events](https://webostv.developer.lge.com/develop/references/webos-event).

### Lifecycle

| Event | Action |
|-------|--------|
| `webOSRelaunch` | Refresh browse if stale |
| App hidden | Pause video if playing |
| App visible | Resume UI state |

### Video

HTML5 `<video>` with byte-range MP4 from Loon server. Test MKV on target TV — if unsupported, server should prefer MP4 (document in setup guide).

No Luna Service video APIs in v1 — keep it simple.

---

## Build & deploy pipeline

```text
developer machine
  npm run build          → client/dist/
  ares-package dist      → com.pacificnm.loon_0.1.0_all.ipk
  ares-install --device tv *.ipk
  ares-inspect --device tv   → Chromium devtools
```

CI (`.github/workflows/ci.yml` extension):

```yaml
# client job — add to loon CI
- run: npm ci
  working-directory: client
- run: npm test
  working-directory: client
- run: npm run build
  working-directory: client
```

TV packaging remains manual until signing credentials exist.

---

## Performance targets

| Metric | Target |
|--------|--------|
| Cold start to Home | < 3s on webOS 6+ |
| Browse paint | < 1s after health OK |
| Poster load | Progressive; first row priority |
| Bundle size | < 500KB gzip (no source maps) |
| Player start | < 5s to first frame on LAN |

### Image strategy

1. Request posters through server proxy (cache hit = fast LAN)
2. `loading="lazy"` for off-screen cards
3. Prefetch detail backdrop on card focus (200ms debounce)

---

## Security

- Server URL is HTTP on LAN only (v1) — no mixed content issues on `http://` server
- No credentials stored in client v1
- CORS must allow TV origin — server already has CORS via nest-http-serve

---

## Resolved planning questions

| Question | Answer |
|----------|--------|
| Hero in v1? | **Yes** — use `browse.hero` above rows; default focus on first row |
| Trailer button? | **Hidden** until server route exists |
| Scan trigger in Settings? | **No** — read-only status via `/api/library/status`; scan is server-side admin |
| Spatial nav library? | **norigin** first; custom fallback if spike fails |
| Scan API shape? | Server streams SSE on `POST /api/library/scan` — client does not call in v1 |

---

## Implementation order (recap)

| Phase | Deliverable |
|-------|-------------|
| **W0** | Vite scaffold, norigin focus, one row, play video on TV |
| **W1** | Router, detail, settings, error states |
| **W2** | Full browse, hero, search, progress, favorites |
| **W3** | Side rail, animations, manual checklist sign-off |

See [README.md](README.md) implementation phases and [W0-SPIKE.md](W0-SPIKE.md) for the day-by-day spike plan.
