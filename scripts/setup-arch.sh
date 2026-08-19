#!/usr/bin/env bash
set -Eeuo pipefail

# ==============================================================================
# EMUBOX OS
# INSTALADOR MAESTRO PARA ARCH LINUX
#
# Hace:
#   1. Comprobacion del sistema
#   2. Configuracion regional
#   3. Usuario + sudo
#   4. NetworkManager
#   5. Git y herramientas base
#   6. Node.js + npm
#   7. Rust + Cargo
#   8. GTK/WebKitGTK para Tauri
#   9. Clonado/actualizacion de EmuBox
#  10. Build de SolidJS + Tauri
#  11. Estructura de datos
#  12. Comando global emubox
#  13. emubox-update
#  14. Servicio systemd
#  15. Verificacion final
#
# NO PARTICIONA NI FORMATEA DISCOS.
# ==============================================================================

set -E

# ------------------------------------------------------------------------------
# CONFIGURACION
# ------------------------------------------------------------------------------

EMUBOX_DIR="/opt/emubox"
REPO_URL="https://github.com/Fixius50/EmuBox.git"

EMUBOX_USER="${EMUBOX_USER:-emubox}"
EMUBOX_GROUP="${EMUBOX_GROUP:-emubox}"

LOG_DIR="/var/log/emubox"
SETUP_LOG="${LOG_DIR}/setup.log"
NPM_INSTALL_LOG="${LOG_DIR}/npm-install.log"
NPM_ESBUILD_LOG="${LOG_DIR}/npm-esbuild.log"
NPM_BUILD_LOG="${LOG_DIR}/npm-build.log"
CARGO_BUILD_LOG="${LOG_DIR}/cargo-build.log"

# ------------------------------------------------------------------------------
# COLORES
# ------------------------------------------------------------------------------

if [[ -t 1 ]]; then
    C_RESET='\033[0m'
    C_BLUE='\033[1;34m'
    C_GREEN='\033[1;32m'
    C_YELLOW='\033[1;33m'
    C_RED='\033[1;31m'
else
    C_RESET=''
    C_BLUE=''
    C_GREEN=''
    C_YELLOW=''
    C_RED=''
fi

# ------------------------------------------------------------------------------
# FUNCIONES
# ------------------------------------------------------------------------------

log_info() {
    echo ""
    echo -e "${C_BLUE}[EmuBox]${C_RESET} $*"
}

log_step() {
    echo "  -> $*"
}

log_ok() {
    echo -e "${C_GREEN}[OK]${C_RESET} $*"
}

log_warn() {
    echo -e "${C_YELLOW}[AVISO]${C_RESET} $*"
}

log_error() {
    echo -e "${C_RED}[ERROR]${C_RESET} $*" >&2
}

die() {
    log_error "$*"
    exit 1
}

# ------------------------------------------------------------------------------
# CAPTURA GLOBAL DE LOG
# ------------------------------------------------------------------------------

mkdir -p "${LOG_DIR}"

exec > >(tee -a "${SETUP_LOG}") 2>&1

# ------------------------------------------------------------------------------
# MANEJADOR DE ERRORES
# ------------------------------------------------------------------------------

on_error() {
    local exit_code=$?
    local line_number=$1

    echo ""
    log_error "El instalador ha fallado."
    log_error "Linea: ${line_number}"
    log_error "Codigo: ${exit_code}"
    log_error "Log general: ${SETUP_LOG}"
    echo ""

    exit "${exit_code}"
}

trap 'on_error ${LINENO}' ERR

# ------------------------------------------------------------------------------
# 1. COMPROBACIONES INICIALES
# ------------------------------------------------------------------------------

log_info "[1/15] Comprobando sistema..."

if [[ "${EUID}" -ne 0 ]]; then
    die "Este script debe ejecutarse como root."
fi

if [[ ! -f /etc/arch-release ]]; then
    die "Este script solo funciona en Arch Linux."
fi

log_ok "Sistema Arch Linux detectado."

log_step "Kernel:"
uname -r

log_step "Arquitectura:"
uname -m

log_step "Espacio disponible en /:"
df -h /

# ------------------------------------------------------------------------------
# 2. ACTUALIZACION DEL SISTEMA
# ------------------------------------------------------------------------------

log_info "[2/15] Actualizando repositorios y sistema..."

pacman -Syu --noconfirm

log_ok "Sistema actualizado."

# ------------------------------------------------------------------------------
# 3. PAQUETES BASE
# ------------------------------------------------------------------------------

log_info "[3/15] Instalando herramientas base..."

BASE_PACKAGES=(
    base-devel
    git
    curl
    wget
    file
    unzip
    zip
    tar
    gzip
    xz
    pkgconf
    openssl
    ca-certificates
    nano
    sudo
    which
)

