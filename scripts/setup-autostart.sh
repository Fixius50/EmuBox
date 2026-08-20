#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - AUTO START SETUP FOR ARCH LINUX (SYSTEMD SERVICE)
#  Configures EmuBox to launch cleanly when graphical target is reached.
#  NO desktop environments installed (NO GNOME, KDE, XFCE, Display Managers).
# ==============================================================================

set -euo pipefail

EMUBOX_USER="${SUDO_USER:-${USER}}"
EMUBOX_BIN="/usr/local/bin/emubox"
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
echo "       EmuBox - Auto Start Setup"
echo "=========================================="
echo ""
echo "Usuario: $EMUBOX_USER"
echo "UID:     $EMUBOX_UID"
echo "Home:    $EMUBOX_HOME"
echo ""

# ------------------------------------------------------------
# 1. Comprobar y enlazar ejecutable
# ------------------------------------------------------------
if [[ ! -x "$EMUBOX_BIN" ]]; then
  if [[ -x "/usr/bin/emubox" ]]; then
    EMUBOX_BIN="/usr/bin/emubox"
  else
    echo "[ERROR] No se encuentra el binario de emubox en /usr/local/bin/emubox ni /usr/bin/emubox."
    echo "Ejecuta primero la instalacion con: sudo ./scripts/setup-arch.sh"
    exit 1
  fi
fi

# Asegurar symlink en /usr/bin/emubox
if [[ "$EMUBOX_BIN" != "/usr/bin/emubox" ]]; then
  ln -sf "$EMUBOX_BIN" /usr/bin/emubox
fi

# ------------------------------------------------------------
# 2. Preparar directorios del sistema y logs
# ------------------------------------------------------------
echo "[1/4] Preparando directorios del sistema y logs..."

mkdir -p /etc/emubox
mkdir -p /var/lib/emubox/{games,roms,saves,states,bios,covers,logs,screenshots}
mkdir -p /var/log/emubox

chown -R "$EMUBOX_USER:$EMUBOX_USER" /var/lib/emubox
chown -R "$EMUBOX_USER:$EMUBOX_USER" /var/log/emubox
chmod -R 755 /var/log/emubox

# ------------------------------------------------------------
# 3. Crear servicio systemd
# ------------------------------------------------------------
echo "[2/4] Creando servicio systemd en /etc/systemd/system/$SERVICE_NAME..."

cat > "/etc/systemd/system/$SERVICE_NAME" <<EOF
[Unit]
Description=EmuBox Console Interface
After=graphical.target systemd-user-sessions.service
Wants=graphical.target

[Service]
Type=simple

User=$EMUBOX_USER
Group=$EMUBOX_USER

WorkingDirectory=/opt/emubox

ExecStart=/usr/bin/emubox

Restart=on-failure
RestartSec=3

# Variables generales de entorno
Environment=HOME=$EMUBOX_HOME
Environment=EMUBOX_HOME=/var/lib/emubox
Environment=NODE_ENV=production
Environment=WEBKIT_DISABLE_DMABUF_RENDERER=1
Environment=GDK_BACKEND=wayland,x11

# Logs permanentes
StandardOutput=append:/var/log/emubox/emubox.log
StandardError=append:/var/log/emubox/emubox-error.log

# Seguridad básica
NoNewPrivileges=true

[Install]
WantedBy=graphical.target
EOF

# ------------------------------------------------------------
# 4. Recargar y activar servicio systemd
# ------------------------------------------------------------
echo "[3/4] Recargando daemon systemd..."
systemctl daemon-reload

echo "[4/4] Activando servicio para autoarranque..."
systemctl enable "$SERVICE_NAME"

echo ""
echo "=========================================="
echo "       Autoarranque configurado con exito"
echo "=========================================="
echo ""
echo "Servicio registrado:"
echo "  $SERVICE_NAME"
echo ""
echo "Estado actual:"
systemctl is-enabled "$SERVICE_NAME"
echo ""
echo "Comandos utiles:"
echo "  sudo systemctl start emubox"
echo "  sudo systemctl stop emubox"
echo "  sudo systemctl restart emubox"
echo "  sudo systemctl status emubox"
echo ""
echo "Ficheros de log:"
echo "  /var/log/emubox/emubox.log"
echo "  /var/log/emubox/emubox-error.log"
echo ""
echo "Para desactivar el autoarranque:"
echo "  sudo systemctl disable emubox"
echo "=========================================="
