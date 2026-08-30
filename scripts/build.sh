#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - SCRIPT CENTRALIZADO DE COMPILACION Y EMPAQUETADO (SINGLE SOURCE OF TRUTH)
# ==============================================================================

set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EMUBOX_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

log_info() { echo ""; echo "[EmuBox Build] $1"; }
log_step() { echo "  -> $1"; }
log_ok() { echo "[OK] $1"; }
log_warn() { echo "[AVISO] $1"; }
log_error() { echo "[ERROR] $1" >&2; }

# Directorio de logs resiliente (permite ejecutar como usuario no-root)
LOG_DIR="${EMUBOX_LOG_DIR:-/var/log/emubox}"
if [[ ! -d "${LOG_DIR}" ]] || [[ ! -w "${LOG_DIR}" ]]; then
  mkdir -p "${LOG_DIR}" 2>/dev/null || LOG_DIR="/tmp/emubox-build-logs"
  mkdir -p "${LOG_DIR}" 2>/dev/null || true
fi

cd "${EMUBOX_DIR}"

# Asegurar permisos de usuario sobre el árbol de trabajo (evita colisiones si un build previo se corrió como root)
if command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
  sudo chown -R "$(id -u):$(id -g)" "${EMUBOX_DIR}" 2>/dev/null || true
fi

log_info "Iniciando proceso de compilacion de EmuBox..."

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
log_step "Instalando dependencias npm..."
NPM_LOG="${LOG_DIR}/npm-install.log"
: > "${NPM_LOG}"

if [[ -f package-lock.json ]]; then
  log_step "package-lock.json detectado. Ejecutando npm ci..."
  if npm ci --no-audit --no-fund >"${NPM_LOG}" 2>&1; then
    log_ok "npm ci completado correctamente."
  else
    NPM_STATUS=$?
    log_warn "npm ci ha fallado (codigo ${NPM_STATUS}). Intentando npm install..."
    if npm install --no-audit --no-fund >>"${NPM_LOG}" 2>&1; then
      log_ok "npm install alternativo completado correctamente."
    else
      INSTALL_STATUS=$?
      log_error "La instalacion de dependencias npm ha fallado."
      log_error "Codigo de salida: ${INSTALL_STATUS}"
      log_error "Consulta el log completo: ${NPM_LOG}"
      echo ""
      echo "Ultimas 30 lineas del error:"
      tail -n 30 "${NPM_LOG}"
      exit "${INSTALL_STATUS}"
    fi
  fi
else
  log_step "No existe package-lock.json. Ejecutando npm install..."
  if npm install --no-audit --no-fund >"${NPM_LOG}" 2>&1; then
    log_ok "npm install completado correctamente."
  else
    INSTALL_STATUS=$?
    log_error "npm install ha fallado (codigo ${INSTALL_STATUS})."
    echo "Ultimas 30 lineas del error:"
    tail -n 30 "${NPM_LOG}"
    exit "${INSTALL_STATUS}"
  fi
fi

# ------------------------------------------------------------------------------
# 3. Preparacion de esbuild
# ------------------------------------------------------------------------------
log_step "Preparando binarios nativos de esbuild..."
ESBUILD_LOG="${LOG_DIR}/npm-esbuild.log"
: > "${ESBUILD_LOG}"

if npm rebuild esbuild >>"${ESBUILD_LOG}" 2>&1; then
  log_ok "esbuild preparado correctamente."
else
  ESBUILD_STATUS=$?
  log_warn "npm rebuild esbuild ha reportado advertencias (log: ${ESBUILD_LOG})."
fi

log_ok "Dependencias npm preparadas."

# ------------------------------------------------------------------------------
# 4. Compilacion del frontend SolidJS
# ------------------------------------------------------------------------------
log_step "Compilando frontend SolidJS..."
if ! rm -rf solid/dist 2>/dev/null; then
  if command -v sudo >/dev/null 2>&1; then
    sudo rm -rf solid/dist 2>/dev/null || true
  fi
fi

BUILD_LOG="${LOG_DIR}/npm-build.log"
: > "${BUILD_LOG}"

if npm run build >"${BUILD_LOG}" 2>&1; then
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

if command -v rustup >/dev/null 2>&1; then
  rustup default stable >/dev/null 2>&1 || true
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
TAURI_BINARY="${EMUBOX_DIR}/src-tauri/target/release/emubox"
CARGO_LOG="${LOG_DIR}/cargo-build.log"
: > "${CARGO_LOG}"

log_step "Compilando EmuBox Tauri en modo produccion con frontend embebido..."

# Limpiar cache del crate emubox para forzar re-empaquetado limpio de ../solid/dist
cargo clean --manifest-path "${EMUBOX_DIR}/src-tauri/Cargo.toml" -p emubox >/dev/null 2>&1 || true

if npx tauri build --no-bundle >"${CARGO_LOG}" 2>&1; then
  log_ok "Binario nativo Tauri compilado exitosamente con frontend embebido (tauri build)."
elif cargo build --release --manifest-path "${EMUBOX_DIR}/src-tauri/Cargo.toml" >"${CARGO_LOG}" 2>&1; then
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
cp -f "${TAURI_BINARY}" "${EMUBOX_DIR}/bin/emubox"
chmod +x "${EMUBOX_DIR}/bin/emubox"
log_ok "Binario instalado en ${EMUBOX_DIR}/bin/emubox."
