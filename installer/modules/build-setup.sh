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
TAURI_BINARY="${ROOT_DIR}/bin/emubox"
OPT_BIN="${OPT_BIN_DIR}/emubox"
source "$ROOT_DIR/installer/lib/architecture.sh"
validate_emubox_binary "$TAURI_BINARY"

if [[ "${TAURI_BINARY}" != "${OPT_BIN}" ]]; then
  cp -f "${TAURI_BINARY}" "${OPT_BIN}"
  chmod +x "${OPT_BIN}"
fi

# 6. Instalar lanzador autónomo /usr/local/bin/emubox-launcher
cat << 'EOF' > /usr/local/bin/emubox-launcher
#!/usr/bin/env bash
set -euo pipefail

exec bash /opt/emubox/scripts/run.sh "$@"
EOF
chmod 0755 /usr/local/bin/emubox-launcher
ln -sf /usr/local/bin/emubox-launcher /usr/bin/emubox-launcher

# 7. Instalar ejecutable global en /usr/local/bin/emubox (ESTRICTO CON SOPORTE WAYLAND/CAGE/GAMESCOPE)
cat << 'EOF' > /usr/local/bin/emubox
#!/usr/bin/env bash
set -euo pipefail

exec bash /opt/emubox/scripts/run.sh "$@"
EOF
chmod +x /usr/local/bin/emubox
ln -sf /usr/local/bin/emubox /usr/bin/emubox

# 7. Instalar comando de actualización global /usr/local/bin/emubox-update
EMUBOX_APP_DIR="/opt/emubox"
chmod +x "${EMUBOX_APP_DIR}/scripts/update-emubox.sh"
ln -sf "${EMUBOX_APP_DIR}/scripts/update-emubox.sh" /usr/local/bin/emubox-update
ln -sf /usr/local/bin/emubox-update /usr/bin/emubox-update

log_ok "Binario nativo y comandos 'emubox' y 'emubox-update' registrados para producción."

