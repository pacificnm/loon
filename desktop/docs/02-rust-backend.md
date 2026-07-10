# Rust Backend Documentation

## Module Structure

```
src-tauri/src/
├── main.rs           # Application entry point
├── commands.rs       # Tauri IPC command handlers
├── config.rs         # Configuration structure and loading
└── config_host.rs    # Config path resolution
```

---

## main.rs - Application Entry Point

**File:** `src-tauri/src/main.rs`

### Purpose
Initializes the Tauri application, loads configuration, and registers plugins.

### Code Flow

```rust
fn main() {
    // 1. Get required config path (exits if missing)
    let config_path = require_config_path();
    
    // 2. Load and validate configuration
    let desktop_config = LoonDesktopConfig::load(&config_path);
    
    // 3. Log config location for debugging
    eprintln!("loon-desktop: config={} api={}", ...);
    
    // 4. Build Tauri app with:
    //    - ThemeModule for styling
    //    - Managed desktop_config state
    //    - Custom loon plugin
    //    - Opener plugin for URLs
    TauriApp::new("loon-admin")
        .with_config_path(&config_path)
        .module(ThemeModule::default())
        .with_builder(|builder| {
            builder
                .manage(desktop_config)
                .plugin(loon_plugin())
                .plugin(tauri_plugin_opener::init())
        })
        .run(tauri::generate_context!());
}
```

### Key Functions

#### `require_config_path()` → `PathBuf`
- **Source:** `config_host.rs`
- **Behavior:** Returns `~/.config/loon/config.toml` or exits process
- **Exit conditions:** Config file not found

#### `LoonDesktopConfig::load()` → `NestResult<LoonDesktopConfig>`
- **Source:** `config.rs`
- **Validates:** 
  - `server_url` is non-empty
  - `server_url` starts with `http://` or `https://`
- **Returns:** Config struct with parsed values

---

## commands.rs - Tauri IPC Commands

**File:** `src-tauri/src/commands.rs`

### Purpose
Defines the custom `loon` Tauri plugin with commands callable from the React frontend.

### Exported Commands

#### `get_config`

**Signature:**
```rust
#[tauri::command]
fn get_config(config: State<'_, LoonDesktopConfig>) -> DesktopConfigResponse
```

**Purpose:** Returns the loaded desktop configuration to the frontend.

**Response Type:** `DesktopConfigResponse`
```rust
pub struct DesktopConfigResponse {
    pub server_url: String,       // Backend API base URL
    pub config_path: String,      // Absolute path to config file
    pub player_path: Option<String>, // Optional external player binary
}
```

**Usage from Frontend:**
```typescript
const config = await invoke('plugin:loon|get_config')
```

---

#### `play_stream`

