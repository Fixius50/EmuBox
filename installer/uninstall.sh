#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX OS - UNINSTALLER SCRIPT
#  Safely removes application files, sessions and services while preserving ROMs & saves
# ==============================================================================

set -euo pipefail

CLR_RESET="\033[0m"
CLR_BOLD="\033[1m"
CLR_YELLOW="\033[33m"
CLR_GREEN="\033[32m"
CLR_RED="\033[31m"

XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
XDG_CACHE_HOME="${XDG_CACHE_HOME:-$HOME/.cache}"

echo -e "${CLR_YELLOW}${CLR_BOLD}ADVERTENCIA: Vas a desinstalar EmuBox del sistema.${CLR_RESET}"
echo -e "Por defecto, tus ROMs, partidas guardadas y capturas se MANTENDRÁN intactas en:"
echo -e "  ${XDG_DATA_HOME}/emubox/"
echo ""
read -p "¿Deseas continuar con la desinstalación? (s/N): " -r CONFIRM
if [[ ! "${CONFIRM}" =~ ^[sSyY]$ ]]; then
  echo "Desinstalación cancelada."
  exit 0
fi

# 1. Stop and disable systemd session if active
if systemctl --user is-active --quiet emubox-session.service 2>/dev/null; then
  echo "Deteniendo servicio de sesión de consola..."
  systemctl --user stop emubox-session.service || true
  systemctl --user disable emubox-session.service || true
fi

# 2. Remove desktop entries & binaries
echo "Eliminando accesos y binarios..."
rm -f "$HOME/.local/share/applications/emubox.desktop"
rm -f "$HOME/.config/systemd/user/emubox-session.service"
sudo rm -f "/usr/local/bin/emubox" || true
sudo rm -rf "/opt/emubox" || true

# 3. Clean cache
echo "Limpiando caché..."
rm -rf "${XDG_CACHE_HOME}/emubox"

# 4. Optional removal of data
echo ""
read -p "¿Deseas ELIMINAR TAMBIÉN todas las ROMs y partidas guardadas? (s/N): " -r PURGE_DATA
if [[ "${PURGE_DATA}" =~ ^[sSyY]$ ]]; then
  echo "Eliminando directorio de datos y configuración..."
  rm -rf "${XDG_DATA_HOME}/emubox"
  rm -rf "${XDG_CONFIG_HOME}/emubox"
  echo -e "${CLR_RED}✓ Datos de usuario eliminados.${CLR_RESET}"
else
  echo -e "${CLR_GREEN}✓ Datos de ROMs y partidas preservados.${CLR_RESET}"
fi

echo -e "${CLR_GREEN}${CLR_BOLD}EmuBox ha sido desinstalado del sistema.${CLR_RESET}"
