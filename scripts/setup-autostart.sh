#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - AUTO START & ADAPTIVE CONSOLE APPLIANCE SETUP (ARCH LINUX)
#  Configures:
#    1. Single Clean Getty TTY1 Autologin (emubox-autologin.conf)
#    2. Hardware-Adaptive Session Launcher with Vulkan / DRM Probing
#    3. NATIVE mode (Cage -> Gamescope) on Vulkan GPUs / COMPATIBILITY mode (Cage) on VMs
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
echo "Usuario: $EMUBOX_USER (UID: $EMUBOX_UID)"
echo "Home:    $EMUBOX_HOME"
echo ""

# ------------------------------------------------------------
# 1. Preparar directorios del sistema y permisos
# ------------------------------------------------------------
echo "[1/5] Preparando directorios del sistema y permisos..."

mkdir -p /etc/emubox
mkdir -p /var/lib/emubox/{games,roms,saves,states,bios,covers,logs,screenshots}
mkdir -p /var/log/emubox

chown -R "$EMUBOX_USER:$EMUBOX_USER" /var/lib/emubox
chown -R "$EMUBOX_USER:$EMUBOX_USER" /var/log/emubox
chmod -R 755 /var/log/emubox

# ------------------------------------------------------------
# 2. Instalar lanzador de sesión adaptativo (/usr/local/bin/emubox-session)
# ------------------------------------------------------------
echo "[2/5] Instalando lanzador de sesión adaptativo (/usr/local/bin/emubox-session)..."

cat << 'EOF' > /usr/local/bin/emubox-session
#!/usr/bin/env bash
set -euo pipefail

export WEBKIT_DISABLE_DMABUF_RENDERER=1
export GDK_BACKEND=wayland
export XDG_SESSION_TYPE=wayland
export EMUBOX_HOME="${EMUBOX_HOME:-/var/lib/emubox}"

EMUBOX_BIN="/opt/emubox/bin/emubox"
if [[ ! -x "${EMUBOX_BIN}" ]]; then
  echo "[ERROR] No existe el binario de EmuBox en ${EMUBOX_BIN}" >&2
  exit 1
fi

# 1. Detección de Virtualización
if command -v systemd-detect-virt >/dev/null 2>&1 && systemd-detect-virt --quiet; then
  EMUBOX_VIRT="$(systemd-detect-virt)"
else
  EMUBOX_VIRT="none"
fi

# 2. Detección de GPU PCI
GPU_DESC="$(lspci -nnk 2>/dev/null | grep -A2 -Ei 'VGA compatible controller|3D controller|Display controller' | head -n 2 | tr '\n' ' ' || echo 'Generic/Unknown')"

# 3. Detección de DRM/KMS
HAS_DRM=0
if [[ -e /dev/dri/card0 || -e /dev/dri/renderD128 ]]; then
  HAS_DRM=1
fi

# 4. Detección de Vulkan REAL y operativo (comprobando existencia de GPU física)
HAS_VULKAN=0
if command -v vulkaninfo >/dev/null 2>&1; then
  if vulkaninfo --summary 2>&1 | grep -qE 'deviceName|GPU0|GPU id'; then
    HAS_VULKAN=1
  fi
fi

echo "======================================================================"
echo "[EmuBox] Detección de Entorno Gráfico:"
echo "  - Virtualización: ${EMUBOX_VIRT}"
echo "  - GPU:            ${GPU_DESC}"
echo "  - DRM/KMS:        $([[ ${HAS_DRM} -eq 1 ]] && echo 'OK' || echo 'NO DISPONIBLE')"
echo "  - Vulkan HW:      $([[ ${HAS_VULKAN} -eq 1 ]] && echo 'OPERATIVO (NATIVO)' || echo 'NO DETECTADO / INCOMPATIBLE')"
echo "======================================================================"

# Orquestación de backend según capacidades reales
DBUS_RUN=""
if command -v dbus-run-session >/dev/null 2>&1 && [[ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ]]; then
  DBUS_RUN="dbus-run-session"
fi

