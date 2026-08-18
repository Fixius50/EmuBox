#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX MODULE - GAMESCOPE COMPOSITOR CONFIGURATION
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=lib/logging.sh
. "${SCRIPT_DIR}/lib/logging.sh"
# shellcheck source=lib/paths.sh
. "${SCRIPT_DIR}/lib/paths.sh"

log_step "Preparando perfil de compositor Gamescope para 1080p 60Hz..."

ENV_FILE="${EMUBOX_CONFIG_DIR}/emubox.env"
if [[ ! -f "${ENV_FILE}" ]]; then
  cat > "${ENV_FILE}" << 'EOF'
# Gamescope Compositor Settings for EmuBox Dedicated Console
GAMESCOPE_WIDTH=1920
GAMESCOPE_HEIGHT=1080
GAMESCOPE_REFRESH=60
GAMESCOPE_SCALING=integer
GAMESCOPE_HDR=0
PIPEWIRE_LATENCY=128/48000
EOF
  log_ok "Archivo de entorno Gamescope creado en ${ENV_FILE}"
else
  log_ok "Archivo de entorno Gamescope existente preservado."
fi
