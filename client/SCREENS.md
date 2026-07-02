# Loon Client — Screen Specifications

Companion to [README.md](README.md). Per-screen layout, states, API calls, and remote behavior.

---

## App bootstrap

### First launch (no server URL)

```text
┌────────────────────────────────────────────────────────────┐
│                                                            │
│                      Loon                                  │
│                                                            │
│         Connect to your Loon server                        │
│                                                            │
│         ┌──────────────────────────────────────┐           │
│         │ http://192.168.1.10:3000             │           │
│         └──────────────────────────────────────┘           │
│                                                            │
│                    [ Test & Save ]                         │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

| Step | Behavior |
|------|----------|
| Show | `localStorage.loon_server_url` missing or invalid |
| Focus | URL field (virtual keyboard on webOS) |
| Test & Save | `GET {url}/api/health` → on success persist URL → Home |
| Error | Inline message: "Could not reach server" |

### Returning launch

1. Read `loon_server_url` from `localStorage`
2. Prefetch `GET /api/browse` (show splash / skeleton)
3. On `503 library_scanning` → **Scanning screen** (below)
4. On network error → **Offline screen** with link to Settings
5. On success → **Home**

### Scanning screen (catalog not ready)

```text
┌────────────────────────────────────────────────────────────┐
│                                                            │
│              Updating your library…                        │
│                                                            │
│              ████████░░░░░░░░  42%                         │
│              (optional — poll /api/library/status)         │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

- Poll `GET /api/library/status` every 2s while `scan_in_progress`
- When `state: idle` and `movies_count > 0` → Home
- Back → exit app (webOS standard)

---

## Home

### Layout zones

| Zone | Height | Content |
|------|--------|---------|
| Top bar | 72px | Logo, Search, Settings, clock |
| Hero (optional) | 40vh max | `BrowseResponse.hero` backdrop + title |
| Row stack | remainder | Vertical list of `ContentRow` |

### Row order (from server)

Server returns rows in this order; client renders as-is:

1. Continue Watching (`continue_watching`)
2. Recently Added (`recently_added`)
3. Favorites (`favorites`) — if non-empty
4. Genre rows (`genre`) — one per genre with ≥3 movies

**Client-only rows (hidden until data exists):**

- Movies — full library shortcut → Movies screen
- TV Shows — hidden (no API)
- Collections — deferred

### Hero banner (v1 — included)

```text
┌────────────────────────────────────────────────────────────┐
│ ░░░░░░░░░░░░░ backdrop (16:9 crop) ░░░░░░░░░░░░░░░░░░░░░░ │
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │
│  Blade Runner                                              │
│  1982 • Sci-Fi                                             │
│  [ Watch ]                                                 │
└────────────────────────────────────────────────────────────┘
```

| Property | Value |
|----------|-------|
| Source | `browse.hero` |
| Image | `hero.backdrop_url` (artwork proxy) |
| Gradient | Bottom 60%: `linear-gradient(transparent, #090B10)` |
| Default focus | **First carousel row** (not hero) — lean-back browse first |
| Up from row 1 | Focus hero → Watch button |
| OK on Watch | Movie detail (same as selecting hero movie in a row) |

### Top bar items

| Item | Key | Action |
|------|-----|--------|
| Search | Right ×1 from logo area | Navigate to Search |
| Settings | Right ×2 | Navigate to Settings |
| Clock | Display only | `Intl.DateTimeFormat` local time, updates each minute |

Top bar is **not** in the main focus tree on Home — map **color buttons** or long-press pattern later; v1: reach Search/Settings via **side rail** or dedicated keys from hero zone. Alternative v1: make top bar focusable as a horizontal row above carousels (Search, Settings only).

**Decision (v1):** Top bar icons are focusable as **row −1** above hero. Down enters hero or first carousel.

### States

