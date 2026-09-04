#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - CENTRO DE CONTROL Y MENÚ DE SCRIPTS
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}"

# Colores ANSI
BOLD='\033[1m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # Sin color

show_header() {
  clear 2>/dev/null || true
  echo -e "${CYAN}${BOLD}"
  echo "======================================================================"
  echo "                     🎮 EMUBOX OS - CONTROL CENTER                   "
  echo "======================================================================"
  echo -e "${NC}"
  echo -e " Directorio: ${BOLD}${SCRIPT_DIR}${NC}"
  echo -e " Usuario:    ${BOLD}$(whoami)${NC} (UID: $(id -u))"
  echo -e " Terminal:   ${BOLD}$(tty 2>/dev/null || echo 'pts/ssh')${NC}"
  echo "----------------------------------------------------------------------"
}

pause_screen() {
  echo ""
  echo -e "${YELLOW}Pulsa Enter para volver al menú...${NC}"
  read -r _
}

run_and_check() {
  local desc="$1"
  shift
  echo ""
  echo -e "${CYAN}==> Ejecutando: ${desc}${NC}"
  echo "----------------------------------------------------------------------"
  
  if "$@"; then
    echo "----------------------------------------------------------------------"
    echo -e "${GREEN}${BOLD}[ÉXITO] ${desc} finalizó correctamente.${NC}"
    return 0
  else
    local code=$?
    echo "----------------------------------------------------------------------"
    echo -e "${RED}${BOLD}[ERROR] ${desc} falló con código de salida ${code}.${NC}"
    return "${code}"
  fi
}

while true; do
  show_header
  echo -e "${BOLD}Selecciona una opción:${NC}"
  echo ""
  echo -e "  ${GREEN}1)${NC} ${BOLD}Compilar EmuBox${NC} (frontend + binario Tauri)"
  echo -e "  ${GREEN}2)${NC} ${BOLD}Actualizar desde GitHub${NC} (actualización + compilación + despliegue)"
  echo -e "  ${GREEN}3)${NC} ${BOLD}Configurar Appliance${NC} (permisos + autologin + servicios)"
  echo -e "  ${GREEN}4)${NC} ${BOLD}Diagnosticar Entorno${NC} (GPU, Vulkan, DRM, systemd y logs)"
  echo -e "  ${GREEN}5)${NC} ${BOLD}Instalación Completa Arch Linux${NC} (aprovisionamiento inicial)"
  echo ""
  echo -e "  ${RED}0)${NC} Salir"
  echo "----------------------------------------------------------------------"
  read -rp "Opción [0-5]: " OPTION

  case "${OPTION}" in
    1)
      chmod +x scripts/build.sh
      if run_and_check "Compilación EmuBox (scripts/build.sh)" bash scripts/build.sh; then
        echo -e "${YELLOW}[INFO] La compilación queda preparada para el siguiente arranque.${NC}"
      fi
      pause_screen
      ;;
    2)
      chmod +x scripts/update-emubox.sh
      run_and_check "Actualización desde GitHub (scripts/update-emubox.sh)" bash scripts/update-emubox.sh || true
      pause_screen
      ;;
    3)
      chmod +x scripts/setup-autostart.sh
      run_and_check "Configuración de Autoarranque (scripts/setup-autostart.sh)" sudo bash scripts/setup-autostart.sh || true
      pause_screen
      ;;
    4)
      echo ""
      echo -e "${CYAN}================ DIAGNÓSTICO RÁPIDO ================${NC}"
      echo -e "${BOLD}1. Virtualización:${NC} $(command -v systemd-detect-virt >/dev/null && systemd-detect-virt || echo 'N/A')"
      echo -e "${BOLD}2. Dispositivos DRM:${NC}"
      ls -l /dev/dri/ 2>/dev/null || echo "No se encontró /dev/dri"
      echo ""
      echo -e "${BOLD}3. Vulkan Hardware:${NC}"
      if command -v vulkaninfo >/dev/null 2>&1; then
        vulkaninfo --summary 2>&1 | grep -E 'deviceName|GPU0|GPU id|driverName' || echo "Vulkan no detecta GPU válida."
      else
        echo "vulkaninfo no instalado."
      fi
      echo ""
      echo -e "${BOLD}4. Estado de getty@tty1:${NC}"
      systemctl is-active getty@tty1 2>/dev/null && echo "Activo (Autologin TTY1)" || echo "Inactivo"
      echo ""
      echo -e "${BOLD}5. Últimas líneas de log (/var/log/emubox/session.log):${NC}"
      if [[ -f /var/log/emubox/session.log ]]; then
        tail -n 15 /var/log/emubox/session.log
      elif [[ -f /var/log/emubox/emubox.log ]]; then
        tail -n 15 /var/log/emubox/emubox.log
      else
        echo "No hay logs recientes en /var/log/emubox/"
      fi
      pause_screen
      ;;
    5)
      echo ""
      echo -e "${RED}${BOLD}ADVERTENCIA:${NC} Esto reinstalará las dependencias completas del sistema."
      read -rp "¿Estás seguro de continuar? [s/N]: " CONFIRM
      if [[ "${CONFIRM}" =~ ^[sS]$ ]]; then
        chmod +x scripts/setup-arch.sh
        run_and_check "Instalación de Sistema (scripts/setup-arch.sh)" sudo bash scripts/setup-arch.sh || true
      else
        echo "Operación cancelada."
      fi
      pause_screen
      ;;
    0)
      echo ""
      echo -e "${GREEN}¡Hasta pronto!${NC}"
      exit 0
      ;;
    *)
      echo -e "${RED}Opción no válida.${NC}"
      sleep 1
      ;;
  esac
done
