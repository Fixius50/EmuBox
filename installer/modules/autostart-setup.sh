#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX MODULE - AUTOSTART & SYSTEMD USER SERVICE
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=lib/logging.sh
. "${SCRIPT_DIR}/lib/logging.sh"
# shellcheck source=lib/systemd.sh
. "${SCRIPT_DIR}/lib/systemd.sh"
# shellcheck source=lib/paths.sh
. "${SCRIPT_DIR}/lib/paths.sh"

log_step "Preparando integración de escritorio y servicio de sesión systemd..."

APPS_DIR="${XDG_DATA_HOME}/applications"
mkdir -p "${APPS_DIR}"

cat > "${APPS_DIR}/emubox.desktop" << 'EOF'
[Desktop Entry]
Name=EmuBox Console
Comment=Dedicated Console Environment for Emulation
Exec=emubox
Icon=emubox
Terminal=false
Type=Application
Categories=Game;Emulator;Console;
StartupNotify=true
EOF

install_systemd_user_service

log_ok "Integración completada (Modo desarrollo: inicio manual vía 'emubox')."
