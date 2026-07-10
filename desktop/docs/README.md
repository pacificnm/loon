# Loon Desktop Documentation

Complete documentation for the Loon Admin Desktop application - a Tauri-based admin UI for the Loon media server.

---

## Documentation Index

### Getting Started

| Document | Description |
|----------|-------------|
| [Overview](./01-overview.md) | Architecture, project structure, and key concepts |
| [Configuration](./06-configuration.md) | Setup and configuration guide |

### Technical Reference

| Document | Description |
|----------|-------------|
| [Rust Backend](./02-rust-backend.md) | Tauri commands, config loading, player launcher |
| [React Frontend](./03-react-frontend.md) | App structure, components, state management |
| [UI Components](./04-components.md) | Component reference with props and examples |
| [API Reference](./05-api-reference.md) | Library functions, hooks, and TypeScript types |

---

## Quick Links

### For Developers

- **New to the project?** Start with [Overview](./01-overview.md)
- **Setting up locally?** See [Configuration](./06-configuration.md)
- **Adding features?** Review [React Frontend](./03-react-frontend.md) and [UI Components](./04-components.md)
- **Debugging backend?** Check [Rust Backend](./02-rust-backend.md)

### For Maintainers

- [Rust Backend](./02-rust-backend.md) - IPC commands, config validation
- [API Reference](./05-api-reference.md) - Type definitions, function signatures

---

## Application Overview

**Loon Desktop** is a native admin application built with Tauri that provides:

- 📚 **Library Management** - Browse, search, and manage movies
- 🎬 **Video Playback** - Built-in player window for streaming content
- 🔄 **Library Scanning** - Real-time scan progress with SSE streaming
- ⚙️ **Configuration** - Backend connection and player settings

### Technology Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust + Tauri 2.x |
| Frontend | React 18 + TypeScript |
| Styling | Tailwind CSS (custom theme) |
| Bundler | Vite 5.x |

---

## Project Structure

```
desktop/
├── src-tauri/           # Rust/Tauri backend
│   ├── src/
│   │   ├── main.rs      # Entry point
│   │   ├── commands.rs  # IPC commands
│   │   ├── config.rs    # Config loading
│   │   └── config_host.rs
│   ├── build.rs
│   ├── tauri.conf.json
│   └── Cargo.toml
├── ui/                  # React frontend
│   ├── src/
│   │   ├── App.tsx
│   │   ├── PlayerApp.tsx
│   │   ├── components/
│   │   ├── hooks/
│   │   ├── lib/
│   │   └── types/
│   ├── package.json
│   └── vite.config.ts
├── crates/
│   └── loon-player/     # External player crate
└── docs/                # This documentation
```

---

## Build & Run

```bash
cd apps/loon/desktop

# Development mode
./build dev

# Run production build
./build run

# Build only
./build build

# Run tests
./build test

# Clean
./build clean
```

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────┐
│                  Loon Desktop App                   │
│  ┌─────────────────────┐    ┌─────────────────────┐│
│  │   React Frontend    │◄──►│    Tauri Backend    ││
│  │   (TypeScript)      │ IPC │      (Rust)        ││
│  └─────────┬───────────┘    └──────────┬──────────┘│
│            │                           │           │
│            │ HTTP                      │ IPC       │
│            ▼                           ▼           │
│  ┌──────────────────┐         ┌──────────────────┐ │
│  │   Loon Backend   │         │  Player Window   │ │
│  │   (Separate)     │         │  (HTML5 Video)   │ │
│  └──────────────────┘         └──────────────────┘ │
└─────────────────────────────────────────────────────┘
```

---

## Key Concepts

### Thin Client
Desktop app contains **no server code** - it only communicates via HTTP to the Loon backend API.

### Config-Driven
All configuration lives in `~/.config/loon/config.toml`. Missing config = immediate exit.

### Dual Window
- **Main window**: Admin UI for library management
- **Player window**: Dedicated video playback with loading states

### IPC Communication
Frontend calls Rust backend via Tauri commands:
- `plugin:loon|get_config` - Load configuration
- `plugin:loon|play_stream` - Launch player

---

## Related Projects

| App | Path | Role |
|-----|------|------|
| **Server** | `apps/loon/server` | Backend API (`/api/movies`, `/api/health`) |
| **Client** | `apps/loon/client` | LG webOS TV app |
| **Desktop** | `apps/loon/desktop` | This admin UI |

---

## Support

- **GitHub**: https://github.com/pacificnm/loon
- **Issues**: File on the main Nest repository

---

## License

MIT OR Apache-2.0 (see workspace root)
