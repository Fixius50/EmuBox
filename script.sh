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
  echo -e " Directorio activo: ${BOLD}${SCRIPT_DIR}${NC}"
  echo -e " Usuario:           ${BOLD}$(whoami)${NC} (UID: $(id -u))"
  echo -e " Terminal:          ${BOLD}$(tty 2>/dev/null || echo 'pts/ssh')${NC}"
  echo "----------------------------------------------------------------------"
}

pause_screen() {
  echo ""
  echo -e "${YELLOW}Pulsa Enter para volver al menú...${NC}"
  read -r _
}

while true; do
  show_header
  echo -e "${BOLD}Selecciona una opción:${NC}"
  echo ""
  echo -e "  ${GREEN}1)${NC} ${BOLD}Compilar EmuBox${NC} (npm ci, Vite build, Tauri build -> bin/emubox)"
  echo -e "  ${GREEN}2)${NC} ${BOLD}Actualizar desde GitHub${NC} (git pull, build atómico y reinicio seguro)"
  echo -e "  ${GREEN}3)${NC} ${BOLD}Configurar Autoarranque${NC} (getty@tty1, autologin, lanzador Wayland)"
  echo -e "  ${GREEN}4)${NC} ${BOLD}Probar Lanzamiento${NC} (Lanzar EmuBox con Cage / modo adaptativo)"
  echo -e "  ${GREEN}5)${NC} ${BOLD}Ejecutar Tests Automatizados${NC} (npm test - 42 contratos)"
  echo -e "  ${GREEN}6)${NC} ${BOLD}Iniciar Servidor de Desarrollo${NC} (npm run dev)"
  echo -e "  ${GREEN}7)${NC} ${BOLD}Diagnosticar Entorno y Logs${NC} (GPU, Vulkan, DRM, Systemd, TTY1)"
  echo -e "  ${GREEN}8)${NC} ${BOLD}Reiniciar Consola Física${NC} (systemctl restart getty@tty1)"
  echo -e "  ${GREEN}9)${NC} ${BOLD}Instalación Completa Arch Linux${NC} (scripts/setup-arch.sh)"
  echo ""
  echo -e "  ${RED}0)${NC} Salir"
  echo "----------------------------------------------------------------------"
  read -rp "Opción [0-9]: " OPTION

  case "${OPTION}" in
    1)
      echo ""
      echo -e "${CYAN}==> Ejecutando: bash scripts/build.sh${NC}"
      chmod +x scripts/build.sh
      bash scripts/build.sh || true
      pause_screen
      ;;
    2)
      echo ""
      echo -e "${CYAN}==> Ejecutando: bash scripts/update-emubox.sh${NC}"
      chmod +x scripts/update-emubox.sh
      bash scripts/update-emubox.sh || true
      pause_screen
      ;;
    3)
      echo ""
      echo -e "${CYAN}==> Ejecutando con sudo: scripts/setup-autostart.sh${NC}"
      chmod +x scripts/setup-autostart.sh
      sudo bash scripts/setup-autostart.sh || true
      pause_screen
      ;;
    4)
      echo ""
      echo -e "${CYAN}==> Ejecutando: bash scripts/run.sh${NC}"
      chmod +x scripts/run.sh
      bash scripts/run.sh || true
      pause_screen
      ;;
    5)
      echo ""
      echo -e "${CYAN}==> Ejecutando: npm test${NC}"
      npm test || true
      pause_screen
      ;;
    6)
      echo ""
      echo -e "${CYAN}==> Ejecutando: npm run dev${NC}"
      npm run dev || true
      pause_screen
      ;;
    7)
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
    8)
      echo ""
      echo -e "${YELLOW}==> Reiniciando sesión de consola en TTY1...${NC}"
      sudo systemctl restart getty@tty1 2>/dev/null && echo -e "${GREEN}[OK] getty@tty1 reiniciado.${NC}" || echo -e "${RED}[ERROR] No se pudo reiniciar.${NC}"
      pause_screen
      ;;
    9)
      echo ""
      echo -e "${RED}${BOLD}ADVERTENCIA:${NC} Esto reinstalará las dependencias completas del sistema."
      read -rp "¿Estás seguro de continuar? [s/N]: " CONFIRM
      if [[ "${CONFIRM}" =~ ^[sS]$ ]]; then
        chmod +x scripts/setup-arch.sh
        sudo bash scripts/setup-arch.sh || true
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
