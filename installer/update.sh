#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX OS - IDEMPOTENT UPDATER SCRIPT
#  Updates binary, assets and dependencies without altering user data or saves
# ==============================================================================

set -euo pipefail

CLR_RESET="\033[0m"
CLR_BOLD="\033[1m"
CLR_CYAN="\033[36m"
CLR_GREEN="\033[32m"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${CLR_CYAN}${CLR_BOLD}==> Actualizando EmuBox a la última versión...${CLR_RESET}"

# Re-run setup orchestrators (idempotent)
bash "${SCRIPT_DIR}/setup/dependencies.sh"
bash "${SCRIPT_DIR}/setup/directories.sh"
bash "${SCRIPT_DIR}/setup/permissions.sh"
bash "${SCRIPT_DIR}/setup/emulators.sh"
bash "${SCRIPT_DIR}/setup/desktop.sh"

echo -e "${CLR_GREEN}${CLR_BOLD}✓ EmuBox actualizado correctamente sin modificar tus ROMs ni partidas.${CLR_RESET}"