| State | UI |
|-------|-----|
| Loading | 3 skeleton rows, shimmer animation |
| Empty library | "No movies yet" + hint to scan on server |
| Error | Full-screen retry + Settings link |
| Success | Hero + rows |

### API

- `GET /api/browse` — single call on mount and on return from player (refresh continue watching)

---

## Movies (full grid)

Reached from side rail **Movies** or future "See all" on a row.

```text
┌────────────────────────────────────────────────────────────┐
│  Movies                                    Sort: Title ▼  │
├────────────────────────────────────────────────────────────┤
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐               │
│  │    │ │    │ │    │ │    │ │    │ │    │               │
│  └────┘ └────┘ └────┘ └────┘ └────┘ └────┘               │
│  ┌────┐ ┌────┐ ...                                         │
└────────────────────────────────────────────────────────────┘
```

| Property | Value |
|----------|-------|
| API | `GET /api/movies?page=&limit=50&sort=title` |
| Layout | 6 columns @ 1080p, 10 @ 4K (see [COMPONENTS.md](COMPONENTS.md)) |
| Focus | 2D grid: arrows move cell; OK → detail |
| Pagination | Load next page when focus reaches last row edge |
| Sort | `title` \| `year` \| `recently_added` — cycle with OK on sort control |
| Genre filter | Deferred v1.1 — `GET /api/genres` exists |

---

## Movie detail

### Layout

```text
┌────────────────────────────────────────────────────────────┐
│▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│
│▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ backdrop + dark overlay ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│
│                                                            │
│  ┌────────┐   The Matrix                          ♥       │
│  │ poster │   1999 • 2h 16m                               │
│  │ 2:3    │   Sci-Fi • Action                             │
│  └────────┘                                                │
│              [ Resume ]  [ Play ]  [ More Info ]           │
│                                                            │
│              In a dystopian future... (3 lines max)          │
└────────────────────────────────────────────────────────────┘
```

### Action buttons

| Button | When shown | Action |
|--------|------------|--------|
| Resume | `watch_progress_seconds > 0` and not finished | Player at saved position |
| Play | always | Player from start |
| More Info | always | Expand cast + full summary in-place |
| Trailer | **hidden** | Deferred — no server route |

**Default focus:** Resume if present, else Play.

### Favorite (♥)

- Toggle on OK when focused
- `PUT /api/movies/:slug/favorite` (empty body = toggle)
- Filled heart when `is_favorite: true`

### More Info expansion

Same screen — no navigation. Down from buttons moves to cast list (horizontal scroll of names). Back collapses expansion before leaving screen.

### API

- `GET /api/movies/:slug` on mount
- Uses `stream_url` from response (relative — prepend server base)

### States

| State | UI |
|-------|-----|
| Loading | Poster skeleton + shimmer |
| Not found | "Movie not found" + Back |
| Success | Full layout |

---

## TV show detail (shell — P1)

Hidden from navigation until TV API exists. Component structure ready:

- Show poster (2:3), title, overview
- Play Next Episode CTA
- Season selector (vertical)
- Episode row (16:9 `EpisodeCard`)

No API calls in v1 — screen not routed.

---

## Search

```text
┌────────────────────────────────────────────────────────────┐
│  Search                                                    │
│  ┌────────────────────────────────────────────────────┐    │
│  │ alien_                                             │    │
│  └────────────────────────────────────────────────────┘    │
│                                                            │
│  Results                                                   │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐ →                            │
└────────────────────────────────────────────────────────────┘
```

| Property | Value |
|----------|-------|
| Input | webOS virtual keyboard; min 2 chars |
| Debounce | 300ms before `GET /api/search?q=` |
| Results | Single horizontal row (or wrap grid if >8) |
| Empty query | Show recent searches from `localStorage` (max 5) |
| No results | "No matches for …" |
| OK on result | Movie detail |
| Back | Previous screen |

---

## Settings

