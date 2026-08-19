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

# 1. Compilación mediante el script centralizado build.sh
chmod +x "${ROOT_DIR}/scripts/build.sh"
bash "${ROOT_DIR}/scripts/build.sh"

OPT_BIN_DIR="/opt/emubox/bin"
mkdir -p "${OPT_BIN_DIR}"
TAURI_BINARY="${ROOT_DIR}/src-tauri/target/release/emubox"
OPT_BIN="${OPT_BIN_DIR}/emubox"

if [[ -f "${TAURI_BINARY}" ]]; then
  cp -f "${TAURI_BINARY}" "${OPT_BIN}"
  chmod +x "${OPT_BIN}"
fi

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

chmod +x "${EMUBOX_APP_DIR}/scripts/build.sh"
bash "${EMUBOX_APP_DIR}/scripts/build.sh"

echo "[OK] EmuBox actualizado correctamente en modo producción."
EOF
chmod +x /usr/local/bin/emubox-update
ln -sf /usr/local/bin/emubox-update /usr/bin/emubox-update

log_ok "Binario nativo y comandos 'emubox' y 'emubox-update' registrados para producción."