**Signature:**
```rust
#[tauri::command]
fn play_stream<R: Runtime>(
    app: AppHandle<R>,
    config: State<'_, LoonDesktopConfig>,
    slug: String,
    title: Option<String>,
) -> Result<PlayStreamResponse, String>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `app` | `AppHandle<R>` | Tauri application handle |
| `config` | `State` | Managed config state |
| `slug` | `String` | Movie slug identifier |
| `title` | `Option<String>` | Optional display title |

**Response Type:** `PlayStreamResponse`
```rust
pub struct PlayStreamResponse {
    pub stream_url: String,      // Full stream URL
    pub player_path: String,     // Always "tauri-window" for built-in player
}
```

**Behavior:**
1. Constructs stream URL: `{server_url}/stream/{encoded_slug}`
2. Determines display title (falls back to slug if not provided)
3. Calls `show_player_window()` to display player
4. Returns stream info to caller

**Usage from Frontend:**
```typescript
await invoke('plugin:loon|play_stream', { slug, title })
```

---

### Internal Functions

#### `show_player_window`

**Signature:**
```rust
fn show_player_window<R: Runtime>(
    app: &AppHandle<R>,
    slug: &str,
    title: &str,
    stream_url: &str,
) -> Result<(), String>
```

**Purpose:** Displays and configures the player window for playback.

**Steps:**
1. Gets player window by label `"player"`
2. Creates `PlayerLoadEvent` payload
3. Emits `player:load` event to player window
4. Sets window title
5. Shows window (was hidden at startup)
6. Focuses window

**Error Conditions:**
- Player window not registered in `tauri.conf.json`
- Event emission fails
- Window operations fail (show, focus, title)

**Event Payload:** `PlayerLoadEvent`
```rust
struct PlayerLoadEvent {
    slug: String,
    title: String,
    stream_url: String,
}
```

---

### Plugin Definition

#### `loon_plugin`

**Signature:**
```rust
pub fn loon_plugin<R: Runtime>() -> TauriPlugin<R>
```

**Purpose:** Creates the custom Tauri plugin that exposes commands to the frontend.

**Registered Commands:**
- `get_config`
- `play_stream`

**Build Registration:** `build.rs`
```rust
tauri_build::try_build(
    tauri_build::Attributes::new().plugin(
        "loon",
        tauri_build::InlinedPlugin::new()
            .commands(LOON_COMMANDS)
            .default_permission(tauri_build::DefaultPermissionRule::AllowAllCommands),
    ),
)
```

---

## config.rs - Configuration Management

**File:** `src-tauri/src/config.rs`

### Types

#### `LoonAdminSection`

**Purpose:** Represents the `[loon-admin]` TOML section.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct LoonAdminSection {
    pub server_url: String,
    #[serde(default)]
    pub player_path: Option<String>,
}
```

**Fields:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `server_url` | `String` | Yes | Backend API base URL (no trailing slash) |
| `player_path` | `Option<String>` | No | Path to external player (ffplay/mpv) |

---

#### `LoonDesktopConfig`

**Purpose:** Fully loaded and validated desktop configuration.

```rust
#[derive(Debug, Clone)]
pub struct LoonDesktopConfig {
    pub config_path: PathBuf,
    pub server_url: String,
    pub player_path: Option<String>,
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `config_path` | `PathBuf` | Absolute path to config file |
| `server_url` | `String` | Normalized backend URL (trimmed, no trailing slash) |
| `player_path` | `Option<String>` | External player path if configured |

---

### Methods

#### `LoonDesktopConfig::load`

**Signature:**
```rust
pub fn load(path: &Path) -> NestResult<Self>
```

**Validation Steps:**
1. Loads TOML using `ConfigLoader::file_or_search()`
2. Extracts `[loon-admin]` section
3. Trims whitespace and trailing slashes from `server_url`
4. Validates `server_url` is non-empty
5. Validates `server_url` starts with `http://` or `https://`

**Error Conditions:**
- Config file not found
- Invalid TOML syntax
- Missing `[loon-admin]` section
- Empty `server_url`
- Invalid URL scheme

**Test Coverage:**
```rust
#[test]
fn load_parses_server_url() {
    // Verifies URL parsing from temp config file
}
```

---

## config_host.rs - Config Path Resolution

**File:** `src-tauri/src/config_host.rs`

### Functions

#### `loon_config_dir` → `PathBuf`

**Purpose:** Returns the config directory path.

**Logic:**
```rust
pub fn loon_config_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".config").join("loon"))
        .unwrap_or_else(|| PathBuf::from(".config").join("loon"))
}
```

**Returns:**
- `$HOME/.config/loon` if HOME env var exists
- `./.config/loon` as fallback

---

#### `loon_config_path` → `PathBuf`

**Purpose:** Returns the full config file path.

**Returns:** `{config_dir}/config.toml`

---

#### `require_config_path` → `PathBuf`

**Purpose:** Gets config path or exits the process.

**Behavior:**
1. Calls `loon_config_path()`
2. Checks if file exists with `is_file()`
3. If missing:
   - Prints error to stderr
   - Prints help message
   - Exits with code 1
4. If found: returns path

**Exit Message:**
```
loon-desktop: configuration file not found: {path}
loon-desktop: create ~/.config/loon/config.toml with [loon-admin].server_url
```

---

## crates/loon-player - External Player Launcher

**File:** `crates/loon-player/src/lib.rs`

### Purpose
Provides functionality to launch external video players (ffplay, mpv) for HTTP streams.

