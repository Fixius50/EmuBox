#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - SCRIPT CENTRALIZADO DE COMPILACION Y EMPAQUETADO (SINGLE SOURCE OF TRUTH)
# ==============================================================================

set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EMUBOX_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
source "${EMUBOX_DIR}/installer/lib/architecture.sh"

if [[ "$EUID" -eq 0 ]]; then
  BUILD_USER="${SUDO_USER:-emubox}"
  [[ "$BUILD_USER" != root ]] || { echo 'Build must run as a non-root user.' >&2; exit 1; }
  exec runuser -u "$BUILD_USER" -- bash "$0" "$@"
fi

BUILD_ARCH="${BUILD_ARCH:-$(get_emubox_architecture)}"
[[ "$BUILD_ARCH" == "$(get_emubox_architecture)" && "$BUILD_ARCH" != unsupported ]] || {
  echo 'Native builds only: BUILD_ARCH must match this machine.' >&2
  exit 1
}
TARGET="${TARGET:-$(get_emubox_target "$BUILD_ARCH")}"
[[ "$TARGET" == "$(get_emubox_target "$BUILD_ARCH")" ]] || {
  echo 'TARGET must be the native Linux GNU target.' >&2
  exit 1
}

log_info() { echo ""; echo "[EmuBox Build] $1"; }
log_step() { echo "  -> $1"; }
log_ok() { echo "[OK] $1"; }
log_warn() { echo "[AVISO] $1"; }
log_error() { echo "[ERROR] $1" >&2; }

# Directorio de logs resiliente (permite ejecutar como usuario no-root)
LOG_DIR="${EMUBOX_LOG_DIR:-/var/log/emubox}"
if ! mkdir -p "${LOG_DIR}" 2>/dev/null || [[ ! -w "${LOG_DIR}" ]]; then
  LOG_DIR=$(mktemp -d "${TMPDIR:-/tmp}/emubox-build-XXXXXXXX")
fi
for LOG_NAME in npm-install.log npm-build.log cargo-build.log; do
  if [[ -e "$LOG_DIR/$LOG_NAME" && ! -w "$LOG_DIR/$LOG_NAME" ]]; then
    LOG_DIR=$(mktemp -d "${TMPDIR:-/tmp}/emubox-build-XXXXXXXX")
    break
  fi
done
log_step "Logs: $LOG_DIR"

cd "${EMUBOX_DIR}"

log_info "Iniciando proceso de compilacion de EmuBox..."
log_step "BUILD_ARCH=${BUILD_ARCH} TARGET=${TARGET}"

# ------------------------------------------------------------------------------
# 1. Diagnostico del entorno Node.js
# ------------------------------------------------------------------------------
log_step "Verificando entorno Node.js y npm..."
if ! command -v node >/dev/null 2>&1 || ! command -v npm >/dev/null 2>&1; then
  log_error "Node.js o npm no estan instalados en el sistema."
  exit 1
fi

log_step "Node: $(node --version) | npm: $(npm --version)"

# ------------------------------------------------------------------------------
# 2. Instalacion de dependencias (npm ci / npm install)
# ------------------------------------------------------------------------------
log_step "Validando dependencias npm y cache..."
NPM_LOG="${LOG_DIR}/npm-install.log"
if ! node scripts/build-cache.mjs dependencies >"${NPM_LOG}" 2>&1; then
  tail -n 30 "${NPM_LOG}"
  exit 1
fi

log_ok "Dependencias npm preparadas."

# ------------------------------------------------------------------------------
# 4. Compilacion del frontend SolidJS
# ------------------------------------------------------------------------------
log_step "Compilando frontend SolidJS..."
BUILD_LOG="${LOG_DIR}/npm-build.log"
: > "${BUILD_LOG}"

if node scripts/build-cache.mjs frontend >"${BUILD_LOG}" 2>&1; then
  log_ok "Frontend SolidJS compilado correctamente."
else
  BUILD_STATUS=$?
  log_error "La compilacion del frontend ha fallado (codigo ${BUILD_STATUS})."
  log_error "Log completo: ${BUILD_LOG}"
  echo ""
  echo "Ultimas 40 lineas del error:"
  tail -n 40 "${BUILD_LOG}"
  exit "${BUILD_STATUS}"
fi

# ------------------------------------------------------------------------------
# 5. Compilacion del binario nativo Tauri (Release)
# ------------------------------------------------------------------------------
export PATH="${HOME}/.cargo/bin:/usr/local/bin:${PATH}"
if [[ -f "${HOME}/.cargo/env" ]]; then
  # shellcheck source=/dev/null
  . "${HOME}/.cargo/env"
fi

if ! command -v rustc >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
  log_error "El compilador Rust (rustc) o Cargo no estan disponibles en el PATH."
  exit 1
fi

# Asegurar iconos de la aplicacion para Tauri
if [[ ! -f "${EMUBOX_DIR}/src-tauri/icons/icon.png" ]] && [[ -f "${EMUBOX_DIR}/scripts/generate-icons.js" ]]; then
  log_step "Generando iconos nativos para Tauri..."
  node "${EMUBOX_DIR}/scripts/generate-icons.js" >/dev/null 2>&1 || true
fi

# Verificar que el frontend estatico existe antes de compilar Tauri
if [[ ! -f "${EMUBOX_DIR}/solid/dist/index.html" ]]; then
  log_error "No se encontro ${EMUBOX_DIR}/solid/dist/index.html. La compilacion del frontend es requerida antes de Tauri."
  exit 1
fi

mkdir -p "${EMUBOX_DIR}/bin"
TARGET_DIR=$(cargo metadata --manifest-path "${EMUBOX_DIR}/src-tauri/Cargo.toml" --no-deps --format-version 1 | node -e 'let data="";process.stdin.on("data",chunk=>data+=chunk);process.stdin.on("end",()=>console.log(JSON.parse(data).target_directory))')
TAURI_BINARY="${TARGET_DIR}/${TARGET}/release/emubox"
CARGO_LOG="${LOG_DIR}/cargo-build.log"
: > "${CARGO_LOG}"

log_step "Compilando EmuBox Tauri en modo produccion con frontend embebido..."

if npx tauri build --no-bundle --target "$TARGET" --config '{"build":{"beforeBuildCommand":""}}' >"${CARGO_LOG}" 2>&1; then
  log_ok "Binario nativo Tauri compilado exitosamente con frontend embebido (tauri build)."
elif cargo build --release --target "$TARGET" --manifest-path "${EMUBOX_DIR}/src-tauri/Cargo.toml" >>"${CARGO_LOG}" 2>&1; then
  log_ok "Binario nativo Tauri compilado exitosamente mediante Cargo release."
else
  CARGO_STATUS=$?
  log_error "La compilacion de Tauri ha fallado (codigo ${CARGO_STATUS})."
  log_error "Log completo: ${CARGO_LOG}"
  echo ""
  echo "Ultimas 40 lineas del error:"
  tail -n 40 "${CARGO_LOG}"
  exit "${CARGO_STATUS}"
fi

# Copiar binario a ubicacion estable
validate_emubox_binary "$TAURI_BINARY" "$BUILD_ARCH"
install -m 0755 "$TAURI_BINARY" "${EMUBOX_DIR}/bin/emubox-linux-${BUILD_ARCH}"
install -m 0755 "$TAURI_BINARY" "${EMUBOX_DIR}/bin/emubox.next"
mv -f "${EMUBOX_DIR}/bin/emubox.next" "${EMUBOX_DIR}/bin/emubox"
log_ok "Binario instalado en ${EMUBOX_DIR}/bin/emubox."