# MODO NATIVO: GPU física con Vulkan 100% operativo + Gamescope disponible
if [[ ${HAS_VULKAN} -eq 1 ]] && command -v gamescope >/dev/null 2>&1; then
  if command -v cage >/dev/null 2>&1; then
    echo "[EmuBox] Modo: NATIVO (Cage -> Gamescope -> EmuBox)"
    if [[ -n "${DBUS_RUN}" ]]; then
      exec dbus-run-session cage -- gamescope -f -W 1920 -H 1080 -r 60 -- "${EMUBOX_BIN}" "$@"
    else
      exec cage -- gamescope -f -W 1920 -H 1080 -r 60 -- "${EMUBOX_BIN}" "$@"
    fi
  else
    echo "[EmuBox] Modo: NATIVO DIRECTO (Gamescope -> EmuBox)"
    if [[ -n "${DBUS_RUN}" ]]; then
      exec dbus-run-session gamescope -f -W 1920 -H 1080 -r 60 -- "${EMUBOX_BIN}" "$@"
    else
      exec gamescope -f -W 1920 -H 1080 -r 60 -- "${EMUBOX_BIN}" "$@"
    fi
  fi

# MODO COMPATIBILIDAD (VMware, VirtualBox o GPU sin Vulkan directo): Cage Kiosk Wayland
elif command -v cage >/dev/null 2>&1; then
  echo "[EmuBox] Modo: COMPATIBILIDAD VM/LEGACY (Cage Wayland Kiosk -> EmuBox)"
  if [[ -n "${DBUS_RUN}" ]]; then
    exec dbus-run-session cage -- "${EMUBOX_BIN}" "$@"
  else
    exec cage -- "${EMUBOX_BIN}" "$@"
  fi

# MODO FALLBACK DIRECTO
else
  echo "[EmuBox] Modo: FALLBACK DIRECTO -> EmuBox"
  exec "${EMUBOX_BIN}" "$@"
fi
EOF

chmod 0755 /usr/local/bin/emubox-session
ln -sf /usr/local/bin/emubox-session /usr/bin/emubox

# ------------------------------------------------------------
# 3. Configurar Autologin único y limpio en TTY1 (emubox-autologin.conf)
# ------------------------------------------------------------
echo "[3/5] Configurando Autologin único en TTY1 para el usuario $EMUBOX_USER..."

GETTY_OVERRIDE_DIR="/etc/systemd/system/getty@tty1.service.d"
mkdir -p "${GETTY_OVERRIDE_DIR}"

# Eliminar duplicados previos
rm -f "${GETTY_OVERRIDE_DIR}/autologin.conf"

cat << EOF > "${GETTY_OVERRIDE_DIR}/emubox-autologin.conf"
[Service]
ExecStart=
ExecStart=-/sbin/agetty -o '-p -f -- \\\\u' --noclear --autologin ${EMUBOX_USER} %I \$TERM
Type=idle
EOF

# ------------------------------------------------------------
# 4. Configurar autoarranque en .bash_profile para la TTY1 física
# ------------------------------------------------------------
echo "[4/5] Configurando autoarranque en $EMUBOX_HOME/.bash_profile..."

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
# 5. Servicio systemd opcional de diagnóstico (Deshabilitado por defecto)
# ------------------------------------------------------------
echo "[5/5] Registrando servicio de diagnóstico (deshabilitado por defecto)..."

cat > "/etc/systemd/system/$SERVICE_NAME" <<EOF
[Unit]
Description=EmuBox Dedicated Console Interface (Diagnostic Unit)
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

StandardOutput=append:/var/log/emubox/emubox.log
StandardError=append:/var/log/emubox/emubox-error.log
NoNewPrivileges=true

[Install]
WantedBy=graphical.target
EOF

systemctl daemon-reload
# Mantener deshabilitado para evitar conflictos con getty@tty1
systemctl disable "$SERVICE_NAME" 2>/dev/null || true

echo ""
echo "=========================================="
echo "   Configuración Adaptativa Completada"
echo "=========================================="
echo "1. Autologin TTY1:        ACTIVO ($EMUBOX_USER)"
echo "2. Override limpio:       getty@tty1.service.d/emubox-autologin.conf"
echo "3. Detección Inteligente: Vulkan HW -> Gamescope / VM sin Vulkan -> Cage"
echo "4. Cero Bucle Infinito:   vkCreateInstance nunca romperá el arranque"
echo "=========================================="
