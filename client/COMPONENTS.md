# Loon Client — Component Catalog

Reusable UI building blocks. Dimensions assume **1920×1080**; scale proportionally at 4K (webOS reports CSS pixels).

---

## Layout primitives

### `AppShell`

Full-viewport wrapper. Background `#090B10`. Renders `TopBar` + `Outlet` + optional `SideRail`.

### `TopBar`

| Prop | Type | Notes |
|------|------|-------|
| `showClock` | boolean | default true |

Height 72px, horizontal padding 48px. Logo text "Loon" 32px semibold.

### `SideRail`

| Prop | Type | Notes |
|------|------|-------|
| `open` | boolean | |
| `activeItem` | `NavItem` | |
| `onClose` | () => void | |
| `onNavigate` | (item) => void | |

Items: `home | movies | search | settings | server`. TV Shows omitted v1.

---

## Content components

### `PosterCard`

Primary browse unit.

```text
Width:  200px
Poster: 200 × 300px (2:3)
Title:  24px, max 2 lines, ellipsis
Year:   20px muted
Gap:    8px between poster and title
```

| Prop | Type | Notes |
|------|------|-------|
| `movie` | `MovieSummary` | |
| `focused` | boolean | scale + ring |
| `progress` | number? | 0–1 for continue watching bar |
| `onSelect` | () => void | |

**Continue watching badge:** 4px bar at bottom of poster, width = `progress * 100%`, color `#7DD3FC`.

**Image loading:** `loading="lazy"`, placeholder `#141821` with film icon. On error → initials on `#141821`.

### `EpisodeCard`

| Prop | Type | Notes |
|------|------|-------|
| `episode` | TBD | |
| `focused` | boolean | |

Thumbnail 280 × 158px (16:9). Title + SxxExx below. Shell only v1.

### `ContentRow`

Horizontal scrolling row with title.

| Prop | Type | Notes |
|------|------|-------|
| `title` | string | 28px, margin-bottom 16px |
| `movies` | `MovieSummary[]` | |
| `focusedIndex` | number | controlled by spatial nav |
| `onFocusIndex` | (i) => void | |
| `onSelect` | (movie) => void | |

Row height: poster + title + 48px vertical margin. Overflow-x auto, hidden scrollbar.

### `HeroBanner`

| Prop | Type | Notes |
|------|------|-------|
| `movie` | `MovieSummary` | |
| `focused` | boolean | highlights Watch button |
| `onWatch` | () => void | |

Max height 40vh. Backdrop `object-fit: cover`. Bottom gradient overlay.

### `MovieGrid`

Paginated grid for Movies screen.

| Prop | Type | Notes |
|------|------|-------|
| `movies` | `MovieSummary[]` | |
| `columns` | number | 6 @ 1080p |
| `onLoadMore` | () => void | |
| `onSelect` | (movie) => void | |

Gap 24px. Same `PosterCard` as rows.

---

## Detail components

### `MovieDetailHeader`

Poster 240×360 + metadata block side-by-side. Backdrop full-bleed behind at 30% opacity + blur optional W3.

### `ActionButton`

| Prop | Type | Notes |
|------|------|-------|
| `label` | string | |
| `variant` | `primary \| secondary` | primary = filled `#7DD3FC` on `#141821` |
| `focused` | boolean | |

Min size 180 × 56px. Font 24px. Gap 16px between buttons.

### `FavoriteButton`

Toggle heart. 48×48 touch target. Filled `#A78BFA` when active.

### `CastRow`

Horizontal list of cast names. Each chip 高度 40px, padding 12px 20px, surface background.

---

## Player components

### `VideoPlayer`

| Prop | Type | Notes |
|------|------|-------|
| `src` | string | stream URL |
| `startAt` | number? | seconds |
| `onProgress` | (pos, dur) => void | |
| `onEnded` | () => void | |
| `onError` | (err) => void | |

Native `<video>` full viewport. `playsInline`. webOS: test `video.webkitSetPresentationMode` if needed.

### `PlayerControls`

Transport bar. Surface `#141821` at 90% opacity, 120px tall, bottom anchored.

### `Scrubber`

| Prop | Type | Notes |
|------|------|-------|
| `position` | number | seconds |
| `duration` | number | |
| `focused` | boolean | thicker thumb when focused |

Track height 6px; thumb 16px circle `#38BDF8`.

### `PlayerInfoOverlay`

Top gradient + title + year. Shown on Up. Auto-hide 5s.

---

## Feedback components

### `SkeletonCard`

Poster-shaped `#141821` with CSS shimmer animation. Same dimensions as `PosterCard`.

### `EmptyState`

Icon + message + optional action button. Centered vertically.

### `ErrorBanner`

Inline surface bar, accent border left 4px `#f87171`, message 24px.

### `SplashScreen`

Logo + spinner on bootstrap. Max 2s unless network slow.

---

## TypeScript types (mirror server)

```typescript
interface MovieSummary {
  slug: string;
  title: string;
  year?: number;
  runtime_minutes: number;
  poster_url?: string;
  backdrop_url?: string;
  summary: string;
}

interface MovieDetail {
  slug: string;
  title: string;
  original_title?: string;
  year?: number;
  runtime_minutes?: number;
  summary?: string;
  genres: string[];
  poster_url?: string;
  backdrop_url?: string;
  cast: { name: string; character?: string }[];
  crew: { name: string; job?: string }[];
  is_favorite: boolean;
  watch_progress_seconds?: number;
  stream_url: string;
}

interface BrowseResponse {
  hero?: MovieSummary;
  rows: BrowseRow[];
}

interface BrowseRow {
  slug: string;
  title: string;
  row_type:
    | 'continue_watching'
    | 'recently_added'
    | 'favorites'
    | 'genre';
  movies: MovieSummary[];
}

interface HealthResponse {
  status: string;
  service: string;
  version: string;
  movies_count: number;
  library_scanned_at: number;
}
```

Live in `src/api/types.ts`.

---

## 4K / overscan

LG TVs may overscan ~5%. Safe margin: **48px** horizontal padding on all screens (already in TopBar). Test with [webOS overscan guide](https://webostv.developer.lge.com/develop/guides/design-style).

At 3840×2160, double poster size to 400px or keep 200px with more columns — **decision:** keep 200px cards, more visible per row (cinematic density).
