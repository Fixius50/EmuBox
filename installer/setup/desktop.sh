#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX SETUP - DESKTOP ENTRY & SYSTEMD CONSOLE SESSION INSTALLER
# ==============================================================================

set -euo pipefail

XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
APPS_DIR="${XDG_DATA_HOME}/applications"
SYSTEMD_USER_DIR="${XDG_CONFIG_HOME}/systemd/user"

mkdir -p "${APPS_DIR}" "${SYSTEMD_USER_DIR}"

echo "  -> Creando acceso de escritorio (emubox.desktop)..."

cat > "${APPS_DIR}/emubox.desktop" << 'EOF'
[Desktop Entry]
Name=EmuBox Console
Comment=Dedicated Console Frontend for Retro & Modern Emulation
Exec=emubox
Icon=emubox
Terminal=false
Type=Application
Categories=Game;Emulator;Console;
StartupNotify=true
EOF

echo "  -> Creando unidad de sesión de consola systemd (emubox-session.service)..."

cat > "${SYSTEMD_USER_DIR}/emubox-session.service" << 'EOF'
[Unit]
Description=EmuBox Dedicated Console Wayland/Gamescope Session
After=graphical-session-pre.target
PartOf=graphical-session.target

[Service]
Type=simple
Environment=GAMESCOPE_DEFAULT_ARGS=-W 1920 -H 1080 -f -r 60 --prefer-vk-device
ExecStart=/usr/bin/bash /opt/emubox/scripts/run.sh
Restart=on-failure
RestartSec=2s

[Install]
WantedBy=graphical-session.target
EOF

systemctl --user daemon-reload 2>/dev/null || true

echo "  ✓ Acceso de escritorio y servicio de consola instalados."
