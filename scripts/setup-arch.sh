#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX OS - SCRIPT UNIFICADO DE PREPARACION, INSTALACION Y DESPLIEGUE EN ARCH
#  Tauri v2 + SolidJS + Rust + WebKitGTK 4.1 + Estructura de Datos + Autoarranque
# ==============================================================================

set -Eeuo pipefail

EMUBOX_DIR="/opt/emubox"
REPO_URL="https://github.com/Fixius50/EmuBox.git"
CALLER_USER="${SUDO_USER:-$USER}"

# ------------------------------------------------------------------------------
# Funciones de salida (Cero Emojis)
# ------------------------------------------------------------------------------
log_info() {
  echo ""
  echo "[EmuBox] $1"
}

log_step() {
  echo "  -> $1"
}

log_ok() {
  echo "[OK] $1"
}

log_error() {
  echo "[ERROR] $1" >&2
}

# ------------------------------------------------------------------------------
# 1. Comprobaciones iniciales de entorno
# ------------------------------------------------------------------------------
log_info "Comprobando sistema operativo y permisos..."

if [[ ! -f /etc/arch-release ]]; then
  log_error "Este script esta disenado exclusivamente para Arch Linux."
  exit 1
fi

if [[ $EUID -ne 0 ]]; then
  log_error "Este script unificado requiere permisos de root para instalar dependencias."
  echo "Por favor, ejecutalo con sudo:"
  echo "  sudo ./scripts/setup-arch.sh"
  exit 1
fi

log_step "Comprobando conexion a Internet..."
if ! curl -fsI --max-time 10 https://archlinux.org >/dev/null; then
  log_error "No hay conexion a Internet. Verifica la red antes de continuar."
  exit 1
fi
log_ok "Sistema Arch Linux verificado y conexion activa."

# ------------------------------------------------------------------------------
# 2. Actualizacion del sistema
# ------------------------------------------------------------------------------
log_info "[1/9] Actualizando paquetes y repositorios de Arch Linux..."
pacman -Syu --noconfirm
log_ok "Sistema base actualizado."

# ------------------------------------------------------------------------------
# 3. Herramientas de compilacion y sistema
# ------------------------------------------------------------------------------
log_info "[2/9] Instalando herramientas de compilacion del sistema..."
COMPILATION_PACKAGES=(
  base-devel
  git
  curl
  wget
  file
  unzip
  zip
  pkgconf
  openssl
  librsvg
  xdotool
)

pacman -S --needed --noconfirm "${COMPILATION_PACKAGES[@]}"
log_ok "Herramientas de compilacion instaladas."

# ------------------------------------------------------------------------------
# 4. Dependencias graficas y runtime de Tauri v2
# ------------------------------------------------------------------------------
log_info "[3/9] Instalando dependencias de runtime para Tauri v2..."
TAURI_PACKAGES=(
  webkit2gtk-4.1
  gtk3
)

pacman -S --needed --noconfirm "${TAURI_PACKAGES[@]}"
log_ok "Librerias WebKitGTK 4.1 y GTK3 instaladas."

# ------------------------------------------------------------------------------
# 5. Toolchains: Node.js, npm, Rust y Tauri CLI
# ------------------------------------------------------------------------------
log_info "[4/9] Configurando toolchains de desarrollo (Node.js, npm, Rust)..."

# Node.js + npm
pacman -S --needed --noconfirm nodejs npm
log_ok "Node.js: $(node --version) | npm: $(npm --version)"

# Rust toolchain (pacman oficial o rustup)
if ! command -v rustc >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
  log_step "Instalando compilador Rust y Cargo oficial de Arch Linux..."
  pacman -S --needed --noconfirm rust cargo || {
    log_step "Instalando rustup como alternativa..."
    pacman -S --needed --noconfirm rustup
    rustup default stable || true
  }
fi

export PATH="$HOME/.cargo/bin:/usr/local/bin:$PATH"
if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck source=/dev/null
  . "$HOME/.cargo/env" || true
fi

log_ok "Rust: $(rustc --version 2>/dev/null || echo 'instalado') | Cargo: $(cargo --version 2>/dev/null || echo 'instalado')"
log_ok "Toolchains de Node.js y Rust configurados."

# ------------------------------------------------------------------------------
# 6. Despliegue de la aplicacion EmuBox en /opt/emubox
# ------------------------------------------------------------------------------
log_info "[5/9] Desplegando codigo fuente de EmuBox en ${EMUBOX_DIR}..."

CURRENT_DIR="$(pwd)"
mkdir -p "$(dirname "${EMUBOX_DIR}")"

if [[ -d "${EMUBOX_DIR}/.git" ]]; then
  log_step "Repositorio existente detectado en ${EMUBOX_DIR}. Actualizando via git pull..."
  git -C "${EMUBOX_DIR}" fetch origin || true
  git -C "${EMUBOX_DIR}" pull --ff-only || true
