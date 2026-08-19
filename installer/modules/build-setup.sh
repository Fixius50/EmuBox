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
fi

log_step "Compilando frontend SolidJS y empaquetando en binario Tauri (Release)..."

log_step "Instalando dependencias de Node.js..."
npm install --no-audit --no-fund

log_step "Generando bundle de producción (SolidJS)..."
npm run build

log_step "Compilando binario nativo Tauri con Cargo (Modo Producción Release)..."
cargo build --release --manifest-path src-tauri/Cargo.toml

# Desplegar binario compilado
TARGET_BIN="${ROOT_DIR}/src-tauri/target/release/emubox"
OPT_BIN_DIR="/opt/emubox/bin"
OPT_BIN="${OPT_BIN_DIR}/emubox"

mkdir -p "${OPT_BIN_DIR}"
if [[ -f "${TARGET_BIN}" ]]; then
  cp "${TARGET_BIN}" "${OPT_BIN}"
  chmod +x "${OPT_BIN}"
fi

# Instalar ejecutable global en /usr/local/bin/emubox
cat << 'EOF' > /usr/local/bin/emubox
#!/usr/bin/env bash
set -euo pipefail

EMUBOX_BIN="/opt/emubox/bin/emubox"
if [[ -x "${EMUBOX_BIN}" ]]; then
  exec "${EMUBOX_BIN}" "$@"
elif [[ -x "/opt/emubox/src-tauri/target/release/emubox" ]]; then
  exec "/opt/emubox/src-tauri/target/release/emubox" "$@"
else
  echo "[ERROR] El binario de producción de EmuBox no se encuentra compilado." >&2
  echo "Ejecute 'emubox-update' para compilar el binario nativo." >&2
  exit 1
fi
EOF
chmod +x /usr/local/bin/emubox
ln -sf /usr/local/bin/emubox /usr/bin/emubox

# Instalar comando de actualización global /usr/local/bin/emubox-update
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
cargo build --release --manifest-path src-tauri/Cargo.toml

mkdir -p "${EMUBOX_APP_DIR}/bin"
cp "${EMUBOX_APP_DIR}/src-tauri/target/release/emubox" "${EMUBOX_APP_DIR}/bin/emubox"
chmod +x "${EMUBOX_APP_DIR}/bin/emubox"

echo "[OK] EmuBox compilado y actualizado en modo producción."
EOF
chmod +x /usr/local/bin/emubox-update
ln -sf /usr/local/bin/emubox-update /usr/bin/emubox-update

log_ok "Binario nativo y comandos 'emubox' y 'emubox-update' registrados para producción."
