#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX INSTALLER - SYSTEMD USER SERVICE MANAGEMENT
# ==============================================================================

install_systemd_user_service() {
  local service_dir="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
  local service_file="${service_dir}/emubox.service"

  mkdir -p "${service_dir}"

  cat > "${service_file}" << 'EOF'
[Unit]
Description=EmuBox OS Dedicated Console Session
After=graphical-session-pre.target
PartOf=graphical-session.target

[Service]
Type=simple
Environment=WEBKIT_DISABLE_DMABUF_RENDERER=1
Environment=GDK_BACKEND=wayland,x11
Environment=GAMESCOPE_DEFAULT_ARGS=-W 1920 -H 1080 -f -r 60
ExecStart=/usr/bin/gamescope -W 1920 -H 1080 -f -- /usr/local/bin/emubox
Restart=on-failure
RestartSec=2s

[Install]
WantedBy=graphical-session.target
EOF

  systemctl --user daemon-reload 2>/dev/null || true

  log_ok "Servicio systemd creado en ${service_file} (desactivado por defecto para desarrollo)."
}
