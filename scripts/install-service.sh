#!/usr/bin/env bash
# Install Loon as a systemd service (production home-server layout).
#
# Usage:
#   ./scripts/install-service.sh [--media-root PATH] [--no-build] [--no-start]
#
# Requires: sudo, cargo (unless --no-build and binary already built)
set -euo pipefail

APP_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$APP_ROOT/target}"

BIN_DEST="/usr/local/bin/loon-server"
CONFIG_DIR="/etc/loon"
CONFIG_FILE="$CONFIG_DIR/config.toml"
ENV_FILE="$CONFIG_DIR/env"
DATA_DIR="/var/lib/loon"
LOG_DIR="/var/log/loon"
SERVICE_NAME="loon"
SERVICE_USER="loon"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"

MEDIA_ROOT="/mnt/media"
DO_BUILD=1
DO_START=1

usage() {
  cat <<EOF
Install Loon as a systemd service.

Options:
  --media-root PATH   Media library root (default: $MEDIA_ROOT)
  --no-build          Skip cargo build --release (binary must exist)
  --no-start          Install only; do not enable or start the service
  -h, --help          Show this help

After install:
  sudo systemctl status $SERVICE_NAME
  sudo systemctl restart $SERVICE_NAME
  journalctl -u $SERVICE_NAME -f

Edit config:  sudo nano $CONFIG_FILE
Edit secrets: sudo nano $ENV_FILE
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --media-root)
      MEDIA_ROOT="${2:?--media-root requires a path}"
      shift 2
      ;;
    --no-build)
      DO_BUILD=0
      shift
      ;;
    --no-start)
      DO_START=0
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ "$MEDIA_ROOT" != /* ]]; then
  echo "error: --media-root must be an absolute path (got: $MEDIA_ROOT)" >&2
  exit 1
fi

if ! command -v sudo >/dev/null 2>&1; then
  echo "error: sudo is required" >&2
  exit 1
fi

BIN_SRC="$APP_ROOT/target/release/loon-server"

if [[ "$DO_BUILD" -eq 1 ]]; then
  echo "==> Building release binary..."
  (cd "$APP_ROOT" && cargo build -p loon-server --release)
fi

if [[ ! -x "$BIN_SRC" ]]; then
  echo "error: release binary not found at $BIN_SRC" >&2
  echo "Run without --no-build, or: ./build release" >&2
  exit 1
fi

echo "==> Creating system user and directories..."
if ! id "$SERVICE_USER" &>/dev/null; then
  sudo useradd --system --home "$DATA_DIR" --shell /usr/sbin/nologin "$SERVICE_USER"
fi
sudo mkdir -p "$CONFIG_DIR" "$DATA_DIR" "$LOG_DIR"
sudo chown "$SERVICE_USER:$SERVICE_USER" "$DATA_DIR" "$LOG_DIR"

echo "==> Installing binary to $BIN_DEST..."
sudo install -m 755 "$BIN_SRC" "$BIN_DEST"

if [[ ! -f "$CONFIG_FILE" ]]; then
  echo "==> Installing config to $CONFIG_FILE..."
  sed "s|@MEDIA_ROOT@|$MEDIA_ROOT|g" "$APP_ROOT/deploy/config.example.toml" | sudo tee "$CONFIG_FILE" >/dev/null
  sudo chmod 644 "$CONFIG_FILE"
else
  echo "==> Keeping existing config: $CONFIG_FILE"
fi

if [[ ! -f "$ENV_FILE" ]]; then
  echo "==> Installing env template to $ENV_FILE (add TMDB_API_KEY)..."
  sudo cp "$APP_ROOT/deploy/env.example" "$ENV_FILE"
  sudo chmod 600 "$ENV_FILE"
  sudo chown root:"$SERVICE_USER" "$ENV_FILE"
else
  echo "==> Keeping existing env: $ENV_FILE"
fi

echo "==> Installing systemd unit..."
sed "s|@MEDIA_ROOT@|$MEDIA_ROOT|g" "$APP_ROOT/deploy/loon.service" | sudo tee "$SERVICE_FILE" >/dev/null
sudo chmod 644 "$SERVICE_FILE"

echo "==> Reloading systemd..."
sudo systemctl daemon-reload

if [[ "$DO_START" -eq 1 ]]; then
  echo "==> Enabling and starting $SERVICE_NAME..."
  sudo systemctl enable "$SERVICE_NAME"
  sudo systemctl restart "$SERVICE_NAME"
  sudo systemctl --no-pager status "$SERVICE_NAME" || true
else
  echo "==> Skipping enable/start (--no-start). Run:"
  echo "    sudo systemctl enable --now $SERVICE_NAME"
fi

cat <<EOF

Done.

  Health:  curl http://127.0.0.1:3000/api/health
  Logs:    journalctl -u $SERVICE_NAME -f
  Config:  $CONFIG_FILE
  Secrets: $ENV_FILE

Set TMDB_API_KEY in $ENV_FILE if not already configured, then:
  sudo systemctl restart $SERVICE_NAME

EOF
