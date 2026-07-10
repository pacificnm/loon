# Loon Client (webOS) Documentation

Complete documentation for the Loon Client webOS application - a React-based TV interface for browsing and streaming movies.

---

## Documentation Index

### Getting Started

| Document | Description |
|----------|-------------|
| [Overview](./01-overview.md) | Architecture, project structure, and key concepts |
| [Configuration](../README.md) | Setup and build instructions |

### Technical Reference

| Document | Description |
|----------|-------------|
| [Components](./02-components.md) | UI component reference |
| [Pages](./03-pages.md) | Page component documentation |
| [API Client](./04-api-client.md) | API functions and types |
| [Platform](./05-platform.md) | webOS integration details |

---

## Quick Links

### For Developers

- **New to the project?** Start with [Overview](./01-overview.md)
- **Adding UI components?** Review [Components](./02-components.md)
- **Creating new pages?** Check [Pages](./03-pages.md)
- **API integration?** See [API Client](./04-api-client.md)
- **webOS specifics?** Read [Platform](./05-platform.md)

---

## Application Overview

**Loon Client** is a React web application for **LG webOS smart TVs** that provides:

- 🎬 **Movie Browser** - Alphabetically sorted library with A-Z navigation
- 🔍 **Search** - Title-based search with debounced input
- 📺 **Video Playback** - HTML5 video with Magic Remote controls
- ⭐ **Favorites** - Mark and browse favorite movies
- 🎭 **Actor Details** - Person pages with filmography
- 🗂️ **Genre Browse** - Navigate by genre categories
- ⚙️ **Admin Panel** - Library scanning and server configuration

### Technology Stack

| Layer | Technology |
|-------|------------|
| **Framework** | React 18 |
| **Routing** | react-router-dom 6 (HashRouter) |
| **Navigation** | @noriginmedia/norigin-spatial-navigation |
| **Build** | Vite 5 |
| **Language** | TypeScript 5 |
| **Platform** | LG webOS |

---

## Project Structure

```
client/
├── src/
│   ├── main.tsx              # Entry point, focus init
│   ├── App.tsx               # Root component, routes
│   ├── config.ts             # Server URL management
│   ├── api/                  # API client
│   ├── components/           # Reusable UI components
│   ├── pages/                # Page components
│   ├── player/               # Video player
│   ├── platform/             # webOS integration
│   ├── utils/                # Utilities
│   └── theme/                # CSS design tokens
├── package/                  # webOS package output
├── scripts/                  # Build/deploy scripts
├── appinfo.json              # webOS manifest
└── docs/                     # This documentation
```

---

## Routes

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

---

## Focus Navigation

The app uses **spatial navigation** optimized for TV remotes:

```
┌─────────────────────────────────────────────────────────┐
│  TopBar (Home | Search | Genres | Favorites | Admin)   │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  A  ┌─────────────────────────────────────────────┐    │
│     │ Alien (1979)                                │    │
│     │ 1979 · 117 min · Horror, Sci-Fi             │    │
│     │ In space no one can hear you scream.        │    │
│     └─────────────────────────────────────────────┘    │
│                                                         │
│  B  ┌─────────────────────────────────────────────┐    │
│     │ Blade Runner (1982)                         │    │
│     │ ...                                         │    │
│     └─────────────────────────────────────────────┘    │
│                                                         │
│  ...                                                    │
│                                                         │
│  ──── Index Rail (A-Z) ────                            │
│                                                         │
│  A B C D E F G H I J K L M N O P Q R S T U V W X Y Z # │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Navigation:**
- Arrow keys move focus between elements
- Enter selects focused element
- Back button navigates back/exits app

---

## Build & Deploy

```bash
cd apps/loon/client

# Development
npm run dev

# Production build
npm run build

# Package for webOS
npm run package:webos

# Launch in simulator
npm run launch:simulator
```

**Output:** `package/` directory containing:
- `index.html` - Bundled app
- `appinfo.json` - webOS manifest
- Icons

---

## Configuration

### Server URL

Configured via Admin → Settings or environment:

```bash
# .env file
VITE_LOON_SERVER=http://192.168.1.100:3000
```

Or at runtime:
1. Open Admin → Settings
2. Enter server URL
3. Press Save

### webOS App Info

**File:** `appinfo.json`

```json
{
  "id": "com.pacificnm.loon",
  "version": "0.3.5",
  "type": "web",
  "main": "index.html",
  "title": "Loon",
  "resolution": "1920x1080",
  "disableBackHistoryAPI": true,
  "handlesRelaunch": true
}
```

---

## Key Features

### Spatial Navigation

Uses `@noriginmedia/norigin-spatial-navigation` for TV remote support:

```typescript
const { ref, focused, focusSelf } = useFocusable({
  focusKey: 'unique-key',
  onEnterPress: () => handleSelect(),
  onFocus: (layout) => scrollIntoView(layout),
});

return <div ref={ref} className={focused ? 'focused' : ''}>...</div>;
```

### Server-Sent Events

Library scan progress streamed via SSE:

```typescript
await streamLibraryScan(serverUrl, { full: false }, (event) => {
  console.log(formatScanEvent(event));
  // [12:34:56] enriching: files 100, candidates 50, enriched 25/50
});
```

### Lifecycle Management

Handles webOS launch/relaunch events:

```typescript
useWebOsLifecycle(() => {
  setFocusEpoch((epoch) => epoch + 1);  // Reset focus state
});

useWebOsVisibility(() => {
  videoRef.current?.pause();  // Pause when backgrounded
});
```

---

## API Communication

All API calls through `src/api/client.ts`:

```typescript
// Fetch all movies
const movies = await fetchAllMovies(serverUrl);

// Fetch movie detail
const movie = await fetchMovie(serverUrl, 'alien-1979');

// Set favorite
await setFavorite(serverUrl, 'alien-1979', true);

// Search
const results = await searchMovies(serverUrl, 'alien');
```

---

## Related Projects

| App | Path | Role |
|-----|------|------|
| **Server** | `apps/loon/server` | Backend API |
| **Client** | `apps/loon/client` | This webOS app |
| **Desktop** | `apps/loon/desktop` | Admin desktop UI |

---

## Support

- **GitHub**: https://github.com/pacificnm/loon
- **Issues**: File on the main Nest repository

---

## License

MIT OR Apache-2.0 (see workspace root)
