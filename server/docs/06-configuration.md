# Configuration Guide

## Configuration File Location

The Loon Server application loads configuration from:

```
~/.config/loon/config.toml
```

**Full path examples:**
- Linux: `/home/username/.config/loon/config.toml`
- macOS: `/Users/username/.config/loon/config.toml`

**CLI Override:**
```bash
./build dev -- --config /path/to/custom/config.toml
```

---

## Setup

### 1. Create Configuration Directory

```bash
mkdir -p ~/.config/loon
```

### 2. Create Configuration File

```bash
cat > ~/.config/loon/config.toml << EOF
[loon]
bind = "0.0.0.0:3000"
media_root = "/mnt/media"
data_dir = "./data"

[media-library]
id = "main"
roots = ["Movies"]
video_extensions = ["mp4", "mkv", "avi", "mov"]
EOF
```

---

## Configuration Sections

### `[loon]` (Required)

Core server settings.

#### `bind`

**Type:** `string`  
**Required:** Yes  
**Example:** `"0.0.0.0:3000"`

HTTP bind address (host:port).

**Examples:**
```toml
# All interfaces, port 3000
bind = "0.0.0.0:3000"

# Localhost only
bind = "127.0.0.1:3000"

# IPv6
bind = "[::]:3000"
```

**CLI Override:**
```bash
./build dev -- --bind "127.0.0.1:8080"
```

---

#### `media_root`

**Type:** `string` (path)  
**Required:** Yes  
**Example:** `"/mnt/media"`

Absolute path to the media library root. All library paths are resolved relative to this directory.

**Examples:**
```toml
# Network mount
media_root = "/mnt/media"

# Local storage
media_root = "/Volumes/Media/Movies"

# NAS share
media_root = "/Volumes/NAS/Media"
```

**Security:** Paths containing `..` are rejected to prevent traversal attacks.

---

#### `data_dir`

**Type:** `string` (path)  
**Required:** No  
**Default:** `"./data"` (relative to working directory)

Directory for SQLite database and cache.

**Examples:**
```toml
# Relative path
data_dir = "./data"

# Absolute path
data_dir = "/var/lib/loon"

# XDG location
data_dir = "~/.local/share/loon"
```

**Database Path:** `{data_dir}/loon.db`

---

### `[media-library]` (Required)

Library scanning settings.

#### `id`

**Type:** `string`  
**Required:** Yes  
**Example:** `"main"`

Unique library identifier.

---

#### `roots`

**Type:** `array<string>`  
**Required:** Yes  
**Example:** `["Movies", "4K Movies"]`

Subdirectories under `media_root` to scan.

**Examples:**
```toml
# Single directory
roots = ["Movies"]

# Multiple directories
roots = ["Movies", "4K Movies", "Classic"]

# Nested paths
roots = ["Movies/English", "Movies/Foreign"]
```

**Resolution:**
```
media_root = "/mnt/media"
roots = ["Movies", "4K Movies"]

# Scans:
# /mnt/media/Movies/
# /mnt/media/4K Movies/
```

---

#### `video_extensions`

**Type:** `array<string>`  
**Required:** No  
**Default:** Common video extensions

File extensions to scan (without leading dot).

**Default Extensions:**
- `mp4`, `m4v`, `mov`
- `mkv`
- `webm`
- `avi`
- `wmv`, `flv`, `mpg`, `mpeg`

**Examples:**
```toml
# Restrict to specific formats
video_extensions = ["mkv", "mp4"]

# Include disc images
video_extensions = ["mkv", "mp4", "iso", "m2ts"]
```

---

### `[tmdb]` (Optional)

TMDB metadata enrichment settings.

#### `api_key`

**Type:** `string`  
**Required:** Yes (for enrichment)  
**Environment:** `TMDB_API_KEY`

TMDB API key for metadata fetching.

**Configuration:**
```toml
[tmdb]
api_key = "your-api-key-here"
```

**Environment Variable (recommended):**
```toml
[tmdb]
api_key = "${TMDB_API_KEY}"
```

```bash
export TMDB_API_KEY="your-api-key-here"
```

**When Not Configured:**
- Server starts normally
- Metadata enrichment disabled
- Movies use filename-based titles only

---

### `[ai]` (Optional)

AI-assisted filename guessing settings.

#### `enabled`

**Type:** `boolean`  
**Required:** No  
**Default:** `true`

Enable AI filename guessing.

---

#### `provider`

**Type:** `string`  
**Required:** No  
**Default:** `"ollama"`

AI provider (only `ollama` supported in v0.1).

---

#### `base_url`

**Type:** `string`  
**Required:** No  
**Default:** `"http://localhost:11434"`

Ollama API base URL.

---

#### `model`

**Type:** `string`  
**Required:** No  
**Default:** `"llama3.1"`

Ollama model for inference.

---

#### `min_confidence`

**Type:** `float`  
**Required:** No  
**Default:** `0.5`

Minimum confidence threshold (0.0-1.0) before applying AI guess.

**Examples:**
```toml
[ai]
enabled = true
provider = "ollama"
base_url = "http://localhost:11434"
model = "llama3.1"
min_confidence = 0.7  # Higher threshold
```

**When Not Configured:**
- AI filename guessing disabled
- Falls back to filename parsing

---

### `[cache]` (Optional)

Artwork cache settings.

#### `enabled`

**Type:** `boolean`  
**Required:** No  
**Default:** `false`

Enable on-disk artwork cache.

---

#### `root`

**Type:** `string` (path)  
**Required:** No  
**Default:** `"cache"` (relative to `data_dir`)

Cache directory path.

**Examples:**
```toml
[cache]
enabled = true
root = "./cache"           # Relative to data_dir
root = "/var/cache/loon"   # Absolute path
```

