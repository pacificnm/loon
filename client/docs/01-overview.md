# Loon Client (webOS) - Architecture Overview

## What is Loon Client?

Loon Client is a **React-based web application** designed for **LG webOS smart TVs**. It provides a 10-foot user interface for browsing and streaming movies from the Loon Server backend.

## Application Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  Loon Client (webOS)                        │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  React UI Components                                  │  │
│  │  • MovieAlphabetList  • HorizontalRow                 │  │
│  │  • PosterCard         • FocusButton                   │  │
│  │  • MovieVerticalList  • VideoPlayer                   │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Spatial Navigation (Norigin)                         │  │
│  │  • Focus management for TV remote                     │  │
│  │  • Arrow key navigation                               │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Platform Layer                                       │  │
│  │  • webOS lifecycle (launch/relaunch)                  │  │
│  │  • Magic Remote back button                           │  │
│  │  • Visibility handling                                │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ HTTP
                              ▼
                    ┌──────────────────┐
                    │  Loon Server     │
                    │  (Separate App)  │
                    └──────────────────┘
```

## Key Design Decisions

### 1. Spatial Navigation for TV Remote
- Uses `@noriginmedia/norigin-spatial-navigation` library
- Focus-based navigation (no mouse/touch)
- Arrow keys move focus between focusable elements
- Visual focus indicators for TV viewing distance

### 2. Hash-Based Routing
- Uses `react-router-dom` with `HashRouter`
- Required for webOS web app deployment
- Routes: `/`, `/movie/:slug`, `/play/:slug`, `/admin`, etc.

### 3. Server URL Configuration
- Stored in `localStorage` (persists across app restarts)
- Configurable via Admin → Settings page
- Environment variable fallback (`VITE_LOON_SERVER`)

### 4. Lifecycle Management
- Handles webOS `relaunch` events (`handlesRelaunch: true` in appinfo.json)
- Calls `PalmSystem.activate()` to show app
- Pauses video on visibility change (app backgrounded)

## Project Structure

```
client/
├── src/
│   ├── main.tsx              # Entry point, focus context init
│   ├── App.tsx               # Root component, route definitions
│   ├── config.ts             # Server URL management
│   ├── api/
│   │   ├── client.ts         # API client functions
│   │   ├── types.ts          # TypeScript interfaces
│   │   ├── sse.ts            # Server-Sent Events parser
│   │   └── normalize.ts      # API response normalization
│   ├── components/
│   │   ├── layout/
│   │   │   ├── AppShell.tsx  # Main layout with TopBar
│   │   │   └── TopBar.tsx    # Navigation header
│   │   ├── FocusButton.tsx   # Focusable button/tile
│   │   ├── HorizontalRow.tsx # Browse row component
│   │   ├── MovieAlphabetList.tsx  # A-Z movie list
│   │   ├── MovieVerticalList.tsx  # Vertical movie list
│   │   └── PosterCard.tsx    # Movie poster card
│   ├── pages/
│   │   ├── MoviesPage.tsx    # Home page (alphabet list)
│   │   ├── MovieDetailPage.tsx  # Movie details
│   │   ├── MovieEditPage.tsx # TMDB match editor
│   │   ├── PlayerPage.tsx    # Video player
│   │   ├── SearchPage.tsx    # Search interface
│   │   ├── FavoritesPage.tsx # Favorites list
│   │   ├── GenresPage.tsx    # Genre browser
│   │   ├── PersonPage.tsx    # Actor details
│   │   ├── AdminPage.tsx     # Admin panel
│   │   └── AdminSettingsTab.tsx  # Server URL config
│   ├── player/
│   │   └── VideoPlayer.tsx   # HTML5 video player
│   ├── platform/
│   │   ├── webos.ts          # webOS platform APIs
│   │   ├── useWebOsLifecycle.ts  # Lifecycle hook
│   │   ├── useWebOsBack.ts   # Back button handler
│   │   └── keyboard.ts       # Key code utilities
│   ├── utils/
│   │   ├── alphabet.ts       # Letter grouping
│   │   ├── format.ts         # Formatting utilities
│   │   └── scanLog.ts        # Scan log formatter
│   └── theme/
│       └── tokens.css        # CSS design tokens
├── package/                  # webOS package output
├── scripts/                  # Build/deploy scripts
├── appinfo.json              # webOS app manifest
├── index.html                # HTML entry point
└── docs/                     # This documentation
```

## Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| **Framework** | React 18 | UI components |
| **Routing** | react-router-dom 6 | Hash-based routing |
| **Navigation** | @noriginmedia/norigin-spatial-navigation | TV remote focus |
| **Build** | Vite 5 | Bundling, dev server |
| **Language** | TypeScript 5 | Type safety |
| **Testing** | Vitest | Unit tests |
| **Platform** | LG webOS | TV runtime |

## Pages/Routes

| Route | Component | Description |
|-------|-----------|-------------|
| `/` | `MoviesPage` | Home - alphabetically sorted movies |
| `/search` | `SearchPage` | Search by title |
| `/genres` | `GenresPage` | Genre browser |
| `/genre/:name` | `MoviesPage` | Movies in genre |
| `/favorites` | `FavoritesPage` | Favorited movies |
| `/movie/:slug` | `MovieDetailPage` | Movie details |
| `/movie/:slug/edit` | `MovieEditPage` | TMDB match editor |
| `/person/:tmdbId` | `PersonPage` | Actor details |
| `/play/:slug` | `PlayerPage` | Video playback |
| `/admin` | `AdminPage` | Admin panel |

## Focus Navigation

The app uses a **focus-based navigation model** optimized for TV remotes:

```tsx
// Every focusable element wraps with useFocusable
const { ref, focused, focusSelf } = useFocusable({
  focusKey: 'unique-key',
  onEnterPress: () => handleSelect(),
  onFocus: (layout) => scrollIntoView(layout),
});

