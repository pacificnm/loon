# Loon Admin Desktop

Thin Tauri + React admin UI for the **Loon backend API**. This is one of three Loon apps:

| App | Path | Role |
|-----|------|------|
| **Server** | `apps/loon/server` | Backend API (`/api/movies`, `/api/health`, …) |
| **Client** | `apps/loon/client` | LG webOS TV app |
| **Desktop** | `apps/loon/desktop` | This admin UI — HTTP to server only |

The desktop does **not** embed server code, webOS code, or a local API. It reads config and calls the remote backend.

## Config (required)

```bash
mkdir -p ~/.config/loon
cp config.example.toml ~/.config/loon/config.toml
# Edit [loon-admin].server_url — e.g. http://192.168.88.10:3000
```

Missing or invalid config → app exits immediately.

## Run

```bash
cd apps/loon/desktop
./build run      # build UI + release binary, launch
./build dev      # tauri dev (Vite :5173)
./build build    # production binary only
```

Binary: `target/release/loon-desktop`

## Verify backend

```bash
curl http://192.168.88.10:3000/api/health
```

Settings panel in the app shows the same health check.
