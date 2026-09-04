#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX OS - MASTER INSTALLER & BOOTSTRAP ORCHESTRATOR FOR ARCH LINUX
#  Idempotent, Safe, Modular and Compliant with the appliance filesystem layout
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/logging.sh
. "${SCRIPT_DIR}/lib/logging.sh"
# shellcheck source=lib/paths.sh
. "${SCRIPT_DIR}/lib/paths.sh"

main() {
  log_banner

  # 1. System pre-flight & hardware detection
  bash "${SCRIPT_DIR}/modules/system-setup.sh"

  # 2. XDG Directory Structure setup
  bash "${SCRIPT_DIR}/modules/directory-setup.sh"

  # 3. System packages & dependencies
  bash "${SCRIPT_DIR}/modules/package-setup.sh"

  # 4. Permissions & udev rules
  bash "${SCRIPT_DIR}/modules/input-setup.sh"

  # 5. Emulators registration & discovery
  bash "${SCRIPT_DIR}/modules/emulator-setup.sh"

  # 6. Gamescope compositor configuration
  bash "${SCRIPT_DIR}/modules/gamescope-setup.sh"

  # 7. Native Production Build (SolidJS + Tauri release binary)
  bash "${SCRIPT_DIR}/modules/build-setup.sh"

  # 8. Desktop integration & autostart
  bash "${SCRIPT_DIR}/modules/autostart-setup.sh"

  echo ""
  echo -e "${CLR_GREEN}${CLR_BOLD}===============================================================================${CLR_RESET}"
  echo -e "${CLR_GREEN}${CLR_BOLD}  ¡INSTALACIÓN DE EMUBOX COMPLETADA CON ÉXITO!                                 ${CLR_RESET}"
  echo -e "${CLR_GREEN}${CLR_BOLD}===============================================================================${CLR_RESET}"
  echo -e "  Directorio de ROMs:    ${CLR_CYAN}${EMUBOX_DATA_DIR}/roms${CLR_RESET}"
  echo -e "  Fichero Configuración: ${CLR_CYAN}${EMUBOX_CONFIG_FILE}${CLR_RESET}"
  echo -e "  Para iniciar EmuBox:   ${CLR_BOLD}emubox${CLR_RESET}"
  echo -e "${CLR_GREEN}===============================================================================${CLR_RESET}\n"
}

main "$@"