### Public Functions

#### `resolve_player_path`

**Signature:**
```rust
pub fn resolve_player_path(configured: Option<&str>) -> NestResult<PathBuf>
```

**Purpose:** Finds the external player binary path.

**Resolution Order:**
1. If `configured` path provided:
   - If absolute: verify exists and executable
   - If relative: search in PATH
2. If not configured: search PATH for `["ffplay", "mpv", "mplayer"]`

**Error Conditions:**
- Configured path doesn't exist or isn't executable
- Configured relative name not found in PATH
- No player found in PATH and none configured

---

#### `launch_external_player`

**Signature:**
```rust
pub fn launch_external_player(
    player: &Path,
    url: &str,
    title: Option<&str>,
) -> NestResult<()>
```

**Purpose:** Spawns external player process detached.

**Player-Specific Arguments:**

| Player | Arguments |
|--------|-----------|
| **ffplay** | `-autoexit -window_title "{title}" {url}` |
| **mpv** | `--title={title} {url}` |
| **other** | `{title} {url}` |

**Process Configuration:**
- stdin: null
- stdout: null  
- stderr: null
- Detached: yes (parent doesn't wait)

---

#### `play_movie_stream`

**Signature:**
```rust
pub fn play_movie_stream(
    server_url: &str,
    slug: &str,
    title: Option<&str>,
    player_path: Option<&str>,
) -> NestResult<()>
```

**Purpose:** Complete flow - resolves player and launches with stream URL.

**URL Construction:**
```rust
let url = format!(
    "{}/stream/{}",
    server_url.trim().trim_end_matches('/'),
    urlencoding::encode(slug)
);
```

---

### Internal Functions

#### `is_executable`

**Signature:**
```rust
fn is_executable(path: &Path) -> bool
```

**Platform-Specific:**
- **Unix:** Checks file mode for execute permission (`mode & 0o111 != 0`)
- **Non-Unix:** Returns `true` for any existing file

---

#### `find_in_path`

**Signature:**
```rust
fn find_in_path(name: &str) -> Option<PathBuf>
```

**Purpose:** Searches PATH environment variable for executable.

**Algorithm:**
1. Split PATH by platform separator
2. For each directory, join with name
3. Check if candidate is executable
4. Return first match

---

## Build Configuration

### src-tauri/build.rs

**Purpose:** Configures Tauri build with plugin permissions.

```rust
const LOON_COMMANDS: &[&str] = &["get_config", "play_stream"];

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new().plugin(
            "loon",
            tauri_build::InlinedPlugin::new()
                .commands(LOON_COMMANDS)
                .default_permission(tauri_build::DefaultPermissionRule::AllowAllCommands),
        ),
    )
    .expect("failed to run tauri build");
}
```

---

### src-tauri/Cargo.toml

```toml
[package]
name = "loon-desktop"
version = "0.1.0"
description = "Loon Admin Desktop — thin Tauri shell for the Loon backend API"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
nest-config = { workspace = true }
nest-error = { workspace = true }
nest-tauri = { workspace = true, features = ["runtime"] }
nest-theme = { workspace = true }
serde = { version = "1", features = ["derive"] }
tauri = { workspace = true, features = ["custom-protocol"] }
tauri-plugin-opener = "2"
urlencoding = "2"

[dev-dependencies]
tempfile = "3"
```

---

### src-tauri/tauri.conf.json

**Key Configuration:**

```json
{
  "productName": "Loon Admin",
  "identifier": "com.loon.admin",
  "build": {
    "frontendDist": "../ui/dist",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "npm run dev --prefix ../ui",
    "beforeBuildCommand": "npm run build --prefix ../ui"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "Loon Admin",
        "width": 1280,
        "height": 800,
        "decorations": false
      },
      {
        "label": "player",
        "title": "Loon Player",
        "url": "player.html",
        "width": 1280,
        "height": 720,
        "visible": false,
        "decorations": false
      }
    ]
  }
}
```

**Window Labels:**
- `"main"` - Admin UI window
- `"player"` - Video playback window (hidden at startup)