elif [[ -d "${CURRENT_DIR}/.git" && -f "${CURRENT_DIR}/package.json" ]]; then
  log_step "Desplegando desde el arbol local actual (${CURRENT_DIR}) hacia ${EMUBOX_DIR}..."
  mkdir -p "${EMUBOX_DIR}"
  cp -r "${CURRENT_DIR}/." "${EMUBOX_DIR}/"
else
  log_step "Clonando repositorio remoto en ${EMUBOX_DIR}..."
  git clone "${REPO_URL}" "${EMUBOX_DIR}"
fi

# Ajustar permisos para el usuario normal
if [[ -n "${CALLER_USER}" && "${CALLER_USER}" != "root" ]]; then
  chown -R "${CALLER_USER}:${CALLER_USER}" "${EMUBOX_DIR}"
fi

log_ok "Codigo de EmuBox preparado en ${EMUBOX_DIR}."

# ------------------------------------------------------------------------------
# 7. Instalacion de dependencias npm y compilacion de produccion (Frontend + Tauri)
# ------------------------------------------------------------------------------
log_info "[6/9] Instalando dependencias y compilando EmuBox en modo producción..."

cd "${EMUBOX_DIR}"

if [[ -n "${CALLER_USER}" && "${CALLER_USER}" != "root" ]]; then
  log_step "Instalando dependencias npm..."
  sudo -u "${CALLER_USER}" env PATH="${PATH}" npm install --no-audit --no-fund
  log_step "Compilando frontend SolidJS..."
  sudo -u "${CALLER_USER}" env PATH="${PATH}" npm run build

  log_step "Compilando binario nativo Tauri (Release)..."
  sudo -u "${CALLER_USER}" env PATH="${PATH}" cargo build --release --manifest-path src-tauri/Cargo.toml
else
  log_step "Instalando dependencias npm..."
  npm install --no-audit --no-fund
  log_step "Compilando frontend SolidJS..."
  npm run build

  log_step "Compilando binario nativo Tauri (Release)..."
  cargo build --release --manifest-path src-tauri/Cargo.toml
fi

mkdir -p "${EMUBOX_DIR}/bin"
if [[ -f "${EMUBOX_DIR}/src-tauri/target/release/emubox" ]]; then
  cp "${EMUBOX_DIR}/src-tauri/target/release/emubox" "${EMUBOX_DIR}/bin/emubox"
  chmod +x "${EMUBOX_DIR}/bin/emubox"
fi

log_ok "Compilacion nativa de EmuBox completada con exito."

# ------------------------------------------------------------------------------
# 8. Creacion de la infraestructura de datos y comandos ejecutables
# ------------------------------------------------------------------------------
log_info "[7/9] Creando estructura de directorios de datos y comandos globales..."

# Directorios de sistema y logs
mkdir -p /var/lib/emubox/{games,roms,saves,states,bios,covers,logs,screenshots}
mkdir -p /etc/emubox
mkdir -p /var/log/emubox

# Directorios de usuario XDG
if [[ -n "${CALLER_USER}" && "${CALLER_USER}" != "root" ]]; then
  USER_HOME="$(getent passwd "${CALLER_USER}" | cut -d: -f6)"
  mkdir -p "${USER_HOME}/.local/share/emubox/"{roms,saves,states,bios,covers,logs,screenshots}
  mkdir -p "${USER_HOME}/.config/emubox"
  mkdir -p "${USER_HOME}/.cache/emubox"
  chown -R "${CALLER_USER}:${CALLER_USER}" "${USER_HOME}/.local/share/emubox" "${USER_HOME}/.config/emubox" "${USER_HOME}/.cache/emubox"
  chown -R "${CALLER_USER}:${CALLER_USER}" /var/lib/emubox
  chown -R "${CALLER_USER}:${CALLER_USER}" /var/log/emubox
fi

chmod -R 755 /var/log/emubox

# Crear script ejecutable /usr/local/bin/emubox y enlace /usr/bin/emubox
cat << 'EOF' > /usr/local/bin/emubox
#!/usr/bin/env bash
set -euo pipefail

EMUBOX_APP_DIR="/opt/emubox"
EMUBOX_BIN="${EMUBOX_APP_DIR}/bin/emubox"

if [[ -x "${EMUBOX_BIN}" ]]; then
  exec "${EMUBOX_BIN}" "$@"
elif [[ -x "${EMUBOX_APP_DIR}/src-tauri/target/release/emubox" ]]; then
  exec "${EMUBOX_APP_DIR}/src-tauri/target/release/emubox" "$@"
else
  echo "[ERROR] El binario de producción de EmuBox no se encuentra compilado." >&2
  echo "Ejecuta 'emubox-update' para compilar el binario nativo." >&2
  exit 1
