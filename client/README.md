# Loon webOS Client — UI Plan

## Status: W0 implemented (TV verification pending)

TV-first UI specification for the **Loon webOS** client. All client source lives under this `client/` folder.

**W0 spike:** Vite + React app with movie row, spatial focus, and video player. See [W0-SPIKE.md](W0-SPIKE.md).

```bash
cd client
cp .env.example .env.local   # set VITE_LOON_SERVER
npm install
npm run dev                  # browser dev
npm run package:webos        # dist + appinfo → package/
ares-package -n package -o build   # -n: skip minify (Vite already minifies)
ares-install --device emulator build/*.ipk
```

**Planning docs (this folder)**

| Doc | Contents |
|-----|----------|
| [README.md](README.md) | Overview, palette, v1 scope (this file) |
| [SCREENS.md](SCREENS.md) | Per-screen layouts, states, API calls |
| [FOCUS.md](FOCUS.md) | Remote keys, focus trees, back behavior |
| [COMPONENTS.md](COMPONENTS.md) | Component catalog, sizes, TypeScript types |
| [TECH.md](TECH.md) | Stack, dependencies, appinfo.json, build |
| [W0-SPIKE.md](W0-SPIKE.md) | Day-by-day spike tasks before coding |

**Related docs**

- [Product README](../README.md) — vision and principles
- [Server API v0.2](../docs/api-v0.2.md) — browse, search, progress, artwork proxy
- [webOS v1 plan](../docs/webos-v1.md) — earlier scaffold notes (superseded by `client/` plans)
- [Manual test checklist](../docs/webos-test-checklist.md)
- [webOS knowledge plan](../docs/webos-knowledge-v1.md) — index LG docs into nest-knowledge MCP

**LG webOS references (required reading before implementation)**

