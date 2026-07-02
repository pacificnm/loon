#!/usr/bin/env bash
# Wrapper for ares-launch -sp: starts the webOS TV Simulator with Linux flags
# required on Ubuntu 24+ (X11 + no-sandbox). ares-launch cannot pass these itself.
#
# Set WEBOS_SIM_APPIMAGE to your .AppImage path, e.g.:
#   export WEBOS_SIM_APPIMAGE=~/webOS_TV_26_Simulator_1.5.0.AppImage

set -euo pipefail

appimage="${WEBOS_SIM_APPIMAGE:-}"
if [[ -z "$appimage" || ! -f "$appimage" ]]; then
  echo "webos-simulator.sh: set WEBOS_SIM_APPIMAGE to your webOS TV Simulator .AppImage" >&2
  exit 1
fi

extra_flags=()
if [[ -n "${WEBOS_SIM_EXTRA_FLAGS:-}" ]]; then
  # shellcheck disable=SC2206
  extra_flags=(${WEBOS_SIM_EXTRA_FLAGS})
fi

exec "$appimage" \
  --ozone-platform=x11 \
  --no-sandbox \
  "${extra_flags[@]}" \
  "$@"
