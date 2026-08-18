#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX REPAIR - REPAIR DIRECTORIES, PERMISSIONS & CONFIGURATION
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/logging.sh
. "${SCRIPT_DIR}/lib/logging.sh"

log_banner
log_step "Ejecutando reparación no destructiva de EmuBox..."

bash "${SCRIPT_DIR}/modules/directory-setup.sh"
bash "${SCRIPT_DIR}/modules/input-setup.sh"
bash "${SCRIPT_DIR}/modules/emulator-setup.sh"
bash "${SCRIPT_DIR}/modules/gamescope-setup.sh"
bash "${SCRIPT_DIR}/modules/autostart-setup.sh"

log_ok "Reparación completada. Todos los permisos y directorios han sido restaurados."
