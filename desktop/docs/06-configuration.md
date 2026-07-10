# Configuration Guide

## Configuration File Location

The Loon Desktop application requires a configuration file at:

```
~/.config/loon/config.toml
```

**Full path examples:**
- Linux: `/home/username/.config/loon/config.toml`
- macOS: `/Users/username/.config/loon/config.toml`

---

## Setup

### 1. Create Configuration Directory

```bash
mkdir -p ~/.config/loon
```

### 2. Copy Example Configuration

```bash
cp config.example.toml ~/.config/loon/config.toml
```

### 3. Edit Configuration

```bash
nano ~/.config/loon/config.toml
```

---

## Configuration Format

### Example File

**File:** `config.example.toml`

```toml
# Copy to ~/.config/loon/config.toml
#
# Loon has three apps:
#   server  — apps/loon/server     (backend API, e.g. :3000)
#   client  — apps/loon/client     (LG webOS TV)
#   desktop — apps/loon/desktop    (this admin UI — talks to server only)

[gui]
title = "Loon Admin"
width = 1280
height = 800

[loon-admin]
server_url = "http://192.168.88.10:3000"
# player_path = "/usr/bin/ffplay"   # optional — defaults to ffplay, then mpv

[logging]
level = "info"
directory = "./logs"
```

---

## Configuration Sections

### `[loon-admin]` (Required)

The primary configuration section for the desktop application.

#### `server_url`

**Type:** `string` (URL)  
**Required:** Yes  
**Example:** `http://192.168.88.10:3000`

Backend API base URL. Must start with `http://` or `https://`.

**Validation:**
- Must not be empty
- Must start with `http://` or `https://`
- Trailing slashes are automatically removed

**Examples:**
```toml
# Valid
server_url = "http://192.168.88.10:3000"
server_url = "https://loon.example.com"
server_url = "http://localhost:3000"

# Invalid - will fail to load
server_url = ""                    # Empty
server_url = "192.168.1.1:3000"    # Missing scheme
```

---

#### `player_path`

**Type:** `string` (file path)  
**Required:** No  
**Default:** Auto-detect (ffplay → mpv → mplayer)

Path to external video player binary for HTTP stream playback.

**Supported Players:**
- `ffplay` (from ffmpeg)
- `mpv`
- `mplayer`

**Resolution Order:**
1. If `player_path` configured:
   - Absolute path: used directly (must exist and be executable)
   - Relative name: searched in PATH
2. If not configured: auto-detect in order: ffplay → mpv → mplayer

**Examples:**
```toml
# Absolute path
player_path = "/usr/bin/ffplay"

# Relative name (searched in PATH)
player_path = "mpv"

# Let app auto-detect
# (omit player_path entirely)
```

**Player Arguments:**

| Player | Arguments |
|--------|-----------|
| ffplay | `-autoexit -window_title "{title}" {url}` |
| mpv | `--title={title} {url}` |
| mplayer | `{title} {url}` |

---

### `[gui]` (Optional)

Window appearance settings.

#### `title`

**Type:** `string`  
**Default:** `"Loon Admin"`

Window title displayed in title bar.

---

#### `width`

**Type:** `integer`  
**Default:** `1280`

Initial window width in pixels.

---

#### `height`

**Type:** `integer`  
**Default:** `800`

Initial window height in pixels.

**Example:**
```toml
[gui]
title = "Loon Admin"
width = 1920
height = 1080
```

---

### `[logging]` (Optional)

Logging configuration.

#### `level`

**Type:** `string`  
**Default:** `"info"`  
**Valid values:** `trace`, `debug`, `info`, `warn`, `error`

Minimum log level to output.

---

#### `directory`

**Type:** `string` (path)  
**Default:** `"./logs"`

Directory for log files.

**Example:**
```toml
[logging]
level = "debug"
directory = "/var/log/loon"
```

---

## Error Handling

### Missing Config File

If `~/.config/loon/config.toml` does not exist, the application exits immediately with:

```
loon-desktop: configuration file not found: /home/user/.config/loon/config.toml
loon-desktop: create ~/.config/loon/config.toml with [loon-admin].server_url
```

### Invalid Configuration

If configuration is invalid (empty URL, wrong scheme), the application exits with:

```
loon-desktop: failed to load /home/user/.config/loon/config.toml: {error}
```

**Common Errors:**
- `[loon-admin].server_url must not be empty`
- `[loon-admin].server_url must be an http(s) URL, got: {value}`
- `configured player_path is not executable: {path}`
- `configured player_path not found in PATH: {name}`

---

## Environment Variables

### `HOME`

**Purpose:** Determines config directory location

**Default:** User's home directory

**Fallback:** If `HOME` is not set, uses `./.config/loon`

---

## Runtime Behavior

### Config Loading Flow

```rust
// 1. Get config path
let config_path = require_config_path();  // Exits if missing

// 2. Load and parse TOML
let loaded = ConfigLoader::file_or_search("loon", Some(config_path)).load()?;

// 3. Extract [loon-admin] section
let section: LoonAdminSection = loaded.document.section("loon-admin")?;

// 4. Normalize server_url
let server_url = section.server_url
    .trim()
    .trim_end_matches('/')
    .to_string();

// 5. Validate
if server_url.is_empty() { /* error */ }
if !server_url.starts_with("http://") && !server_url.starts_with("https://") {
    /* error */
}
```

### Config Caching (Frontend)

```typescript
let cached: DesktopConfig | null = null

export async function loadDesktopConfig(): Promise<DesktopConfig> {
  if (cached) return cached  // Return cached value
  
  const response = await invoke('plugin:loon|get_config')
  cached = {
    serverUrl: response.serverUrl.trim().replace(/\/$/, ''),
    configPath: response.configPath,
    playerPath: response.playerPath?.trim() || undefined,
  }
  return cached
}
```

**Note:** Config is loaded once and cached for the lifetime of the application.

---

## Viewing Configuration

### Settings Panel

The Settings panel displays current configuration:

```
Settings
─────────────────────────────────────────
Config file
~/.config/loon/config.toml

Backend API
http://192.168.88.10:3000

Health
ok (150 movies)

Video player
Built-in player window
```

### Console Output

On startup, the application logs config details:

```
loon-desktop: config=/home/user/.config/loon/config.toml api=http://192.168.88.10:3000
```

---

## Troubleshooting

### "Configuration file not found"

**Solution:**
```bash
mkdir -p ~/.config/loon
cp /path/to/desktop/config.example.toml ~/.config/loon/config.toml
# Edit with your server URL
```

### "server_url must be an http(s) URL"

**Problem:** URL missing scheme

**Solution:**
```toml
# Wrong
server_url = "192.168.88.10:3000"

# Correct
server_url = "http://192.168.88.10:3000"
```

### "player_path not found"

**Problem:** Configured player binary doesn't exist

**Solution:**
```toml
# Option 1: Install ffplay
sudo apt install ffmpeg  # Debian/Ubuntu
brew install ffmpeg      # macOS

# Option 2: Use mpv
player_path = "mpv"

# Option 3: Remove player_path (auto-detect)
# (comment out or delete the line)
```

### Verify Backend Connection

```bash
curl http://192.168.88.10:3000/api/health
# Expected: {"status":"ok","movies_count":150}
```

---

## Related Documentation

- [Architecture Overview](./01-overview.md) - Application structure
- [Rust Backend](./02-rust-backend.md) - Config loading implementation
- [Settings Panel](./04-components.md#settingspanel) - UI display
