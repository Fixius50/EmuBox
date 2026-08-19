#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX MODULE - NATIVE TAURI PRODUCTION BUILD & BINARY REGISTRATION
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# shellcheck source=../lib/logging.sh
if [[ -f "${SCRIPT_DIR}/lib/logging.sh" ]]; then
  . "${SCRIPT_DIR}/lib/logging.sh"
else
  log_step() { echo "--> $*"; }
  log_ok() { echo "[OK] $*"; }
  log_error() { echo "[ERROR] $*" >&2; }
fi

cd "${ROOT_DIR}"

# 1. Verificacion de Node.js y npm
if ! command -v node >/dev/null 2>&1 || ! command -v npm >/dev/null 2>&1; then
  log_error "Node.js o npm no estan disponibles en el sistema."
  exit 1
fi

log_step "Node.js: $(node --version)"
log_step "npm: $(npm --version)"

# 2. Dependencias npm - SALIDA OCULTA
log_step "Instalando dependencias npm..."

NPM_LOG="/var/log/emubox/npm-install.log"

if [[ -f package-lock.json ]]; then
  if npm ci --no-audit --no-fund >"${NPM_LOG}" 2>&1; then
    log_ok "Dependencias npm instaladas correctamente."
  else
    NPM_STATUS=$?
    log_error "La instalacion de dependencias npm ha fallado."
    log_error "Codigo de salida: ${NPM_STATUS}"
    log_error "El detalle completo se ha guardado en: ${NPM_LOG}"
    exit "${NPM_STATUS}"
  fi
else
  if npm install --no-audit --no-fund >"${NPM_LOG}" 2>&1; then
    log_ok "Dependencias npm instaladas correctamente."
  else
    NPM_STATUS=$?
    log_error "La instalacion de dependencias npm ha fallado."
    log_error "Codigo de salida: ${NPM_STATUS}"
    log_error "El detalle completo se ha guardado en: ${NPM_LOG}"
    exit "${NPM_STATUS}"
  fi
fi

# 3. Compilacion del Frontend SolidJS - SALIDA OCULTA
log_step "Compilando frontend SolidJS..."
rm -rf solid/dist

BUILD_LOG="/var/log/emubox/npm-build.log"

if npm run build >"${BUILD_LOG}" 2>&1; then
  log_ok "Frontend SolidJS compilado correctamente."
else
  BUILD_STATUS=$?
  log_error "La compilacion del frontend ha fallado."
  log_error "Codigo de salida: ${BUILD_STATUS}"
  log_error "El detalle completo se ha guardado en: ${BUILD_LOG}"
  exit "${BUILD_STATUS}"
fi

# 4. Compilacion de Tauri (Release)
OPT_BIN_DIR="/opt/emubox/bin"
mkdir -p "${OPT_BIN_DIR}"
TAURI_BINARY="${ROOT_DIR}/src-tauri/target/release/emubox"

if [[ -x "${TAURI_BINARY}" ]]; then
  log_ok "Binario Tauri existente detectado."
else
  log_step "Compilando EmuBox Tauri en modo release..."
  cargo build --release --manifest-path "${ROOT_DIR}/src-tauri/Cargo.toml"

  if [[ ! -x "${TAURI_BINARY}" ]]; then
    log_error "Cargo termino pero no se encontro el binario: ${TAURI_BINARY}"
    exit 1
  fi
  log_ok "Binario Tauri compilado correctamente."
fi

# 5. Copiar binario a ubicacion estable
cp -f "${TAURI_BINARY}" "${OPT_BIN_DIR}/emubox"
chmod +x "${OPT_BIN_DIR}/emubox"
log_ok "Binario instalado en ${OPT_BIN_DIR}/emubox."

# 6. Instalar ejecutable global en /usr/local/bin/emubox (ESTRICTO SIN FALLBACK)
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

# 7. Instalar comando de actualizacion global /usr/local/bin/emubox-update
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

log_ok "Binario nativo y comandos 'emubox' y 'emubox-update' registrados para producción."
