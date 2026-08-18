#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX OS - MASTER INSTALLER & BOOTSTRAP ORCHESTRATOR FOR ARCH LINUX
#  Idempotent, Safe, Modular and Compliant with XDG Base Directory Specification
# ==============================================================================

set -euo pipefail

# Color Palette for Console Output
CLR_RESET="\033[0m"
CLR_BOLD="\033[1m"
CLR_CYAN="\033[36m"
CLR_GREEN="\033[32m"
CLR_YELLOW="\033[33m"
CLR_RED="\033[31m"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SETUP_DIR="${SCRIPT_DIR}/setup"

log_banner() {
  echo -e "${CLR_CYAN}${CLR_BOLD}"
  echo "  ███████╗███╗   ███╗██╗   ██╗██████╗  ██████╗ ██╗  ██╗"
  echo "  ██╔════╝████╗ ████║██║   ██║██╔══██╗██╔═══██╗╚██╗██╔╝"
  echo "  █████╗  ██╔████╔██║██║   ██║██████╔╝██║   ██║ ╚███╔╝ "
  echo "  ██╔══╝  ██║╚██╔╝██║██║   ██║██╔══██╗██║   ██║ ██╔██╗ "
  echo "  ███████╗██║ ╚═╝ ██║╚██████╔╝██████╔╝╚██████╔╝██╔╝ ██╗"
  echo "  ╚══════╝╚═╝     ╚═╝ ╚═════╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝"
  echo "         Dedicated Console Frontend for Arch Linux       "
  echo -e "${CLR_RESET}"
}

log_step() {
  echo -e "${CLR_CYAN}==>${CLR_BOLD} $1${CLR_RESET}"
}

log_ok() {
  echo -e "  ${CLR_GREEN}✓${CLR_RESET} $1"
}

log_warn() {
  echo -e "  ${CLR_YELLOW}⚠${CLR_RESET} $1"
}

log_error() {
  echo -e "  ${CLR_RED}✗ ERROR:${CLR_RESET} $1" >&2
}

# ------------------------------------------------------------------------------
# 1. Pre-Flight System & Environment Checks
# ------------------------------------------------------------------------------
check_system_compatibility() {
  log_step "Comprobando compatibilidad de la distribución y arquitectura..."

  # Architecture verification (x86_64 required for 6th gen emulators)
  ARCH=$(uname -m)
  if [[ "${ARCH}" != "x86_64" ]]; then
    log_error "EmuBox requiere arquitectura x86_64 (detectado: ${ARCH})."
    exit 1
  fi
  log_ok "Arquitectura compatible: ${ARCH}"

  # Distribution verification (Arch Linux or Arch-based)
  if [[ -f /etc/os-release ]]; then
    . /etc/os-release
    if [[ "${ID:-}" != "arch" && "${ID_LIKE:-}" != *"arch"* ]]; then
      log_warn "Distribución detectada (${NAME:-Desconocida}) no es Arch Linux nativo."
      log_warn "Continuando en modo permisivo..."
    else
      log_ok "Distribución validada: ${NAME}"
    fi
  else
    log_warn "No se pudo leer /etc/os-release. Asumiendo entorno compatible..."
  fi

  # Privilege check (Installer must NOT run directly as root)
  if [[ "${EUID}" -eq 0 ]]; then
    log_error "El instalador NO debe ejecutarse directamente como root. Ejecútalo como tu usuario normal con permisos sudo."
    exit 1
  fi
  log_ok "Usuario de ejecución no-root: $(whoami)"
}

# ------------------------------------------------------------------------------
# 2. Main Execution Pipeline
# ------------------------------------------------------------------------------
main() {
  log_banner
  check_system_compatibility

  # Step 1: Dependencies & Package Management
  log_step "[1/6] Verificando e instalando dependencias de sistema y emuladores..."
  bash "${SETUP_DIR}/dependencies.sh"

  # Step 2: XDG Directory Structure
  log_step "[2/6] Creando estructura de directorios conforme a XDG Base Directory..."
  bash "${SETUP_DIR}/directories.sh"

  # Step 3: Hardware, udev and Audio Permissions
  log_step "[3/6] Configurando permisos de hardware, reglas udev y audio en tiempo real..."
  bash "${SETUP_DIR}/permissions.sh"

  # Step 4: Emulators & Libretro Cores Registration
  log_step "[4/6] Registrando y validando binarios de emuladores instalados..."
  bash "${SETUP_DIR}/emulators.sh"

  # Step 5: Desktop Integration & Systemd Console Session
  log_step "[5/6] Instalando accesos de escritorio y servicio de sesión de consola..."
  bash "${SETUP_DIR}/desktop.sh"

  # Step 6: First-Run Bootstrap & Config Generation
  log_step "[6/6] Ejecutando detección de primer arranque (First Run Bootstrap)..."
  bash "${SETUP_DIR}/first-run.sh"

  echo ""
  echo -e "${CLR_GREEN}${CLR_BOLD}===============================================================================${CLR_RESET}"
  echo -e "${CLR_GREEN}${CLR_BOLD}  ¡INSTALACIÓN DE EMUBOX COMPLETADA CON ÉXITO!                                 ${CLR_RESET}"
  echo -e "${CLR_GREEN}${CLR_BOLD}===============================================================================${CLR_RESET}"
  echo -e "  Directorio de ROMs:    ${CLR_CYAN}${XDG_DATA_HOME:-$HOME/.local/share}/emubox/roms${CLR_RESET}"
  echo -e "  Fichero Configuración: ${CLR_CYAN}${XDG_CONFIG_HOME:-$HOME/.config}/emubox/config.json${CLR_RESET}"
  echo -e "  Para iniciar EmuBox:   ${CLR_BOLD}emubox${CLR_RESET}"
  echo -e "${CLR_GREEN}===============================================================================${CLR_RESET}\n"
}

main "$@"
