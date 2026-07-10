# Loon Desktop - Architecture Overview

## What is Loon Desktop?

Loon Desktop is a **Tauri-based admin application** for managing the Loon media server. It provides a thin native shell around a React frontend that communicates with the Loon backend API.

## Application Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Loon Desktop App                         │
│  ┌─────────────────────┐    ┌─────────────────────────────┐│
│  │   React Frontend    │    │      Tauri Backend          ││
│  │   (TypeScript/TSX)  │◄──►│       (Rust)                ││
│  │   - App Shell       │    │   - Config loading          ││
│  │   - Library Panel   │    │   - IPC commands            ││
│  │   - Movie Detail    │    │   - Player window mgmt      ││
│  │   - Scan Panel      │    │   - External player launch  ││
│  │   - Settings Panel  │    │                             ││
│  └─────────┬───────────┘    └─────────────┬───────────────┘│
│            │                              │                │
│            │                              │                │
│            └──────────┬───────────────────┘                │
│                       │                                    │
│              IPC (Tauri Commands)                          │
└───────────────────────┼────────────────────────────────────┘
                        │
                        │ HTTP
                        ▼
              ┌──────────────────┐
              │  Loon Backend    │
              │  (Separate App)  │
              │  :3000           │
              └──────────────────┘
```

## Key Design Decisions

### 1. Thin Client Architecture
- **No embedded server code** - Desktop app only communicates via HTTP to the backend
- **No local database** - All data persists on the server
- **Config-driven** - Backend URL configured in `~/.config/loon/config.toml`

### 2. Dual Window System
- **Main window** - Admin UI for library management
- **Player window** - Dedicated video playback with HTML5 video element

### 3. Tauri Plugin Architecture
- Custom `loon` plugin exposes two commands:
  - `get_config` - Returns loaded configuration
  - `play_stream` - Launches player window with stream URL

## Project Structure

```
desktop/
├── src-tauri/                 # Rust/Tauri backend
│   ├── src/
│   │   ├── main.rs           # Application entry point
│   │   ├── commands.rs       # Tauri IPC commands
│   │   ├── config.rs         # Configuration loading
│   │   └── config_host.rs    # Config path resolution
│   ├── build.rs              # Tauri build script
│   ├── Cargo.toml            # Rust dependencies
│   ├── tauri.conf.json       # Tauri configuration
│   └── capabilities/
│       └── default.json      # Tauri permissions
├── ui/                        # React/TypeScript frontend
│   ├── src/
│   │   ├── App.tsx           # Main app component
│   │   ├── PlayerApp.tsx     # Player window component
│   │   ├── main.tsx          # App entry point
│   │   ├── player-main.tsx   # Player entry point
│   │   ├── components/       # React components
│   │   ├── hooks/            # Custom React hooks
│   │   ├── lib/              # Utility modules
│   │   └── types/            # TypeScript types
│   ├── index.html            # Main window HTML
│   ├── player.html           # Player window HTML
│   ├── package.json          # Node dependencies
│   ├── tsconfig.json         # TypeScript config
│   ├── tailwind.config.ts    # Tailwind CSS config
│   └── vite.config.ts        # Vite bundler config
├── crates/
│   └── loon-player/          # External player launcher crate
│       └── src/lib.rs        # ffplay/mpv integration
├── config.example.toml       # Example configuration
├── Cargo.toml                # Workspace root
└── build                     # Build script wrapper
```

## Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| **Backend** | Rust + Tauri 2.x | Native window management, IPC |
| **Frontend** | React 18 + TypeScript | UI components |
| **Styling** | Tailwind CSS | Custom design system |
| **Bundler** | Vite 5.x | Fast development builds |
| **Icons** | FontAwesome 6 | UI icons |
| **Player** | HTML5 Video / ffplay / mpv | Stream playback |

## Data Flow

### Loading Movies
```
1. App initializes → useApi hook
2. loadDesktopConfig() → Tauri invoke('plugin:loon|get_config')
3. fetchMovies() → HTTP GET /api/movies
4. Movies rendered in LibraryPanel
```

### Playing a Movie
```
1. User clicks "Play" in MovieDetail
2. playStream(slug, title) → Tauri invoke
3. commands.rs constructs stream URL
4. Player window shown via show_player_window()
5. player:load event emitted to player window
6. PlayerApp loads stream into <video> element
```

### Scanning Library
```
1. User clicks "Scan for Changes"
2. startScanStream(full) → HTTP POST /api/library/scan
3. SSE stream returns progress events
4. useScan hook updates UI progress
5. ScanPanel displays real-time status
```

## Build & Run

```bash
cd apps/loon/desktop

# Development mode (hot reload)
./build dev

# Run production build
./build run

# Build only
./build build

# Run tests
./build test

# Clean build artifacts
./build clean
```

## Configuration

Required configuration file at `~/.config/loon/config.toml`:

```toml
[loon-admin]
server_url = "http://192.168.88.10:3000"
# Optional: external player path
# player_path = "/usr/bin/ffplay"
```

See [Configuration Guide](./06-configuration.md) for details.

## Related Documentation

- [Rust Backend](./02-rust-backend.md) - Tauri commands, config, player launcher
- [React Frontend](./03-react-frontend.md) - App structure, routing, state
- [UI Components](./04-components.md) - Component reference
- [API Reference](./05-api-reference.md) - Hooks, API functions, types
- [Configuration](./06-configuration.md) - Config file format, options
