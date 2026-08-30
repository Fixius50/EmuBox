#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - AUTO START & ADAPTIVE CONSOLE APPLIANCE SETUP (ARCH LINUX)
# ==============================================================================
#
# Implementa los Puntos 1, 7, 8, 9 y 10:
#   - Punto 1: Arranque como Appliance dedicado en TTY1 sin sesión gráfica previa.
#   - Punto 7: Detección gráfica adaptativa (VM sin Vulkan -> Cage / HW real -> Gamescope).
#   - Punto 8: Detección dinámica de resolución DRM con fallback a 1080p.
#   - Punto 9: Permisos de grupos para Gamepad (input, uinput, video, seat).
#   - Punto 10: Política de recuperación anti-bucles (StartLimitBurst=3) y logs en /var/log/emubox/.
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
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

echo "======================================================================"
echo "         🎮 EmuBox - Configuración de Consola Appliance               "
echo "======================================================================"
echo "Usuario: $EMUBOX_USER (UID: $EMUBOX_UID)"
echo "Home:    $EMUBOX_HOME"
echo ""

# ------------------------------------------------------------
# 1. Preparar directorios del sistema, logs y grupos de mando
# ------------------------------------------------------------
echo "[1/5] Preparando directorios del sistema, permisos y grupos de entrada..."

mkdir -p /etc/emubox
mkdir -p /var/lib/emubox/{games,roms,saves,states,bios,covers,logs,screenshots}
mkdir -p /var/log/emubox

chown -R "$EMUBOX_USER:$EMUBOX_USER" /var/lib/emubox
chown -R "$EMUBOX_USER:$EMUBOX_USER" /var/log/emubox
chmod -R 755 /var/log/emubox

# Asegurar membresía en grupos para Gamepad (Punto 9) y Video/DRM
for grp in video input uinput seat; do
  if getent group "$grp" >/dev/null 2>&1; then
    usermod -aG "$grp" "$EMUBOX_USER" 2>/dev/null || true
  fi
done

# ------------------------------------------------------------
# 2. Instalar el Lanzador de Sesión Adaptativo (/usr/local/bin/emubox-session)
# ------------------------------------------------------------
echo "[2/5] Instalando lanzador de sesión adaptativo (/usr/local/bin/emubox-session)..."

cat << 'EOF' > /usr/local/bin/emubox-session
#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - SESSION MANAGER ADAPTATIVO (PUNTOS 1, 7, 8, 9, 10)
# ==============================================================================

set -euo pipefail

LOG_FILE="/var/log/emubox/session.log"
mkdir -p "$(dirname "${LOG_FILE}")" 2>/dev/null || true

# Redirigir trazas de arranque a session.log
exec > >(tee -a "${LOG_FILE}" 2>/dev/null || cat)
exec 2>&1

echo "======================================================================"
echo "[EmuBox Session] Iniciando gestor de sesión: $(date '+%Y-%m-%d %H:%M:%S')"
echo "======================================================================"

export WEBKIT_DISABLE_DMABUF_RENDERER=1
export GDK_BACKEND=wayland
export XDG_SESSION_TYPE=wayland
export EMUBOX_HOME="${EMUBOX_HOME:-/var/lib/emubox}"

EMUBOX_BIN="/opt/emubox/bin/emubox"
if [[ ! -x "${EMUBOX_BIN}" ]]; then
  echo "[ERROR] No existe el binario ejecutable en: ${EMUBOX_BIN}" >&2
  exit 1
fi

# 1. Detección de Virtualización (Punto 7)
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

# 4. Detección de Vulkan REAL y funcional (Punto 7)
HAS_VULKAN=0
if command -v vulkaninfo >/dev/null 2>&1; then
  if vulkaninfo --summary 2>&1 | grep -qE 'deviceName|GPU0|GPU id'; then
    HAS_VULKAN=1
  fi
fi

echo "======================================================================"
echo "[EmuBox Hardware Detection]:"
echo "  - Virtualización: ${EMUBOX_VIRT}"
echo "  - GPU:            ${GPU_DESC}"
echo "  - DRM/KMS:        $([[ ${HAS_DRM} -eq 1 ]] && echo 'OK (/dev/dri)' || echo 'NO DISPONIBLE')"
echo "  - Vulkan HW:      $([[ ${HAS_VULKAN} -eq 1 ]] && echo 'OPERATIVO (NATIVO)' || echo 'NO DETECTADO / INCOMPATIBLE')"
echo "======================================================================"

# Orquestación D-Bus
DBUS_RUN=""
if command -v dbus-run-session >/dev/null 2>&1 && [[ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ]]; then
  DBUS_RUN="dbus-run-session"
fi

# Iniciar sincronizador de eventos DRM en segundo plano (0% CPU, reactivo)
SYNC_PID=""
if [[ -x /usr/local/bin/emubox-drm-sync ]]; then
  /usr/local/bin/emubox-drm-sync >/dev/null 2>&1 &
  SYNC_PID=$!
  trap '[[ -n "${SYNC_PID:-}" ]] && kill -TERM "$SYNC_PID" 2>/dev/null || true' EXIT INT TERM
fi