pacman -S --needed --noconfirm "${BASE_PACKAGES[@]}"

log_ok "Herramientas base instaladas."

# ------------------------------------------------------------------------------
# 4. CONFIGURACION REGIONAL
# ------------------------------------------------------------------------------

log_info "[4/15] Configurando sistema regional..."

if [[ -f /usr/share/zoneinfo/Europe/Madrid ]]; then
    ln -sf /usr/share/zoneinfo/Europe/Madrid /etc/localtime
    hwclock --systohc || true
fi

if [[ -f /etc/locale.gen ]]; then
    sed -i 's/^#es_ES.UTF-8 UTF-8/es_ES.UTF-8 UTF-8/' /etc/locale.gen
    sed -i 's/^#en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen
    locale-gen
fi

cat > /etc/locale.conf <<'EOF'
LANG=es_ES.UTF-8
LC_NUMERIC=C
EOF

if [[ ! -f /etc/hostname ]]; then
    echo "emubox" > /etc/hostname
fi

log_ok "Configuracion regional completada."

# ------------------------------------------------------------------------------
# 5. NETWORKMANAGER
# ------------------------------------------------------------------------------

log_info "[5/15] Configurando red..."

pacman -S --needed --noconfirm networkmanager

systemctl enable NetworkManager || true

log_ok "NetworkManager habilitado."

# ------------------------------------------------------------------------------
# 6. USUARIO Y SUDO
# ------------------------------------------------------------------------------

log_info "[6/15] Configurando usuario y sudo..."

if ! getent group "${EMUBOX_GROUP}" >/dev/null 2>&1; then
    groupadd "${EMUBOX_GROUP}"
fi

if id "${EMUBOX_USER}" >/dev/null 2>&1; then
    log_step "Usuario ${EMUBOX_USER} ya existe."
else
    log_step "Creando usuario ${EMUBOX_USER}..."
    useradd \
        -m \
        -g "${EMUBOX_GROUP}" \
        -G wheel \
        -s /bin/bash \
        "${EMUBOX_USER}"
    log_ok "Usuario ${EMUBOX_USER} creado."

    log_warn "El usuario ${EMUBOX_USER} necesita una contraseña."
    passwd "${EMUBOX_USER}"
fi

# Asegurar grupo wheel
usermod -aG wheel "${EMUBOX_USER}"

# Configurar sudoers de forma segura
SUDOERS_FILE="/etc/sudoers.d/emubox-wheel"

cat > "${SUDOERS_FILE}" <<'EOF'
%wheel ALL=(ALL:ALL) ALL
EOF

chmod 0440 "${SUDOERS_FILE}"

if ! visudo -cf "${SUDOERS_FILE}" >/dev/null; then
    rm -f "${SUDOERS_FILE}"
    die "La configuracion de sudoers no es valida."
fi

log_ok "sudo configurado correctamente."

# ------------------------------------------------------------------------------
# 7. TOOLCHAINS NODE / NPM / RUST
# ------------------------------------------------------------------------------

log_info "[7/15] Instalando y configurando Node.js, npm y Rust..."

pacman -S --needed --noconfirm \
    nodejs \
    npm

# Configuracion de Rust mediante rustup y toolchain estable
log_step "Configurando toolchain estable de Rust..."

if ! command -v rustup >/dev/null 2>&1; then
    log_step "rustup no esta instalado. Instalando rustup via pacman..."
    pacman -S --needed --noconfirm rustup || pacman -S --needed --noconfirm rust cargo
fi

export PATH="${HOME}/.cargo/bin:/usr/local/bin:${PATH}"

if [[ -f "${HOME}/.cargo/env" ]]; then
    # shellcheck source=/dev/null
    . "${HOME}/.cargo/env"
fi

if command -v rustup >/dev/null 2>&1; then
    if ! rustup toolchain list 2>/dev/null | grep -q '^stable'; then
        log_step "Instalando toolchain estable de Rust..."
        rustup toolchain install stable
    fi

    rustup default stable

    # Configurar también para el usuario de compilación si es distinto de root
    if [[ -n "${EMUBOX_USER:-}" && "${EMUBOX_USER}" != "root" ]] && id "${EMUBOX_USER}" >/dev/null 2>&1; then
        runuser -u "${EMUBOX_USER}" -- env PATH="/home/${EMUBOX_USER}/.cargo/bin:${PATH}" rustup default stable 2>/dev/null || true
    fi
fi

hash -r

