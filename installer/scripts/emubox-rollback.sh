#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX OS - ATOMIC ROLLBACK SCRIPT
#  Reverts /opt/emubox/current to a previous release safely.
# ==============================================================================

set -euo pipefail

CLR_RESET="\033[0m"
CLR_BOLD="\033[1m"
CLR_CYAN="\033[36m"
CLR_GREEN="\033[32m"
CLR_YELLOW="\033[33m"

OPT_DIR="/opt/emubox"
RELEASES_DIR="${OPT_DIR}/releases"
CURRENT_LINK="${OPT_DIR}/current"

echo -e "${CLR_CYAN}${CLR_BOLD}==> Gestor de Rollback de EmuBox...${CLR_RESET}"

TARGET_VERSION="${1:-}"

if [[ -z "${TARGET_VERSION}" ]]; then
  echo "Versiones instaladas disponibles:"
  ls -1 "${RELEASES_DIR}" 2>/dev/null || echo "Ninguna versión previa."
  echo ""
  read -p "Introduce la versión a la que deseas revertir (ej. v1.0.0): " -r TARGET_VERSION
fi

TARGET_PATH="${RELEASES_DIR}/${TARGET_VERSION}"

if [[ ! -d "${TARGET_PATH}" ]]; then
  echo -e "${CLR_YELLOW}Error: La versión ${TARGET_VERSION} no existe en ${RELEASES_DIR}.${CLR_RESET}"
  exit 1
fi

echo -e "  -> Reasignando enlace atómico ${CURRENT_LINK} -> ${TARGET_PATH}..."
ln -sfn "${TARGET_PATH}" "${CURRENT_LINK}"

if systemctl --user is-active --quiet emubox.service 2>/dev/null; then
  systemctl --user restart emubox.service || true
fi

echo -e "${CLR_GREEN}${CLR_BOLD}✓ Rollback completado a la versión ${TARGET_VERSION}.${CLR_RESET}"
