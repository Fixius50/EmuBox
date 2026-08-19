#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX OS - SCRIPT UNIFICADO DE PREPARACION, INSTALACION Y DESPLIEGUE EN ARCH
#  Tauri v2 + SolidJS + Rust + WebKitGTK 4.1 + Estructura de Datos
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
  echo "Por favor, ejecutable con sudo:"
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
log_info "[1/8] Actualizando paquetes y repositorios de Arch Linux..."
pacman -Syu --noconfirm
log_ok "Sistema base actualizado."

# ------------------------------------------------------------------------------
# 3. Herramientas de compilacion y sistema
# ------------------------------------------------------------------------------
log_info "[2/8] Instalando herramientas de compilacion del sistema..."
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
log_info "[3/8] Instalando dependencias de runtime para Tauri v2..."
TAURI_PACKAGES=(
  webkit2gtk-4.1
  gtk3
)

pacman -S --needed --noconfirm "${TAURI_PACKAGES[@]}"
log_ok "Librerias WebKitGTK 4.1 y GTK3 instaladas."

# ------------------------------------------------------------------------------
# 5. Toolchains: Node.js, npm, Rust y Tauri CLI
# ------------------------------------------------------------------------------
log_info "[4/8] Configurando toolchains de desarrollo (Node.js, npm, Rust)..."

# Node.js + npm
pacman -S --needed --noconfirm nodejs npm
log_ok "Node.js: $(node --version) | npm: $(npm --version)"

# Rust toolchain
if ! command -v rustc >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
  log_step "Instalando rustup y configurando toolchain estable..."
  pacman -S --needed --noconfirm rustup
  if [[ -n "${CALLER_USER}" && "${CALLER_USER}" != "root" ]]; then
    sudo -u "${CALLER_USER}" rustup default stable || true
    sudo -u "${CALLER_USER}" rustup update stable || true
  else
    rustup default stable || true
    rustup update stable || true
  fi
else
  log_ok "Rust ya instalado: $(rustc --version)"
fi

# Tauri CLI global
if ! npm list -g @tauri-apps/cli >/dev/null 2>&1 && ! command -v cargo-tauri >/dev/null 2>&1; then
  log_step "Instalando @tauri-apps/cli globalmente..."
  npm install --global @tauri-apps/cli || true
fi
log_ok "Toolchains de Node.js, Rust y Tauri configurados."

# ------------------------------------------------------------------------------
# 6. Despliegue de la aplicacion EmuBox en /opt/emubox
# ------------------------------------------------------------------------------
log_info "[5/8] Desplegando codigo fuente de EmuBox en ${EMUBOX_DIR}..."

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
# 7. Instalacion de dependencias npm y compilacion de produccion
# ------------------------------------------------------------------------------
log_info "[6/8] Instalando dependencias npm y compilando interfaz..."

cd "${EMUBOX_DIR}"

if [[ -n "${CALLER_USER}" && "${CALLER_USER}" != "root" ]]; then
  if [[ -f "package-lock.json" ]]; then
    sudo -u "${CALLER_USER}" npm ci
  else
    sudo -u "${CALLER_USER}" npm install
  fi
  sudo -u "${CALLER_USER}" npm run build
else
  if [[ -f "package-lock.json" ]]; then
    npm ci
  else
    npm install
  fi
  npm run build
fi

log_ok "Compilacion de interfaz completada con exito."

# ------------------------------------------------------------------------------
# 8. Creacion de la infraestructura de datos y configuracion
# ------------------------------------------------------------------------------
log_info "[7/8] Creando estructura de directorios de datos y configuracion..."

# Directorios de sistema
mkdir -p /var/lib/emubox/{games,roms,saves,states,bios,covers,logs,screenshots}
mkdir -p /etc/emubox

# Directorios de usuario XDG
if [[ -n "${CALLER_USER}" && "${CALLER_USER}" != "root" ]]; then
  USER_HOME="$(getent passwd "${CALLER_USER}" | cut -d: -f6)"
  mkdir -p "${USER_HOME}/.local/share/emubox/"{roms,saves,states,bios,covers,logs,screenshots}
  mkdir -p "${USER_HOME}/.config/emubox"
  mkdir -p "${USER_HOME}/.cache/emubox"
  chown -R "${CALLER_USER}:${CALLER_USER}" "${USER_HOME}/.local/share/emubox" "${USER_HOME}/.config/emubox" "${USER_HOME}/.cache/emubox"
  chown -R "${CALLER_USER}:${CALLER_USER}" /var/lib/emubox
fi

# Crear script ejecutable /usr/local/bin/emubox
cat << 'EOF' > /usr/local/bin/emubox
#!/usr/bin/env bash
set -euo pipefail

EMUBOX_APP_DIR="/opt/emubox"
cd "${EMUBOX_APP_DIR}"

if [[ -f "${EMUBOX_APP_DIR}/src-tauri/target/release/emubox" ]]; then
  exec "${EMUBOX_APP_DIR}/src-tauri/target/release/emubox" "$@"
else
  exec npm run dev "$@"
fi
EOF
chmod +x /usr/local/bin/emubox

# Crear script de actualizacion rapida /usr/local/bin/emubox-update
cat << 'EOF' > /usr/local/bin/emubox-update
#!/usr/bin/env bash
set -euo pipefail

EMUBOX_APP_DIR="/opt/emubox"
echo "[EmuBox Update] Comprobando actualizaciones del repositorio..."

cd "${EMUBOX_APP_DIR}"
git fetch origin
git pull --ff-only

if [[ -f "package-lock.json" ]]; then
  npm ci
else
  npm install
fi

npm run build
echo "[OK] EmuBox actualizado correctamente."
EOF
chmod +x /usr/local/bin/emubox-update

log_ok "Comandos globales 'emubox' y 'emubox-update' registrados."

# ------------------------------------------------------------------------------
# 9. Verificacion final del entorno
# ------------------------------------------------------------------------------
log_info "[8/8] Verificacion integral del entorno..."

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
echo "  EMUBOX OS: INSTALACION Y PREPARACION COMPLETADA CON EXITO"
echo "==============================================================================="
echo ""
echo "Ubicacion de la aplicacion:  ${EMUBOX_DIR}"
echo "Directorio de datos (ROMs):  ~/.local/share/emubox/roms o /var/lib/emubox/roms"
echo "Directorio de partidas:      ~/.local/share/emubox/saves"
echo "Directorio de configuracion: ~/.config/emubox"
echo ""
echo "Comandos disponibles:"
echo "  emubox         -> Inicia la interfaz de EmuBox"
echo "  emubox-update  -> Actualiza EmuBox desde Git y recompila en caliente"
echo ""
echo "==============================================================================="