command -v node >/dev/null 2>&1 || die "Node.js no esta disponible despues de la instalacion."
command -v npm >/dev/null 2>&1 || die "npm no esta disponible despues de la instalacion."
command -v rustc >/dev/null 2>&1 || die "rustc no esta disponible despues de configurar Rust."
command -v cargo >/dev/null 2>&1 || die "cargo no esta disponible despues de configurar Rust."

log_ok "Node.js: $(node --version)"
log_ok "npm: $(npm --version)"
log_ok "Rust: $(rustc --version)"
log_ok "Cargo: $(cargo --version)"
log_ok "Toolchain Rust estable configurado correctamente."

# ------------------------------------------------------------------------------
# 8. DEPENDENCIAS GRAFICAS TAURI
# ------------------------------------------------------------------------------

log_info "[8/15] Instalando dependencias graficas para Tauri..."

TAURI_PACKAGES=(
    gtk3
    webkit2gtk-4.1
    librsvg
    openssl
    pkgconf
    xdotool
)

pacman -S --needed --noconfirm "${TAURI_PACKAGES[@]}"

log_ok "Dependencias graficas de Tauri instaladas."

# ------------------------------------------------------------------------------
# 9. PREPARACION DEL REPOSITORIO
# ------------------------------------------------------------------------------

log_info "[9/15] Preparando EmuBox..."

mkdir -p /opt

if [[ -d "${EMUBOX_DIR}/.git" ]]; then
    log_step "Repositorio existente detectado."
    git -C "${EMUBOX_DIR}" fetch origin
    if git -C "${EMUBOX_DIR}" diff --quiet && \
       git -C "${EMUBOX_DIR}" diff --cached --quiet; then
        git -C "${EMUBOX_DIR}" pull --ff-only
    else
        log_warn "Hay modificaciones locales. No se sobrescribiran."
    fi
else
    if [[ -e "${EMUBOX_DIR}" ]]; then
        die "${EMUBOX_DIR} existe pero no es un repositorio Git."
    fi
    log_step "Clonando repositorio..."
    git clone "${REPO_URL}" "${EMUBOX_DIR}"
fi

chown -R "${EMUBOX_USER}:${EMUBOX_GROUP}" "${EMUBOX_DIR}"
chmod -R 775 "${LOG_DIR}"

log_ok "Repositorio EmuBox preparado."

# ------------------------------------------------------------------------------
# 10. BUILD DE EMUBOX
# ------------------------------------------------------------------------------

log_info "[10/15] Compilando EmuBox..."

cd "${EMUBOX_DIR}"

if [[ ! -f package.json ]]; then
    die "No existe package.json en ${EMUBOX_DIR}."
fi

if [[ ! -f "${EMUBOX_DIR}/scripts/build.sh" ]]; then
    die "No existe scripts/build.sh en el proyecto."
fi

chmod +x "${EMUBOX_DIR}/scripts/build.sh"

log_step "Ejecutando build.sh del proyecto..."
log_step "La salida npm se almacenara en los logs correspondientes."

# El build debe ejecutarse como usuario normal.
# Esto evita que node_modules y Cargo queden propiedad de root.

BUILD_SCRIPT="${EMUBOX_DIR}/scripts/build.sh"

if ! runuser -u "${EMUBOX_USER}" -- bash "${BUILD_SCRIPT}"; then
    log_error "El build de EmuBox ha fallado."
    log_error "Logs disponibles:"
    log_error "  npm:     ${NPM_INSTALL_LOG}"
    log_error "  esbuild: ${NPM_ESBUILD_LOG}"
    log_error "  frontend:${NPM_BUILD_LOG}"
    log_error "  cargo:   ${CARGO_BUILD_LOG}"
    exit 1
fi

log_ok "Build de EmuBox completado."

# ------------------------------------------------------------------------------
# 11. ESTRUCTURA DE DATOS
# ------------------------------------------------------------------------------

log_info "[11/15] Creando estructura de datos..."

mkdir -p \
    /var/lib/emubox/games \
    /var/lib/emubox/roms \
    /var/lib/emubox/saves \
    /var/lib/emubox/states \
    /var/lib/emubox/bios \
    /var/lib/emubox/covers \
    /var/lib/emubox/logs \
    /var/lib/emubox/screenshots

USER_HOME="$(getent passwd "${EMUBOX_USER}" | cut -d: -f6)"

mkdir -p \
    "${USER_HOME}/.local/share/emubox/roms" \
    "${USER_HOME}/.local/share/emubox/saves" \
    "${USER_HOME}/.local/share/emubox/states" \
    "${USER_HOME}/.local/share/emubox/bios" \
    "${USER_HOME}/.local/share/emubox/covers" \
    "${USER_HOME}/.local/share/emubox/logs" \
    "${USER_HOME}/.local/share/emubox/screenshots" \
    "${USER_HOME}/.config/emubox" \
    "${USER_HOME}/.cache/emubox"