---

#### `max_mb`

**Type:** `integer`  
**Required:** No  
**Default:** No limit (LRU eviction deferred)

Maximum cache size in megabytes.

**Examples:**
```toml
[cache]
enabled = true
max_mb = 1024  # 1 GB limit
```

**Full Example:**
```toml
[cache]
enabled = true
root = "./cache"
max_mb = 512
```

---

### `[logging]` (Optional)

Logging configuration.

#### `level`

**Type:** `string`  
**Required:** No  
**Default:** `"info"`  
**Valid values:** `trace`, `debug`, `info`, `warn`, `error`

Minimum log level.

---

#### `directory`

**Type:** `string` (path)  
**Required:** No  
**Default:** `"./logs"`

Directory for log files.

---

#### `file`

**Type:** `string` (path)  
**Required:** No

Alternative: specify exact log file path.

**Examples:**
```toml
[logging]
level = "debug"
directory = "./logs"

# Or specify exact file
# file = "./logs/loon-server.log"
```

---

## Complete Example

```toml
# ~/.config/loon/config.toml

[loon]
bind = "0.0.0.0:3000"
media_root = "/mnt/media"
data_dir = "/var/lib/loon"

[media-library]
id = "main"
roots = ["Movies", "4K Movies"]
video_extensions = ["mp4", "mkv", "avi"]

[tmdb]
api_key = "${TMDB_API_KEY}"

[ai]
enabled = true
provider = "ollama"
base_url = "http://localhost:11434"
model = "llama3.1"
min_confidence = 0.5

[cache]
enabled = true
root = "./cache"
max_mb = 1024

[logging]
level = "info"
directory = "./logs"
```

---

## Environment Variables

| Variable | Section | Description |
|----------|---------|-------------|
| `HOME` | All | Config directory base (`~/.config`) |
| `TMDB_API_KEY` | `[tmdb]` | TMDB API key |
| `OLLAMA_HOST` | `[ai]` | Ollama host (alternative to `base_url`) |

---

## CLI Options

**File:** `src/config/mod.rs`

### `--config`

**Type:** `PathBuf`  
**Default:** `config.toml` (working directory)

```bash
./build dev -- --config /path/to/config.toml
```

---

### `--bind`

**Type:** `String`  
**Default:** From config `[loon].bind`

```bash
./build dev -- --bind "127.0.0.1:8080"
```

---

### `--force-scan`

**Type:** `flag`  
**Default:** `false`

Force full library scan on startup (even if database has movies).

```bash
./build dev -- --force-scan
```

---

## Error Handling

### Missing Config File

```
loon-server: failed to load config.toml: missing or invalid [loon]
Copy config.example.toml to config.toml and set media_root to your movie folder.
```

**Solution:**
```bash
mkdir -p ~/.config/loon
cp config.example.toml ~/.config/loon/config.toml
# Edit with your settings
```

---

### Missing Required Section

```
missing or invalid [loon] in /path/to/config.toml
```

**Solution:** Ensure `[loon]` section exists with `bind` and `media_root`.

---

### Invalid Bind Address

```
failed to bind to 0.0.0.0:3000: Address already in use
```

**Solutions:**
1. Change port: `bind = "0.0.0.0:3001"`
2. Stop conflicting service
3. Use different interface: `bind = "127.0.0.1:3000"`

---

### TMDB Not Configured

```
TMDB_API_KEY not set; metadata enrichment disabled
```

**Behavior:** Server starts normally, but movies won't have metadata.

**Solution:** Set `TMDB_API_KEY` environment variable or add to config.

---

### AI Provider Unavailable

```
AI filename guessing failed: connection refused
```

**Behavior:** Falls back to filename parsing.

**Solutions:**
1. Start Ollama: `ollama serve`
2. Disable AI: `enabled = false`
3. Fix `base_url` configuration

---

## Runtime Behavior

### Config Loading Flow

```rust
// 1. CLI parsing
let cli = Cli::parse();

// 2. Logging init (before config load for early errors)
logging::init_from_cli(&cli)?;

// 3. Load config from file or search path
let loaded = ConfigLoader::file_or_search("loon", Some(cli.config)).load()?;
let service = ConfigService::new(loaded);

// 4. Extract sections
let loon: LoonSection = service.section("loon")?;
let media_library: MediaLibrarySection = service.section("media-library")?;

// 5. Apply CLI overrides
let bind = cli.bind.clone().unwrap_or(loon.bind);

// 6. Load optional sections
let tmdb = load_tmdb(&service);    // None if API key missing
let ai = load_ai(&service);        // None if disabled
let cache = load_cache(&service);  // None if disabled
```

---

### Config Caching

Configuration is loaded once at startup and stored in `AppState`:

```rust
pub struct AppState {
    pub config: Arc<ServerConfig>,
    // ...
}
```

**Note:** Config changes require server restart.

---

## Verification

### Check Config Loading

```bash
# Start server and watch for config log
./build dev

# Expected output:
# loon-server: config=/home/user/.config/loon/config.toml
# loon-server: bind=0.0.0.0:3000 media_root=/mnt/media
```

---

### Verify Backend Connection

```bash
curl http://localhost:3000/api/health
# Expected: {"status":"ok","service":"loon-server",...}
```

---

### Test Library Scan

```bash
curl -X POST http://localhost:3000/api/library/scan \
  -H "Content-Type: application/json" \
  -d '{"full": false}'
```

---

## Related Documentation

- [Architecture Overview](./01-overview.md) - Application structure
- [API Reference](./02-api-reference.md) - HTTP endpoints
- [Database](./04-database.md) - SQLite schema
