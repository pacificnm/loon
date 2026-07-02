# Loon repository v1 Plan

## Status: Active

**Repository:** [github.com/pacificnm/loon](https://github.com/pacificnm/loon)  
**Local checkout:** `nest/apps/loon/` (separate git repo; ignored by nest)

Related: [v1.md](v1.md), [setup-v1.md](setup-v1.md), [implementation-v1.md](implementation-v1.md).

## Current state (checkout)

| Present | Missing (next) |
|---------|----------------|
| `README.md`, `docs/`, `config.example.toml` | Root `Cargo.toml`, `.cargo/config.toml`, `.gitignore` |
| Remote `origin` → `pacificnm/loon` | `server/` crate, CI, `.github/workflows/` |

Planning docs and config template may be uncommitted locally — commit to `pacificnm/loon` when ready.

## Clone (already done here)

```bash
cd nest/apps
git clone https://github.com/pacificnm/loon.git loon
```

## Directory layout

```text
loon/
├── README.md
├── LICENSE
├── .gitignore
├── Cargo.toml                 # workspace
├── .cargo/
│   └── config.toml            # path-patch nest (local dev only — see below)
├── config.example.toml
├── config.toml                # gitignored — operator copy
├── docs/
│   ├── v1.md
│   ├── api-v0.2.md
│   ├── data-v1.md
│   ├── repo-v1.md             # this file
│   ├── setup-v1.md
│   ├── webos-v1.md
│   ├── webos-test-checklist.md
│   └── implementation-v1.md
├── server/                    # loon-server binary
│   ├── Cargo.toml
│   ├── migrations/
│   │   └── 001_initial.sql
│   ├── tests/
│   │   ├── api.rs
│   │   └── fixtures/media/...
│   └── src/
├── webos/                     # deferred — Vite + React
└── .github/
    └── workflows/
        └── ci.yml
```

## Workspace `Cargo.toml`

```toml
[workspace]
resolver = "2"
members = ["server"]

[workspace.package]
edition = "2021"
rust-version = "1.75"
license = "MIT OR Apache-2.0"
repository = "https://github.com/pacificnm/loon"

[workspace.dependencies]
# Pin to nest git rev for reproducible CI (update intentionally)
nest-http-serve = { git = "https://github.com/pacificnm/nest", branch = "main" }
nest-config = { git = "https://github.com/pacificnm/nest", branch = "main" }
nest-error = { git = "https://github.com/pacificnm/nest", branch = "main" }
nest-logging = { git = "https://github.com/pacificnm/nest", branch = "main" }
nest-media = { git = "https://github.com/pacificnm/nest", branch = "main", features = ["async", "serde"] }
nest-file = { git = "https://github.com/pacificnm/nest", branch = "main" }
nest-media-library = { git = "https://github.com/pacificnm/nest", branch = "main" }
nest-tmdb = { git = "https://github.com/pacificnm/nest", branch = "main" }
nest-transcode = { git = "https://github.com/pacificnm/nest", branch = "main" }
nest-http-client = { git = "https://github.com/pacificnm/nest", branch = "main" }
nest-task-runtime = { git = "https://github.com/pacificnm/nest", branch = "main" }
nest-data = { git = "https://github.com/pacificnm/nest", branch = "main" }
nest-data-sqlite = { git = "https://github.com/pacificnm/nest", branch = "main" }

serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
clap = { version = "4", features = ["derive"] }
```

## `server/Cargo.toml`

```toml
[package]
name = "loon-server"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "loon-server"
path = "src/main.rs"

[dependencies]
nest-http-serve = { workspace = true }
nest-config = { workspace = true }
nest-error = { workspace = true }
nest-logging = { workspace = true }
nest-media = { workspace = true }
nest-file = { workspace = true }
# Phase 2+
nest-media-library = { workspace = true, optional = true }
nest-tmdb = { workspace = true, optional = true }
nest-transcode = { workspace = true, optional = true }
nest-http-client = { workspace = true, optional = true }
nest-task-runtime = { workspace = true, optional = true }
# Phase 3+
nest-data = { workspace = true, optional = true }
nest-data-sqlite = { workspace = true, optional = true }

serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
clap = { workspace = true }
axum = "0.7"

[features]
default = []
full = ["nest-media-library", "nest-tmdb", "nest-transcode", "nest-http-client", "nest-task-runtime", "nest-data", "nest-data-sqlite"]

[dev-dependencies]
reqwest = { version = "0.12", features = ["json"] }
tempfile = "3"
```

Trim features per phase — Phase 1 enables only http-serve + config + error.

## Local dev: path-patch Nest

`.cargo/config.toml` in the **loon repo** (committed — paths assume `apps/loon` beside nest):

```toml
[patch."https://github.com/pacificnm/nest"]
nest-http-serve = { path = "../../core/crates/nest-http-serve" }
nest-config = { path = "../../core/crates/nest-config" }
nest-error = { path = "../../core/crates/nest-error" }
nest-logging = { path = "../../core/crates/nest-logging" }
nest-media = { path = "../../core/crates/nest-media" }
nest-file = { path = "../../core/crates/nest-file" }
nest-media-library = { path = "../../modules/crates/nest-media-library" }
nest-tmdb = { path = "../../modules/crates/nest-tmdb" }
nest-transcode = { path = "../../modules/crates/nest-transcode" }
nest-http-client = { path = "../../core/crates/nest-http-client" }
nest-task-runtime = { path = "../../core/crates/nest-task-runtime" }
nest-data = { path = "../../core/crates/nest-data" }
nest-data-sqlite = { path = "../../modules/crates/nest-data-sqlite" }
```

CI uses **git deps only** (no patch). Developers beside nest get instant nest crate changes.

## CLI (loon-server)

```rust
#[derive(Parser)]
struct Cli {
    /// Path to config TOML
    #[arg(long, default_value = "/etc/loon/config.toml")]
    config: PathBuf,

    /// Force full library scan on startup (ignore cached DB catalog)
    #[arg(long)]
    force_scan: bool,

    /// Override bind address
    #[arg(long)]
    bind: Option<String>,
}
```

## `.gitignore`

```gitignore
/target/
/server/target/
config.toml
.env
*.db
*.db-journal
/webos/node_modules/
/webos/dist/
.DS_Store
```

## CI (`.github/workflows/ci.yml`)

```yaml
name: ci

on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test -p loon-server --features full
      - run: cargo clippy -p loon-server --features full -- -D warnings

  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check
```

Phase 1 CI may use `--no-default-features` until `full` compiles.

## Release binary

```bash
cargo build --release -p loon-server --features full
# artifact: target/release/loon-server
```

Install per [setup-v1.md](setup-v1.md) — copy to `/usr/local/bin/loon-server`.

## Branching

| Branch | Purpose |
|--------|---------|
| `main` | Stable; CI green |
| `feature/*` | Server or webOS work |

Tag releases: `v0.1.0`, `v0.2.0`.

## Related

- [setup-v1.md](setup-v1.md) — install on home server
- [implementation-v1.md](implementation-v1.md) — build order