chown -R "${EMUBOX_USER}:${EMUBOX_GROUP}" \
    /var/lib/emubox \
    "${USER_HOME}/.local/share/emubox" \
    "${USER_HOME}/.config/emubox" \
    "${USER_HOME}/.cache/emubox"

log_ok "Estructura de datos creada."

# ------------------------------------------------------------------------------
# 12. COMANDO GLOBAL EMUBOX
# ------------------------------------------------------------------------------

log_info "[12/15] Registrando comando global emubox..."

cat > /usr/local/bin/emubox <<'EOF'
#!/usr/bin/env bash

set -euo pipefail

EMUBOX_BIN="/opt/emubox/bin/emubox"

if [[ ! -x "${EMUBOX_BIN}" ]]; then
    echo "[ERROR] No existe el binario de EmuBox:"
    echo "${EMUBOX_BIN}"
    exit 1
fi

exec "${EMUBOX_BIN}" "$@"
EOF

chmod 0755 /usr/local/bin/emubox
ln -sf /usr/local/bin/emubox /usr/bin/emubox

log_ok "Comando emubox instalado."

# ------------------------------------------------------------------------------
# 13. COMANDO UPDATE
# ------------------------------------------------------------------------------

log_info "[13/15] Registrando emubox-update..."

cat > /usr/local/bin/emubox-update <<'EOF'
#!/usr/bin/env bash

set -Eeuo pipefail

EMUBOX_DIR="/opt/emubox"
EMUBOX_USER="emubox"

cd "${EMUBOX_DIR}"

echo "[EmuBox Update] Actualizando repositorio..."

git fetch origin
git pull --ff-only

echo "[EmuBox Update] Ejecutando build..."

chown -R "${EMUBOX_USER}:${EMUBOX_USER}" "${EMUBOX_DIR}"

runuser -u "${EMUBOX_USER}" -- \
    bash "${EMUBOX_DIR}/scripts/build.sh"

echo "[OK] EmuBox actualizado correctamente."
EOF

chmod 0755 /usr/local/bin/emubox-update
ln -sf /usr/local/bin/emubox-update /usr/bin/emubox-update

log_ok "emubox-update instalado."

# ------------------------------------------------------------------------------
# 14. SYSTEMD
# ------------------------------------------------------------------------------

log_info "[14/15] Configurando servicio EmuBox..."

cat > /etc/systemd/system/emubox.service <<EOF
[Unit]
Description=EmuBox Console Interface
After=graphical.target
Wants=graphical.target

[Service]
Type=simple

User=${EMUBOX_USER}
Group=${EMUBOX_GROUP}

WorkingDirectory=${EMUBOX_DIR}

ExecStart=/usr/bin/emubox

Restart=on-failure
RestartSec=3

Environment=HOME=${USER_HOME}
Environment=EMUBOX_HOME=/var/lib/emubox
Environment=NODE_ENV=production

StandardOutput=append:/var/log/emubox/emubox.log
StandardError=append:/var/log/emubox/emubox-error.log

[Install]
WantedBy=graphical.target
EOF

systemctl daemon-reload
systemctl enable emubox.service || true

log_ok "Servicio emubox.service configurado."

# ------------------------------------------------------------------------------
# 15. VERIFICACION FINAL
# ------------------------------------------------------------------------------

log_info "[15/15] Verificacion final..."

echo ""
echo "=============================================================="
echo "             EMUBOX - INSTALACION COMPLETADA"
echo "=============================================================="
echo ""
echo "Usuario:              ${EMUBOX_USER}"
echo "Proyecto:             ${EMUBOX_DIR}"
echo "Binario:              ${EMUBOX_DIR}/bin/emubox"
echo "Comando:              emubox"
echo "Actualizacion:        emubox-update"
echo "Datos:                /var/lib/emubox"
echo "Datos usuario:        ${USER_HOME}/.local/share/emubox"
echo "Configuracion:        ${USER_HOME}/.config/emubox"
echo "Logs:                 ${LOG_DIR}"
echo "Servicio:             emubox.service"
echo ""
echo "Node:                 $(node --version)"
echo "npm:                  $(npm --version)"
echo "Rust:                 $(rustc --version)"
echo "Cargo:                $(cargo --version)"
echo ""
echo "=============================================================="

if [[ -x "${EMUBOX_DIR}/bin/emubox" ]]; then
    log_ok "Binario EmuBox verificado."
else
    log_error "No se encontro el binario EmuBox."
    exit 1
fi

log_ok "INSTALACION FINALIZADA CORRECTAMENTE."
