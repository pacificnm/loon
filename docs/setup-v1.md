# Loon server setup v1 Plan

## Status: Planned

Native install on a **Linux home server** — systemd only, **no Docker**. See [repo-v1.md](repo-v1.md) for building the binary.

## Quick install (systemd)

From a clone of [pacificnm/loon](https://github.com/pacificnm/loon):

```bash
./scripts/install-service.sh --media-root /mnt/media
```

This builds `loon-server --release`, creates the `loon` system user, installs
`/etc/loon/config.toml` and `/etc/loon/env`, copies the binary to
`/usr/local/bin/loon-server`, installs `deploy/loon.service`, and starts the
service.

Options:

| Flag | Effect |
|------|--------|
| `--media-root PATH` | Set `media_root` in config and systemd `ReadOnlyPaths` (default `/mnt/media`) |
| `--no-build` | Skip `cargo build --release` (binary must already exist) |
| `--no-start` | Install files only; do not `enable` or `start` |

After install, set `TMDB_API_KEY` in `/etc/loon/env` and restart:

```bash
sudo nano /etc/loon/env
sudo systemctl restart loon
```

Manual steps below remain for reference or custom layouts.

## Prerequisites

| Requirement | Notes |
|-------------|-------|
| Linux x86_64 | Debian/Ubuntu or similar |
| Rust-built binary | `loon-server` from [pacificnm/loon](https://github.com/pacificnm/loon) releases or local `cargo build --release` |
| Media mount | e.g. `/mnt/media/Movies/` |
| `ffprobe` | On PATH or configured in config |
| TMDB API key | Optional; [themoviedb.org](https://www.themoviedb.org/settings/api) |

## Directory layout on server

```text
/usr/local/bin/loon-server          # binary
/etc/loon/
├── config.toml                     # main config (from config.example.toml)
└── env                             # secrets — TMDB_API_KEY, overrides
/var/lib/loon/
├── loon.db                         # SQLite (v0.2+)
└── cache/                          # poster cache (optional v0.2+)
/var/log/loon/
└── server.log                      # if file logging enabled
/mnt/media/                         # read-only mount recommended
└── Movies/
    └── ...
```

## System user

```bash
sudo useradd --system --home /var/lib/loon --shell /usr/sbin/nologin loon
sudo mkdir -p /etc/loon /var/lib/loon /var/log/loon
sudo chown loon:loon /var/lib/loon /var/log/loon
```

## Install binary

```bash
sudo cp target/release/loon-server /usr/local/bin/
sudo chmod 755 /usr/local/bin/loon-server
```

## Configuration

```bash
sudo cp config.example.toml /etc/loon/config.toml
sudo nano /etc/loon/config.toml
```

Minimal production config:

```toml
[loon]
bind = "0.0.0.0:3000"
data_dir = "/var/lib/loon"
media_root = "/mnt/media"

[media-library]
id = "main"
roots = ["Movies"]

[tmdb]
api_key_env = "TMDB_API_KEY"

[transcode]
ffprobe_path = "/usr/bin/ffprobe"

[http]
# webOS on LAN — set to TV subnet origin in production if needed
cors_origins = ["*"]

[logging]
level = "info"
file = "/var/log/loon/server.log"
```

### Secrets (`/etc/loon/env`)

```bash
TMDB_API_KEY=your-key-here
# LOON_MEDIA_ROOT=/mnt/media   # optional override
```

```bash
sudo chmod 600 /etc/loon/env
sudo chown root:loon /etc/loon/env
```

## systemd unit

Shipped in [`deploy/loon.service`](../deploy/loon.service). Installed by
[`scripts/install-service.sh`](../scripts/install-service.sh), or manually at
`/etc/systemd/system/loon.service`:

```ini
[Unit]
Description=Loon media server
Documentation=https://github.com/pacificnm/loon
After=network-online.target local-fs.target
Wants=network-online.target

[Service]
Type=simple
User=loon
Group=loon
EnvironmentFile=-/etc/loon/env
ExecStart=/usr/local/bin/loon-server --config /etc/loon/config.toml
Restart=on-failure
RestartSec=5
LimitNOFILE=65536

# Hardening (adjust if media on separate mount)
NoNewPrivileges=true
ProtectSystem=strict
ReadWritePaths=/var/lib/loon /var/log/loon
ReadOnlyPaths=/mnt/media

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable loon
sudo systemctl start loon
sudo systemctl status loon
```

## First-run checklist

1. [ ] Media mounted at `media_root` with movie folders visible
2. [ ] `ffprobe -version` works as `loon` user: `sudo -u loon ffprobe -version`
3. [ ] `TMDB_API_KEY` in env (optional but recommended)
4. [ ] Start service — first boot runs library scan (may take minutes)
5. [ ] Verify: `curl http://localhost:3000/api/health`
6. [ ] Verify: `curl http://localhost:3000/api/movies`
7. [ ] From TV subnet: `curl http://SERVER_IP:3000/api/health`
8. [ ] Point webOS app at `http://SERVER_IP:3000`

## Force rescan

```bash
sudo systemctl stop loon
sudo -u loon /usr/local/bin/loon-server --config /etc/loon/config.toml --force-scan
# Or v0.2: curl -X POST http://localhost:3000/api/library/scan
sudo systemctl start loon
```

## Upgrade

```bash
sudo systemctl stop loon
sudo cp loon-server /usr/local/bin/loon-server
sudo systemctl start loon
# Migrations run automatically on startup (v0.2+)
```

## TLS (optional)

Loon listens plain HTTP on LAN. For HTTPS:

- Terminate TLS on **Caddy** or **nginx** on the same host
- Reverse proxy `https://loon.home.local` → `http://127.0.0.1:3000`
- webOS app uses HTTPS base URL

No TLS inside `loon-server` in v0.2.

## Firewall

Allow TCP **3000** from LAN only (adjust for your network):

```bash
# ufw example
sudo ufw allow from 192.168.0.0/16 to any port 3000 proto tcp
```

## Troubleshooting

| Symptom | Check |
|---------|-------|
| Empty movie list | Logs; `LOON_MEDIA_ROOT`; folder naming; scan errors |
| No posters | `TMDB_API_KEY`; outbound HTTPS |
| Stream fails | File permissions for `loon` user; codec (see v1 direct-play notes) |
| Permission denied on media | `loon` user read access to `/mnt/media` |

```bash
journalctl -u loon -f
tail -f /var/log/loon/server.log
```

## Related

- [repo-v1.md](repo-v1.md) — build binary
- [v1.md](v1.md) — server behavior
- [setup is not Docker](../README.md) — non-goal
