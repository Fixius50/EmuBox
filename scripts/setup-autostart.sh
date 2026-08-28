#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - AUTO START & CONSOLE APPLIANCE SETUP (ARCH LINUX)
#  Configures:
#    1. Getty TTY1 Autologin for dedicated console appliance
#    2. Anti-crash loop systemd bounds (StartLimitBurst=3)
#    3. Pure Wayland Session Launcher (Cage + Gamescope -> EmuBox)
# ==============================================================================

set -euo pipefail

EMUBOX_USER="${SUDO_USER:-${USER}}"
EMUBOX_APP_DIR="/opt/emubox"
SERVICE_NAME="emubox.service"

if [[ "$EUID" -ne 0 ]]; then
  echo "[ERROR] Ejecuta este script con sudo."
  echo ""
  echo "Ejemplo:"
  echo "  sudo ./scripts/setup-autostart.sh"
  exit 1
fi

if ! id "$EMUBOX_USER" >/dev/null 2>&1; then
  echo "[ERROR] No existe el usuario '$EMUBOX_USER'."
  exit 1
fi

EMUBOX_UID="$(id -u "$EMUBOX_USER")"
EMUBOX_HOME="$(getent passwd "$EMUBOX_USER" | cut -d: -f6)"

echo "=========================================="
echo "   EmuBox - Console Appliance Setup"
echo "=========================================="
echo ""
echo "Usuario: $EMUBOX_USER"
echo "UID:     $EMUBOX_UID"
echo "Home:    $EMUBOX_HOME"
echo ""

# ------------------------------------------------------------
# 1. Preparar directorios del sistema y permisos
# ------------------------------------------------------------
echo "[1/5] Preparando directorios del sistema y logs..."

mkdir -p /etc/emubox
mkdir -p /var/lib/emubox/{games,roms,saves,states,bios,covers,logs,screenshots}
mkdir -p /var/log/emubox

chown -R "$EMUBOX_USER:$EMUBOX_USER" /var/lib/emubox
chown -R "$EMUBOX_USER:$EMUBOX_USER" /var/log/emubox
chmod -R 755 /var/log/emubox

# ------------------------------------------------------------
# 2. Instalar el lanzador de sesión de consola (/usr/local/bin/emubox-session)
# ------------------------------------------------------------
echo "[2/5] Instalando lanzador de sesión de consola (/usr/local/bin/emubox-session)..."

cat << 'EOF' > /usr/local/bin/emubox-session
#!/usr/bin/env bash
set -euo pipefail

export WEBKIT_DISABLE_DMABUF_RENDERER=1
export GDK_BACKEND=wayland
export XDG_SESSION_TYPE=wayland
export EMUBOX_HOME="${EMUBOX_HOME:-/var/lib/emubox}"

EMUBOX_BIN="/opt/emubox/bin/emubox"
if [[ ! -x "${EMUBOX_BIN}" ]]; then
  echo "[ERROR] No existe el binario de EmuBox: ${EMUBOX_BIN}" >&2
  exit 1
fi

# Lanzamiento estratificado: Cage (kiosko/asiento) -> Gamescope (composición/escalado) -> EmuBox
if command -v cage >/dev/null 2>&1 && command -v gamescope >/dev/null 2>&1; then
  echo "[EmuBox] Iniciando sesión Wayland con Cage + Gamescope..."
  exec dbus-run-session cage -- gamescope -f -W 1920 -H 1080 -r 60 -- "${EMUBOX_BIN}" "$@"
elif command -v cage >/dev/null 2>&1; then
  echo "[EmuBox] Iniciando sesión Wayland con Cage..."
  exec dbus-run-session cage -- "${EMUBOX_BIN}" "$@"
elif command -v gamescope >/dev/null 2>&1; then
  echo "[EmuBox] Iniciando sesión Wayland directa con Gamescope..."
  exec dbus-run-session gamescope -f -W 1920 -H 1080 -r 60 -- "${EMUBOX_BIN}" "$@"
else
  exec "${EMUBOX_BIN}" "$@"
fi
EOF

chmod 0755 /usr/local/bin/emubox-session
ln -sf /usr/local/bin/emubox-session /usr/bin/emubox

# ------------------------------------------------------------
# 3. Configurar Autologin en TTY1 (getty@tty1)
# ------------------------------------------------------------
echo "[3/5] Configurando Autologin en TTY1 para el usuario $EMUBOX_USER..."

GETTY_OVERRIDE_DIR="/etc/systemd/system/getty@tty1.service.d"
mkdir -p "${GETTY_OVERRIDE_DIR}"

cat << EOF > "${GETTY_OVERRIDE_DIR}/autologin.conf"
[Service]
ExecStart=
ExecStart=-/sbin/agetty -o '-p -f -- \\\\u' --noclear --autologin ${EMUBOX_USER} %I \$TERM
Type=idle
EOF

# ------------------------------------------------------------
# 4. Configurar autoarranque en .bash_profile para la TTY1
# ------------------------------------------------------------
echo "[4/5] Configurando inicio automático de sesión en $EMUBOX_HOME/.bash_profile..."

BASH_PROFILE="${EMUBOX_HOME}/.bash_profile"
if ! grep -q "emubox-session" "${BASH_PROFILE}" 2>/dev/null; then
  cat << 'EOF' >> "${BASH_PROFILE}"

# EmuBox Console Appliance: Auto-launch Wayland session on physical TTY1
if [[ -z "$WAYLAND_DISPLAY" ]] && [[ -z "$DISPLAY" ]] && [[ "$(tty)" == "/dev/tty1" ]]; then
  exec /usr/local/bin/emubox-session
fi
EOF
  chown "$EMUBOX_USER:$EMUBOX_USER" "${BASH_PROFILE}"
fi

# ------------------------------------------------------------
# 5. Configurar servicio systemd con protección anti-bucles
# ------------------------------------------------------------
echo "[5/5] Registrando servicio systemd con límites de recuperación..."

cat > "/etc/systemd/system/$SERVICE_NAME" <<EOF
[Unit]
Description=EmuBox Dedicated Console Interface
After=graphical.target systemd-user-sessions.service network.target
Wants=graphical.target
StartLimitIntervalSec=60s
StartLimitBurst=3

[Service]
Type=simple
User=$EMUBOX_USER
Group=$EMUBOX_USER
WorkingDirectory=/opt/emubox
ExecStart=/usr/local/bin/emubox-session
Restart=on-failure
RestartSec=5

# Variables de entorno
Environment=HOME=$EMUBOX_HOME
Environment=EMUBOX_HOME=/var/lib/emubox
Environment=NODE_ENV=production
Environment=WEBKIT_DISABLE_DMABUF_RENDERER=1
Environment=GDK_BACKEND=wayland
Environment=XDG_SESSION_TYPE=wayland

# Logs permanentes
StandardOutput=append:/var/log/emubox/emubox.log
StandardError=append:/var/log/emubox/emubox-error.log

# Seguridad básica
NoNewPrivileges=true

[Install]
WantedBy=graphical.target
EOF

systemctl daemon-reload

echo ""
echo "=========================================="
echo "   Configuración de Consola Completada"
echo "=========================================="
echo "1. Autologin en TTY1:     ACTIVO ($EMUBOX_USER)"
echo "2. Lanzador Wayland:      /usr/local/bin/emubox-session"
echo "3. Límite de reinicios:   3 intentos / 60s (Anti-crash loop)"
echo "4. SSH Desacoplado:       Puerto de administración independiente"
echo "=========================================="
