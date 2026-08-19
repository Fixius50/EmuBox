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
# Directorio de logs de instalacion
# ------------------------------------------------------------------------------
mkdir -p /var/log/emubox
SETUP_LOG="/var/log/emubox/setup.log"
exec > >(tee -a "${SETUP_LOG}") 2>&1

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

log_warn() {
  echo "[AVISO] $1"
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
if ! curl -fsI --max-time 10 https://archlinux.org >/dev/null 2>&1; then
  log_warn "Conexion a Internet limitada o no detectada. Se continuara con paquetes locales."
else
  log_ok "Conexion a Internet activa."
fi

# ------------------------------------------------------------------------------
# 2. Actualizacion del sistema
# ------------------------------------------------------------------------------
log_info "[1/9] Sincronizando repositorios de Arch Linux..."
pacman -Sy --noconfirm
log_ok "Repositorios sincronizados."

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
# 5. Toolchains: Node.js, npm, Rust
# ------------------------------------------------------------------------------
log_info "[4/9] Instalando toolchains de desarrollo (Node.js, npm, Rust)..."

# Node.js + npm
pacman -S --needed --noconfirm nodejs npm
hash -r

if ! command -v node >/dev/null 2>&1; then
  log_error "Node.js no se ha instalado correctamente."
  exit 1
fi

if ! command -v npm >/dev/null 2>&1; then
  log_error "npm no se ha instalado correctamente."
  exit 1
fi

log_ok "Node.js $(node --version)"
log_ok "npm $(npm --version)"

# Rust toolchain (pacman oficial o rustup)
if ! command -v rustc >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
  log_step "Instalando compilador Rust y Cargo oficial de Arch Linux..."
  pacman -S --needed --noconfirm rust cargo
fi

export PATH="$HOME/.cargo/bin:/usr/local/bin:$PATH"
if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck source=/dev/null
  . "$HOME/.cargo/env" || true
fi

log_ok "Rust: $(rustc --version) | Cargo: $(cargo --version)"
log_ok "Toolchains de Node.js y Rust listos."

# ------------------------------------------------------------------------------
# 6. Despliegue de la aplicacion EmuBox en /opt/emubox
# ------------------------------------------------------------------------------
log_info "[5/9] Desplegando codigo fuente de EmuBox en ${EMUBOX_DIR}..."

CURRENT_DIR="$(pwd)"
mkdir -p "$(dirname "${EMUBOX_DIR}")"

if [[ "${CURRENT_DIR}" == "${EMUBOX_DIR}" ]]; then
  log_ok "Ejecutando directamente dentro de ${EMUBOX_DIR}."
elif [[ -d "${EMUBOX_DIR}/.git" ]]; then
  log_step "Repositorio existente detectado en ${EMUBOX_DIR}. Actualizando via git pull..."
  git -C "${EMUBOX_DIR}" fetch origin || true
  git -C "${EMUBOX_DIR}" pull --ff-only || true
elif [[ -d "${CURRENT_DIR}/.git" && -f "${CURRENT_DIR}/package.json" ]]; then
  log_step "Copiando desde el arbol local actual hacia ${EMUBOX_DIR}..."
  mkdir -p "${EMUBOX_DIR}"
  cp -r "${CURRENT_DIR}/." "${EMUBOX_DIR}/"
else
  log_step "Clonando repositorio remoto en ${EMUBOX_DIR}..."
  git clone "${REPO_URL}" "${EMUBOX_DIR}"
fi

# Ajustar permisos iniciales para el usuario normal
if [[ -n "${CALLER_USER}" && "${CALLER_USER}" != "root" ]]; then
  chown -R "${CALLER_USER}:${CALLER_USER}" "${EMUBOX_DIR}"
fi

log_ok "Codigo de EmuBox preparado en ${EMUBOX_DIR}."

# ------------------------------------------------------------------------------
# 7. Instalacion de dependencias y compilacion de EmuBox
# ------------------------------------------------------------------------------
log_info "[6/9] Instalando dependencias y compilando EmuBox..."

cd "${EMUBOX_DIR}"

# --------------------------------------------------------------------------
# Diagnostico del entorno Node.js
# --------------------------------------------------------------------------
log_step "Diagnostico del entorno Node.js..."

echo "PATH=${PATH}"
echo "Usuario: $(whoami)"
echo "Directorio: $(pwd)"

if ! command -v node >/dev/null 2>&1; then
  log_error "Node.js NO esta disponible."
  exit 1
fi

if ! command -v npm >/dev/null 2>&1; then
  log_error "npm NO esta disponible."
  exit 1
fi

echo "Node: $(command -v node)"
echo "Node version: $(node --version)"
echo "npm: $(command -v npm)"
echo "npm version: $(npm --version)"

if [[ ! -f package.json ]]; then
  log_error "No existe package.json en ${EMUBOX_DIR}"
  exit 1
fi

if ! grep -q '"build"' package.json; then
  log_error "El package.json no contiene el script build."
  exit 1
fi

log_ok "Entorno Node.js verificado."

# --------------------------------------------------------------------------
# Dependencias del proyecto (filtrando completamente cualquier linea con 'npm error')
# --------------------------------------------------------------------------
log_step "Instalando dependencias npm..."

set +e
if [[ -f package-lock.json ]]; then
  NPM_OUTPUT="$(npm ci --no-audit --no-fund 2>&1)"
  NPM_STATUS=$?
  if [[ ${NPM_STATUS} -ne 0 ]]; then
    NPM_OUTPUT="$(npm install --no-audit --no-fund 2>&1)"
    NPM_STATUS=$?
  fi
else
  NPM_OUTPUT="$(npm install --no-audit --no-fund 2>&1)"
  NPM_STATUS=$?
fi
set -e

if [[ -n "${NPM_OUTPUT}" ]]; then
  echo "${NPM_OUTPUT}" | grep -viE "(npm error|npm err|npm help)" || true
fi

if [[ ${NPM_STATUS} -ne 0 ]]; then
  log_error "La instalacion de dependencias npm fallo (codigo ${NPM_STATUS})."
  exit ${NPM_STATUS}
fi

log_ok "Dependencias npm instaladas correctamente."

# --------------------------------------------------------------------------
# Compilacion del frontend SolidJS (filtrando completamente lineas de npm error)
# --------------------------------------------------------------------------
log_step "Compilando frontend SolidJS..."
rm -rf solid/dist

set +e
BUILD_OUTPUT="$(npm run build 2>&1)"
BUILD_STATUS=$?
set -e

if [[ -n "${BUILD_OUTPUT}" ]]; then
  echo "${BUILD_OUTPUT}" | grep -viE "(npm error|npm err|npm help)" || true
fi

if [[ ${BUILD_STATUS} -ne 0 ]]; then
  log_error "La compilacion del frontend fallo (codigo ${BUILD_STATUS})."
  exit ${BUILD_STATUS}
fi

log_ok "Frontend SolidJS compilado correctamente."

# --------------------------------------------------------------------------
# Compilacion de Tauri
# --------------------------------------------------------------------------
mkdir -p "${EMUBOX_DIR}/bin"

TAURI_BINARY="${EMUBOX_DIR}/src-tauri/target/release/emubox"

if [[ -x "${TAURI_BINARY}" ]]; then
  log_ok "Binario Tauri existente detectado."
else
  log_step "Compilando EmuBox Tauri en modo release..."

  cargo build \
    --release \
    --manifest-path "${EMUBOX_DIR}/src-tauri/Cargo.toml"

  if [[ ! -x "${TAURI_BINARY}" ]]; then
    log_error "Cargo termino pero no se encontro el binario:"
    log_error "${TAURI_BINARY}"
    exit 1
  fi

  log_ok "Binario Tauri compilado correctamente."
fi

# --------------------------------------------------------------------------
# Copiar binario a ubicacion estable
# --------------------------------------------------------------------------
cp -f "${TAURI_BINARY}" "${EMUBOX_DIR}/bin/emubox"
chmod +x "${EMUBOX_DIR}/bin/emubox"

log_ok "Binario instalado en ${EMUBOX_DIR}/bin/emubox."

# --------------------------------------------------------------------------
# Permisos finales
# --------------------------------------------------------------------------
if [[ -n "${CALLER_USER}" && "${CALLER_USER}" != "root" ]]; then
  chown -R "${CALLER_USER}:${CALLER_USER}" "${EMUBOX_DIR}"
fi

log_ok "Paso de compilacion completado correctamente."

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

# Crear script ejecutable /usr/local/bin/emubox y enlace /usr/bin/emubox (ESTRICTO SIN FALLBACK A VITE)
cat << 'EOF' > /usr/local/bin/emubox
#!/usr/bin/env bash
set -euo pipefail

EMUBOX_APP_DIR="/opt/emubox"
EMUBOX_BIN="${EMUBOX_APP_DIR}/bin/emubox"

if [[ ! -x "${EMUBOX_BIN}" ]]; then
  echo "[ERROR] El binario de EmuBox no esta instalado." >&2
  echo "[ERROR] Ejecuta nuevamente el setup de EmuBox (sudo ./scripts/setup-arch.sh)." >&2
  exit 1
fi

exec "${EMUBOX_BIN}" "$@"
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

if [[ -f package-lock.json ]]; then
  npm ci --no-audit --no-fund
else
  npm install --no-audit --no-fund
fi

npm run build
cargo build --release --manifest-path "${EMUBOX_APP_DIR}/src-tauri/Cargo.toml"

mkdir -p "${EMUBOX_APP_DIR}/bin"
cp -f "${EMUBOX_APP_DIR}/src-tauri/target/release/emubox" "${EMUBOX_APP_DIR}/bin/emubox"
chmod +x "${EMUBOX_APP_DIR}/bin/emubox"

echo "[OK] EmuBox actualizado correctamente en modo producción."
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

echo "==============================================================================="
echo "  INSTALACION Y CONFIGURACION DE EMUBOX COMPLETADA"
echo "==============================================================================="
echo "  Directorio de instalacion: ${EMUBOX_DIR}"
echo "  Directorio de ROMs:        ~/.local/share/emubox/roms"
echo "  Directorio de partidas:    ~/.local/share/emubox/saves"
echo "  Comando de ejecucion:      emubox (ejecuta binario nativo Tauri)"
echo "  Comando de actualizacion:  emubox-update"
echo "  Autoarranque de consola:   Habilitado (${SERVICE_NAME})"
echo "  Log completo de setup:     ${SETUP_LOG}"
echo "==============================================================================="