fi
EOF
chmod +x /usr/local/bin/emubox
ln -sf /usr/local/bin/emubox /usr/bin/emubox

# Crear script de actualizacion rapida /usr/local/bin/emubox-update
cat << 'EOF' > /usr/local/bin/emubox-update
#!/usr/bin/env bash
set -euo pipefail

EMUBOX_APP_DIR="/opt/emubox"
echo "[EmuBox Update] Comprobando actualizaciones del repositorio..."

cd "${EMUBOX_APP_DIR}"
git fetch origin
git pull --ff-only

npm install --no-audit --no-fund

npm run build
cargo build --release --manifest-path src-tauri/Cargo.toml

mkdir -p "${EMUBOX_APP_DIR}/bin"
cp "${EMUBOX_APP_DIR}/src-tauri/target/release/emubox" "${EMUBOX_APP_DIR}/bin/emubox"
chmod +x "${EMUBOX_APP_DIR}/bin/emubox"

echo "[OK] EmuBox compilado y actualizado en modo producción."
EOF
chmod +x /usr/local/bin/emubox-update
ln -sf /usr/local/bin/emubox-update /usr/bin/emubox-update

log_ok "Comandos globales 'emubox' y 'emubox-update' registrados."

# ------------------------------------------------------------------------------
# 9. Configuracion del servicio de autoarranque systemd
# ------------------------------------------------------------------------------
log_info "[8/9] Configurando servicio systemd para autoarranque (emubox.service)..."

SERVICE_NAME="emubox.service"
USER_HOME="$(getent passwd "${CALLER_USER}" | cut -d: -f6)"

cat > "/etc/systemd/system/${SERVICE_NAME}" <<EOF
[Unit]
Description=EmuBox Console Interface
After=graphical.target systemd-user-sessions.service
Wants=graphical.target

[Service]
Type=simple

User=${CALLER_USER}
Group=${CALLER_USER}

WorkingDirectory=/opt/emubox

ExecStart=/usr/bin/emubox

Restart=on-failure
RestartSec=3

# Variables generales
Environment=HOME=${USER_HOME}
Environment=EMUBOX_HOME=/var/lib/emubox
Environment=NODE_ENV=production

# Logs
StandardOutput=append:/var/log/emubox/emubox.log
StandardError=append:/var/log/emubox/emubox-error.log

# Seguridad básica
NoNewPrivileges=true

[Install]
WantedBy=graphical.target
EOF

systemctl daemon-reload
systemctl enable "${SERVICE_NAME}"
log_ok "Servicio ${SERVICE_NAME} habilitado para autoarranque tras la sesion grafica."

# ------------------------------------------------------------------------------
# 10. Verificacion integral del entorno
# ------------------------------------------------------------------------------
log_info "[9/9] Verificacion integral del entorno..."

CHECK_FAILED=0

check_cmd() {
  local cmd="$1"
  if command -v "$cmd" >/dev/null 2>&1; then
    echo "  [OK] $cmd -> $(command -v "$cmd")"
  else
    echo "  [FALTA] $cmd"
    CHECK_FAILED=1
  fi
}

check_cmd git
check_cmd node
check_cmd npm
check_cmd rustc
check_cmd cargo
check_cmd emubox
check_cmd emubox-update

echo ""
log_step "Comprobando paquetes criticos del sistema..."
CRITICAL_PKGS=(
  webkit2gtk-4.1
  gtk3
  base-devel
  openssl
  librsvg
)

for pkg in "${CRITICAL_PKGS[@]}"; do
  if pacman -Q "$pkg" >/dev/null 2>&1; then
    echo "  [OK] $pkg"
  else
    echo "  [FALTA] $pkg"
    CHECK_FAILED=1
  fi
done

echo ""

if [[ "$CHECK_FAILED" -ne 0 ]]; then
  log_error "La instalacion finalizo con advertencias. Revisa los elementos marcados como [FALTA]."
  exit 1
fi

echo "==============================================================================="
echo "  EMUBOX OS: INSTALACION Y AUTOARRANQUE PREPARADOS CON EXITO"
echo "==============================================================================="
echo ""
echo "Ubicacion de la aplicacion:  ${EMUBOX_DIR}"
echo "Directorio de datos (ROMs):  ~/.local/share/emubox/roms o /var/lib/emubox/roms"
echo "Directorio de partidas:      ~/.local/share/emubox/saves"
echo "Directorio de configuracion: ~/.config/emubox"
echo "Ficheros de log:             /var/log/emubox/emubox.log"
echo ""
echo "Comandos disponibles:"
echo "  emubox                 -> Inicia la interfaz de EmuBox"
echo "  emubox-update          -> Actualiza EmuBox desde Git y recompila"
echo "  sudo systemctl status emubox -> Comprueba el estado del servicio"
echo ""
echo "==============================================================================="
