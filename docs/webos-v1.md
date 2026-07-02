# Loon webOS v1 Plan

## Status: Planned

Client plan for the **Loon webOS** app — LG TV experience built with Vite + React. Server API is defined in [v1.md](v1.md).

Product principles: [README](../README.md) — remote-first, 10-foot UI, Netflix-inspired browse.

## Context

webOS is the **only** supported client. The app is presentation-only; all library logic lives on `loon-server`.

```text
LG webOS TV
     │
React UI (focus / remote)
     │
HTTP JSON  +  video stream URL
     │
loon-server
```

## Repository layout

```text
webos/
├── appinfo.json           # webOS app manifest
├── package.json
├── vite.config.ts
├── index.html
└── src/
    ├── main.tsx
    ├── app/
    │   ├── App.tsx
    │   └── routes.tsx
    ├── api/
    │   ├── client.ts        # fetch wrapper, base URL
    │   ├── movies.ts
    │   └── browse.ts        # v0.2
    ├── screens/
    │   ├── Home.tsx         # hero + rows
    │   ├── Browse.tsx       # genre row drill-down
    │   ├── Detail.tsx
    │   └── Search.tsx       # v0.2
    ├── components/
    │   ├── MovieCard.tsx
    │   ├── MovieRow.tsx
    │   ├── HeroBanner.tsx
    │   └── FocusGrid.tsx    # remote navigation
    ├── player/
    │   ├── VideoPlayer.tsx
    │   └── progress.ts      # heartbeat to PUT /progress
    └── config.ts            # server base URL
```

## Server connection

### v0.1 — static base URL

```typescript
// config.ts — build-time or webOS app prefs
export const API_BASE = import.meta.env.VITE_LOON_SERVER ?? "http://192.168.1.10:3000";
```

User enters server IP once in settings (v0.2); v0.1 hard-code or env at build time.

### v0.2 — discovery (deferred)

| Approach | Notes |
|----------|-------|
| Manual IP entry | Settings screen; persist in `localStorage` |
| mDNS `loon.local` | Requires server Bonjour advertisement |
| webOS service discovery | Platform-specific |

## Screens & API mapping

| Screen | API (v0.1) | API (v0.2) |
|--------|------------|------------|
| **Home** | `GET /api/movies` (flat grid) | `GET /api/browse` (hero + rows) |
| **Detail** | `GET /api/movies/:slug` | same + favorite toggle |
| **Player** | `GET /stream/:slug` | + `PUT /progress` |
| **Search** | — | `GET /api/search?q=` |

### Home (v0.1 MVP)

Simple poster grid from `MoviesListResponse.movies`. No hero until v0.2 browse API.

### Home (v0.2)

- Hero: `BrowseResponse.hero` with backdrop image
- Rows: map `BrowseResponse.rows` → horizontal `MovieRow`
- Focus: one row at a time; left/right within row; up/down between rows

## Remote / focus management

webOS is **D-pad only** — no hover states.

| Key | Action |
|-----|--------|
| Arrow keys | Move focus between cards |
| Enter | Open detail / play |
| Back | Navigate up stack |

Use explicit `tabIndex` or a small focus library (e.g. `@noriginmedia/norigin-spatial-navigation` — evaluate in spike).

**Design targets:**

- Focus ring visible at 10-foot distance
- Minimum poster card size ~200px wide on 1080p
- Maximum 2–3 rows visible without scroll on home (v0.2)

## Video playback

### v0.1 approach

Use HTML5 `<video>` with stream URL:

```typescript
const url = `${API_BASE}/stream/${slug}`;
video.src = url;
video.play();
```

webOS Chromium supports MP4 byte-range; MKV support varies by TV — document in server direct-play policy.

### Progress / resume (v0.2)

```typescript
// On pause, back, or every 30s while playing
await api.saveProgress(slug, {
  position_seconds: Math.floor(video.currentTime),
  duration_seconds: Math.floor(video.duration),
});
```

On detail open: if `watch_progress_seconds > 0`, show **Resume** vs **Play from start**.

## webOS packaging

| Step | Tool |
|------|------|
| Build | `npm run build` → `dist/` |
| Package | webOS CLI `ares-package` → `.ipk` |
| Install | `ares-install` to TV on same LAN |
| Debug | webOS Inspector (Chromium devtools) |

`appinfo.json` essentials:

```json
{
  "id": "com.pacificnm.loon",
  "version": "0.1.0",
  "type": "web",
  "main": "index.html",
  "title": "Loon",
  "icon": "icon.png",
  "largeIcon": "icon-large.png"
}
```

## Implementation phases

### Phase W0 — Spike (1–2 days)

- Vite + React scaffold
- Hard-coded server URL
- Fetch `GET /api/movies`; render static grid
- Tap card → play stream in `<video>`

**Exit:** Movie plays on LG TV from real Loon server.

### Phase W1 — Browse + detail (M2)

- React Router: Home → Detail → Player
- `MovieDetail` screen (summary, cast, play button)
- Loading / error states using server error envelope
- Basic focus navigation on grid

### Phase W2 — Full UX (with server v0.2)

- `GET /api/browse` home with hero + rows
- Search screen
- Continue watching row
- Favorites indicator
- Settings: server URL
- Progress heartbeat

## Styling

- Dark theme default (OLED-friendly)
- Large typography (min 24px body at 1080p)
- Poster aspect ratio 2:3; backdrop 16:9 for hero
- No dense text blocks — summary truncated on detail

## Testing

| Test | Type |
|------|------|
| API client mock | Unit (vitest) |
| Focus navigation | Component |
| End-to-end on TV | Manual — primary proof |

Automated TV tests are limited; maintain a **manual test checklist** in repo.

## Deferred

| Feature | Target |
|---------|--------|
| mDNS server discovery | v0.3 |
| Multiple profiles | later |
| Offline mode | later |
| webOS voice | later |

## Related

- [Server v1 plan](v1.md)
- [Product README](../README.md)