return <div ref={ref} className={focused ? 'focused' : ''}>...</div>;
```

**Focus Context Hierarchy:**
```
root (focus boundary)
├── AppShell
│   └── TopBar (navigation)
└── Page Content
    ├── MovieAlphabetList
    │   ├── indexRail (A-Z letters)
    │   └── list (movie rows)
    └── HorizontalRow
        └── cards (posters)
```

## API Communication

All API calls go through `src/api/client.ts`:

```typescript
// Fetch all movies (paginated, sorted by title)
const movies = await fetchAllMovies(serverUrl);

// Fetch single movie detail
const movie = await fetchMovie(serverUrl, 'alien-1979');

// Set favorite
await setFavorite(serverUrl, 'alien-1979', true);

// Stream library scan (SSE)
await streamLibraryScan(serverUrl, { full: false }, (event) => {
  console.log(formatScanEvent(event));
});
```

## Build & Deploy

```bash
cd apps/loon/client

# Development (with Vite dev server)
npm run dev

# Production build
npm run build

# Package for webOS (creates package/ directory)
npm run package:webos

# Launch in webOS simulator
npm run launch:simulator
```

**Output:** `package/` directory containing:
- `index.html` - Bundled app
- `appinfo.json` - webOS manifest
- `icon.png`, `icon-large.png` - App icons

## Configuration

### Server URL

Configured via Admin → Settings or environment:

```bash
# .env file
VITE_LOON_SERVER=http://192.168.1.100:3000
```

Or at runtime (stored in localStorage):
1. Open Admin → Settings
2. Enter server URL
3. Press Save

### webOS App Info

**File:** `appinfo.json`

```json
{
  "id": "com.pacificnm.loon",
  "version": "0.3.5",
  "vendor": "Pacific NM",
  "type": "web",
  "main": "index.html",
  "title": "Loon",
  "resolution": "1920x1080",
  "disableBackHistoryAPI": true,
  "handlesRelaunch": true
}
```

## Related Documentation

- [Components](./02-components.md) - UI component reference
- [Pages](./03-pages.md) - Page component documentation
- [API Client](./04-api-client.md) - API functions and types
- [Platform](./05-platform.md) - webOS integration details