- [Design Principles](https://webostv.developer.lge.com/develop/guides/design-principles) — lean-back, 10-foot UI, motion, consistency
- [References overview](https://webostv.developer.lge.com/develop/references) — platform APIs and packaging
  - [appinfo.json](https://webostv.developer.lge.com/develop/references/appinfo-json) — app manifest
  - [webOS Events](https://webostv.developer.lge.com/develop/references/webos-event) — lifecycle and platform events
  - [Luna Service API](https://webostv.developer.lge.com/develop/references/luna-service-introduction) — system services
  - [webOSTV.js](https://webostv.developer.lge.com/develop/references/webostvjs-introduction) — TV-specific JS APIs
  - [webos-service](https://webostv.developer.lge.com/develop/references/webos-service-introduction) — Node.js system bus (deferred)

---

## Design stance

Loon is **TV-first, not web-page-first**. The client is presentation-only; all library logic lives on `loon-server`.

```text
LG webOS TV (Magic Remote / D-pad)
        │
React UI — spatial focus, large targets, low text density
        │
HTTP JSON  +  artwork proxy  +  byte-range stream
        │
loon-server
```

### LG design principles (applied)

From [webOS Design Principles](https://webostv.developer.lge.com/develop/guides/design-principles):

| Principle | Loon application |
|-----------|------------------|
| **Keep it simple** | No more than two ideas per screen. Content owns the screen; navigation rail is hidden by default. |
| **Give it life** | Subtle focus scale/glow, row scroll momentum, backdrop fanart on detail — never dense chrome. |
| **Make it personal** | Continue Watching, Favorites, Recently Added rows; resume on detail and player. |
| **Consistency matters** | Same card sizes, focus ring, terminology, and remote behavior on every screen. |

### Non-negotiable remote rule

**Every screen must work perfectly with only Arrow keys, OK, and Back.**

No hover states. No mouse-first patterns. No tiny tap targets.

---

## Visual style

Loon should feel:

- **Dark** — OLED-friendly backgrounds
- **Cinematic** — backdrop fanart, poster-heavy layouts
- **Simple** — low text density, large typography
- **Poster-heavy** — artwork is the primary UI element

### Palette

| Token | Hex | Use |
|-------|-----|-----|
| Background | `#090B10` | App shell, player letterbox |
| Surface | `#141821` | Cards, overlays, control bar |
| Primary | `#7DD3FC` | Primary actions, links |
| Accent | `#A78BFA` | Secondary highlights |
| Text | `#F8FAFC` | Titles, body |
| Muted | `#94A3B8` | Year, metadata, hints |
| Focus | `#38BDF8` | Focus ring, glow |

### Typography & scale (1080p baseline)

- Body text: **24px minimum**
- Row titles: **28–32px**
- Screen titles / hero: **40–56px**
- Poster card width: **~200px** minimum (2:3 aspect)
- Episode thumbnail: **16:9** widescreen (not poster ratio)
- Focus ring: clearly visible at **10-foot** viewing distance

---

## Core layout — Home

Content-first shell. Top bar is minimal; no permanent left rail.

```text
┌────────────────────────────────────────────────────────────┐
│ Loon                         Search   Settings   Clock     │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Continue Watching                                         │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐              │
│  │ Poster │ │ Poster │ │ Poster │ │ Poster │  →           │
│  └────────┘ └────────┘ └────────┘ └────────┘              │
│                                                            │
│  Movies                                                    │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐              │
│                                                            │
│  TV Shows                                                  │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐              │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

### Home rows (horizontal carousels)

| Row | API source (v0.2) | Notes |
|-----|---------------------|-------|
| Continue Watching | `GET /api/browse` → `row_type: continue_watching` | Show progress badge on card when `watch_progress_seconds > 0` |
| Recently Added | `row_type: recently_added` | |
| Movies | All movies row or genre-agnostic browse row | |
| TV Shows | **UI placeholder** | Server is movies-only today; hide row when empty |
| Collections | **Deferred** | Not in API v0.2 |
| Unmatched / Needs Metadata | **Deferred** | Admin/repair flow — post-v1 |

Data loads from `GET /api/browse`. Search and Settings are top-bar destinations, not home rows.

### Poster card

```text
┌──────────────┐
│              │
│   Poster     │   ← /api/artwork/:slug/poster
│              │
└──────────────┘
 Title
 Year
```

- Poster via server artwork proxy (`poster_url` in API already points here)
- **Focused card:** scale ~1.08×, `#38BDF8` outline/glow, smooth 150–200ms transition
- **Selected (OK):** navigate to detail (movies) or show detail (TV — future)

### Row focus model

- **Up/Down:** move between rows (row title + first focused card snaps into view)
- **Left/Right:** move within row; row scrolls to keep focus card near center
- **OK:** open focused item
- **Back:** open left rail (see Navigation) or exit app from Home

---

## Main screens

### Home

Described above. Primary entry after launch and after Back from top-level destinations.

### Movie detail

```text
┌────────────────────────────────────────────────────────────┐
│  Background fanart (backdrop), dark gradient overlay       │
│                                                            │
│  Movie Poster     The Matrix                               │
│                   1999 • 2h 16m • R                        │
│                   Sci-Fi • Action                            │
│                                                            │
│                   [ Play ] [ Trailer ] [ More Info ]       │
│                                                            │
│                   Overview text here...                    │
└────────────────────────────────────────────────────────────┘
```

| Element | Source |
|---------|--------|
| Backdrop | `backdrop_url` → `/api/artwork/:slug/backdrop` |
| Metadata | `GET /api/movies/:slug` |
| Play | `GET /stream/:slug` — **default focus on Play** (or Resume when progress exists) |
| Trailer | **Hidden in v1** — no server route; do not show disabled button |
| More Info | Expand overview / cast (same screen, no new route) |
| Resume | If `watch_progress_seconds > 0`, show **Resume** before **Play from start** |

**Remote:** OK on Play starts player. Back returns to Home (preserving row focus if possible).

### TV show detail (planned UI)

```text
Show Poster     Show Title
                Overview

[ Play Next Episode ]

Season 1
Episode cards →
```

- Episode cards use **16:9 thumbnails**, not 2:3 posters
- **Backend note:** Loon server is **movies-only** in v0.2. Build screen structure and components now; wire when TV API exists. Until then, omit TV rows/screens or show empty state.

### Player overlay

```text
┌──────────────────────────────────────────────┐
│                                              │
│                 Video                        │
│                                              │
├──────────────────────────────────────────────┤
│  00:14:22 ━━━━━━━━━━━━━━━────── 01:42:00     │
│  ⏪   ▶/pause   ⏩      CC     Audio          │
└──────────────────────────────────────────────┘
```

| Remote key | Action |
|------------|--------|
| **OK** | Pause / play |
| **Left** | Seek back (~10s) |
| **Right** | Seek forward (~10s) |
| **Down** | Show controls overlay |
| **Up** | Show title/info overlay |
| **Back** | Hide overlay; second Back exits player |

**Progress:** heartbeat to `PUT /api/movies/:slug/progress` on pause, Back, and every 30s while playing.

Stream URL: `{server}/stream/{slug}` via HTML5 `<video>` (webOS Chromium; MP4 direct-play).

### Search

- Query: `GET /api/search?q=`
- Large search field; results as poster grid or single results row
- Debounce input (~300ms); D-pad moves across results like Home rows
- Back returns to previous screen

### Settings / Server URL

- Server base URL entry (e.g. `http://192.168.88.205:3000`)
- Persist in `localStorage` (v1); test connection via `GET /api/health`
- Optional: show server version from health response
- **Deferred:** mDNS / `loon.local` discovery

---

## Navigation model

**Do not keep a left rail always visible.** TV apps feel better when content owns the screen.

Open rail on demand (Back from Home, or dedicated menu key if mapped):

```text
☰
Home
Movies
TV Shows
Search
Settings
Server
```

| Item | Behavior |
|------|----------|
| Home | `GET /api/browse` |
| Movies | Filtered browse or full grid |
| TV Shows | Placeholder until API exists |
| Search | Search screen |
| Settings | Server URL, app info |
| Server | Connection status, read-only library stats — **no scan trigger in v1** |

Rail slides over content; dismiss with Back or Right into content.

---

## Minimum first version (v1 client)

Build **only** these screens first:

| Screen | Priority |
|--------|----------|
| Home | P0 |
| Movie detail | P0 |
| TV show detail | P1 (shell only; hide if no data) |
| Playback | P0 |
| Search | P1 |
| Settings / server URL | P0 |

### Explicitly deferred (post-v1)

- Metadata repair / Unmatched row
- Collections
- User profiles
- Watch history sync across devices
- Subtitles / audio track selector
- Admin tools (scan UI, library status SSE)
- Trailer playback
- mDNS server discovery
- webOS voice

---

## API mapping summary

| Screen | Endpoints |
|--------|-----------|
| Home | `GET /api/browse` |
| Movie detail | `GET /api/movies/:slug` |
| Search | `GET /api/search?q=` |
| Play / resume | `GET /stream/:slug`, `PUT /api/movies/:slug/progress` |
| Artwork | `GET /api/artwork/:slug/poster`, `.../backdrop` |
| Settings | `GET /api/health` (connection test) |
| Favorites (detail badge) | `PUT /api/movies/:slug/favorite` |

Full contract: [api-v0.2.md](../docs/api-v0.2.md).

---

## Planned folder layout

All client code under `client/`:

```text
client/
├── README.md                 # this file
├── appinfo.json              # webOS manifest (see LG appinfo.json reference)
├── package.json
├── vite.config.ts
├── index.html
└── src/
    ├── main.tsx
    ├── app/
    │   ├── App.tsx
    │   └── routes.tsx
    ├── api/
    │   ├── client.ts         # fetch wrapper, base URL from settings
    │   ├── browse.ts
    │   ├── movies.ts
    │   └── search.ts
    ├── screens/
    │   ├── Home.tsx
    │   ├── MovieDetail.tsx
    │   ├── ShowDetail.tsx    # shell until TV API
    │   ├── Search.tsx
    │   └── Settings.tsx
    ├── components/
    │   ├── PosterCard.tsx
    │   ├── ContentRow.tsx
    │   ├── TopBar.tsx
    │   ├── SideRail.tsx
    │   ├── FocusRing.tsx
    │   └── EpisodeCard.tsx   # 16:9
    ├── player/
    │   ├── VideoPlayer.tsx
    │   ├── PlayerOverlay.tsx
    │   └── progress.ts       # PUT /progress heartbeat
    ├── focus/
    │   └── spatial.ts        # D-pad focus manager
    ├── theme/
    │   └── tokens.ts           # palette above
    └── config.ts
```

**Tech stack:** Vite + React + TypeScript + TanStack Query. Spatial focus via `@noriginmedia/norigin-spatial-navigation` (W0 spike). Full detail: [TECH.md](TECH.md).

---

## Implementation phases

### W0 — Spike

See **[W0-SPIKE.md](W0-SPIKE.md)** for day-by-day tasks, prerequisites, and TV debug guide.

- Scaffold `client/` with Vite + React
- Settings: hard-coded or env server URL
- `GET /api/movies` or `/api/browse` → single poster row
- OK → play `GET /stream/:slug` in `<video>`

**Exit:** Movie plays on a real LG TV from loon-server.

### W1 — Core navigation

- React Router: Home → Movie detail → Player
- Spatial focus on rows and grid
- Loading / error states (server error envelope)
- Settings screen with persisted server URL

### W2 — Full v1 UX

- `GET /api/browse` with Continue Watching + Recently Added
- Search screen
- Movie detail with backdrop, resume, favorites
- Player overlay + progress heartbeat
- Artwork proxy images throughout

### W3 — Polish

- Side rail navigation
- Animations and focus polish per LG “Give it life”
- TV show detail shell (hidden until backend ready)
- Manual test checklist sign-off

---

## webOS packaging

| Step | Tool |
|------|------|
| Build | `npm run build` → `dist/` |
| Package | webOS CLI `ares-package` → `.ipk` |
| Install | `ares-install` to TV on same LAN |
| Debug | webOS Inspector (Chromium devtools) |

`appinfo.json` must follow [LG appinfo.json reference](https://webostv.developer.lge.com/develop/references/appinfo-json). Consider [webOSTV.js](https://webostv.developer.lge.com/develop/references/webostvjs-introduction) for platform integration when needed.

---

## Testing

| Type | Scope |
|------|-------|
| Unit | API client, progress helper, slug/url builders |
| Component | PosterCard focus states (jsdom + spatial mock) |
| Manual on TV | **Primary proof** — [webos-test-checklist.md](../docs/webos-test-checklist.md) |

Automated TV tests are limited; every release candidate runs the manual checklist on real hardware.

---

## Planning decisions (resolved)

| Topic | Decision |
|-------|----------|
| Hero banner | **In v1 (W2)** — `browse.hero` above carousels; default focus stays on first row |
| Trailer button | **Hidden** until server route exists |
| Library scan | **Read-only** in Settings (`GET /api/library/status`); no `POST /scan` from TV in v1 |
| Spatial nav | **norigin** first; custom `FocusManager` fallback if W0 TV checklist fails |
| Top bar on Home | Focusable row above hero (Search, Settings) — see [FOCUS.md](FOCUS.md) |
| Movies screen | Full paginated grid via `GET /api/movies` — see [SCREENS.md](SCREENS.md) |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-07-01 | Initial TV-first UI plan — screens, navigation, palette, v1 scope, LG references |
| 2026-07-02 | Added SCREENS, FOCUS, COMPONENTS, TECH; resolved open questions; aligned with live server API |
| 2026-07-02 | Added W0-SPIKE; rewrote webos-test-checklist for client/ screens and W0–W3 phases |