```text
┌────────────────────────────────────────────────────────────┐
│  Settings                                                  │
│                                                            │
│  Server                                                    │
│  ┌────────────────────────────────────────────────────┐    │
│  │ http://192.168.88.205:3000                         │    │
│  └────────────────────────────────────────────────────┘    │
│  Status: Connected • v0.2.0 • 142 movies                 │
│  [ Test Connection ]                                       │
│                                                            │
│  About                                                     │
│  Loon client 0.1.0                                         │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

| Field | Source |
|-------|--------|
| Server URL | Editable, saved to `localStorage` |
| Status line | `GET /api/health` → `version`, `movies_count` |
| Test Connection | Re-fetch health; show error inline |
| Library scan | **Not in v1** — server admin only |
| Last scan | `GET /api/library/status` → `last_scan_at` (read-only display) |

Side rail **Server** item navigates here with focus on Server section.

---

## Player

Full-screen video with two overlay layers.

### Layers

| Layer | Z-index | Visibility |
|-------|---------|------------|
| Video | 0 | Always |
| Info overlay (Up) | 1 | Title, year, elapsed — auto-hide 5s |
| Controls overlay (Down) | 2 | Scrubber, transport — auto-hide 5s |

### Transport

| Control | Key | Behavior |
|---------|-----|----------|
| Play/Pause | OK | Toggle playback |
| Seek −10s | Left | `currentTime -= 10` |
| Seek +10s | Right | `currentTime += 10` |
| CC | — | Disabled / hidden v1 |
| Audio | — | Disabled / hidden v1 |

### Scrubber

- Focusable when controls visible
- Left/Right: ±30s when scrubber focused (vs ±10s when on play button)
- Show `position / duration` from `video.currentTime` / `video.duration`

### Progress persistence

| Event | Action |
|-------|--------|
| Every 30s while playing | `PUT /api/movies/:slug/progress` |
| Pause | Save progress |
| Back (exit player) | Save progress, navigate back |
| `ended` | Save with position = duration (server clears if >90%) |

### Resume on enter

If opened via **Resume** button: `video.currentTime = watch_progress_seconds` on `loadedmetadata`.

### Error states

| Error | UI |
|-------|-----|
| Stream 404 | Toast + return to detail |
| Codec unsupported | "Format not supported on this TV" + Back |
| Network drop | Pause + "Connection lost" overlay with retry |

### API

- Stream: `{server}{movie.stream_url}` → `/stream/:slug`
- Progress: `PUT /api/movies/:slug/progress`

---

## Side rail

```text
┌──────────┬─────────────────────────────────────────────────┐
│ ☰        │                                                 │
│ ● Home   │   (dimmed content behind)                       │
│   Movies │                                                 │
│   TV     │                                                 │
│   Search │                                                 │
│   Settings│                                                │
│   Server │                                                 │
└──────────┴─────────────────────────────────────────────────┘
```

| Property | Value |
|----------|-------|
| Width | 280px |
| Open | Back on Home (first press) OR dedicated menu |
| Close | Back again, or Right into content |
| Focus | Vertical list; OK navigates and closes rail |
| TV Shows | Hidden in v1 |
| Animation | Slide in 200ms ease-out; content scales to 0.95 + dim |

---

## Screen map

```text
                    ┌─────────────┐
                    │  Bootstrap  │
                    └──────┬──────┘
           no URL          │ has URL
              ▼            ▼
        ┌──────────┐  ┌──────────┐     scanning
        │ Settings │  │   Home   │◄────────────┐
        └──────────┘  └────┬─────┘             │
                           │                    │
         ┌─────────────────┼─────────────────┐  │
         ▼                 ▼                 ▼  │
   ┌──────────┐    ┌────────────┐    ┌──────────┐
   │  Search  │    │Movie Detail│    │  Movies  │
   └────┬─────┘    └─────┬──────┘    └────┬─────┘
        │                │                │
        └────────────────┼────────────────┘
                           ▼
                    ┌────────────┐
                    │   Player   │
                    └────────────┘
```