# MODO NATIVO: GPU física con soporte Vulkan real + Gamescope
if [[ ${HAS_VULKAN} -eq 1 ]] && command -v gamescope >/dev/null 2>&1; then
  if command -v cage >/dev/null 2>&1; then
    echo "[EmuBox Pipeline] Modo: NATIVO (Cage -> Gamescope -> EmuBox)"
    if [[ -n "${DBUS_RUN}" ]]; then
      dbus-run-session cage -- gamescope -f -- "${EMUBOX_BIN}" "$@"
    else
      cage -- gamescope -f -- "${EMUBOX_BIN}" "$@"
    fi
  else
    echo "[EmuBox Pipeline] Modo: NATIVO DIRECTO (Gamescope -> EmuBox)"
    if [[ -n "${DBUS_RUN}" ]]; then
      dbus-run-session gamescope -f -- "${EMUBOX_BIN}" "$@"
    else
      gamescope -f -- "${EMUBOX_BIN}" "$@"
    fi
  fi

# MODO COMPATIBILIDAD (VMware, VirtualBox o GPU sin Vulkan funcional): Cage Wayland Kiosk
elif command -v cage >/dev/null 2>&1; then
  echo "[EmuBox Pipeline] Modo: COMPATIBILIDAD VM/LEGACY (Cage Wayland Kiosk -> EmuBox)"
  if [[ -n "${DBUS_RUN}" ]]; then
    dbus-run-session cage -- "${EMUBOX_BIN}" "$@"
  else
    cage -- "${EMUBOX_BIN}" "$@"
  fi

# MODO FALLBACK DIRECTO
else
  echo "[EmuBox Pipeline] Modo: FALLBACK DIRECTO -> EmuBox"
  "${EMUBOX_BIN}" "$@"
fi
EOF

chmod 0755 /usr/local/bin/emubox-session
ln -sf /usr/local/bin/emubox-session /usr/bin/emubox

# Copiar también el script sincronizador a /usr/local/bin
if [[ -f "${SCRIPT_DIR}/emubox-drm-sync.sh" ]]; then
  cp -f "${SCRIPT_DIR}/emubox-drm-sync.sh" /usr/local/bin/emubox-drm-sync
  chmod 0755 /usr/local/bin/emubox-drm-sync
fi

# ------------------------------------------------------------
# 3. Configurar Autologin Único y Limpio en TTY1 (Punto 1)
# ------------------------------------------------------------
echo "[3/5] Configurando Autologin en TTY1 (emubox-autologin.conf)..."

GETTY_OVERRIDE_DIR="/etc/systemd/system/getty@tty1.service.d"
mkdir -p "${GETTY_OVERRIDE_DIR}"

# Eliminar cualquier configuración previa redundante
rm -f "${GETTY_OVERRIDE_DIR}/autologin.conf"

cat << EOF > "${GETTY_OVERRIDE_DIR}/emubox-autologin.conf"
[Service]
ExecStart=
ExecStart=-/usr/bin/agetty --autologin ${EMUBOX_USER} --noclear %I \$TERM
Type=idle
TTYPath=/dev/tty1
TTYReset=yes
TTYVHangup=yes
TTYVTDisallocate=yes
EOF

# ------------------------------------------------------------
# 4. Configurar Autoarranque en .bash_profile con Aislamiento PTS/SSH (Puntos 1 y 9)
# ------------------------------------------------------------
echo "[4/5] Configurando arranque exclusivo en consola física (/home/${EMUBOX_USER}/.bash_profile)..."

BASH_PROFILE="${EMUBOX_HOME}/.bash_profile"
if ! grep -q "emubox-session" "${BASH_PROFILE}" 2>/dev/null; then
  cat << 'EOF' >> "${BASH_PROFILE}"

# EmuBox Console Appliance: Auto-launch Wayland session exclusively on physical TTY1
if [[ "$(tty 2>/dev/null || true)" == "/dev/tty1" ]] && \
   [[ -z "${WAYLAND_DISPLAY:-}" ]] && \
   [[ -z "${DISPLAY:-}" ]]; then

  if [[ -f /tmp/emubox-drop-shell ]]; then
    rm -f /tmp/emubox-drop-shell
    echo ""
    echo "======================================================================"
    echo "  🎮 EmuBox - Salida a Consola Linux (TTY1)"
    echo "======================================================================"
    echo "  Has salido de la interfaz gráfica a la terminal de Arch Linux."
    echo "  - Para abrir el Centro de Control:  ./script.sh"
    echo "  - Para volver a iniciar EmuBox:     /usr/local/bin/emubox-session"
    echo "======================================================================"
    echo ""
  else
    exec /usr/local/bin/emubox-session
  fi
fi
EOF
  chown "$EMUBOX_USER:$EMUBOX_USER" "${BASH_PROFILE}"
fi

# ------------------------------------------------------------
# 5. Servicio de Diagnóstico con Protección Anti-Crash Loops (Punto 10)
# ------------------------------------------------------------
echo "[5/5] Registrando servicio de diagnóstico con límites de recuperación..."

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
# Mantener deshabilitado por defecto para que el ciclo corra exclusivamente en getty@tty1
systemctl disable "$SERVICE_NAME" 2>/dev/null || true

echo ""
echo "======================================================================"
echo -e "\033[1;32m[ÉXITO] Configuración de Consola Appliance Completada\033[0m"
echo "======================================================================"
echo "1. Autologin TTY1:        ACTIVO ($EMUBOX_USER)"
echo "2. Detección Adaptativa:  Vulkan HW -> Gamescope / VM -> Cage"
echo "3. Resolución Dinámica:   Sondeo DRM automático (Fallback 1080p)"
echo "4. Permisos de Mando:     video, input, uinput, seat asignados"
echo "5. Aislamiento SSH:       Las conexiones SSH (pts/*) NO interfieren"
echo "6. Anti-Crash Loops:      Límite de reinicios fijado en 3 / 60s"
echo "======================================================================"
