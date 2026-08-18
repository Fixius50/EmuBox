#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX SETUP - XDG BASE DIRECTORY HIERARCHY INITIALIZER
# ==============================================================================

set -euo pipefail

XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
XDG_CACHE_HOME="${XDG_CACHE_HOME:-$HOME/.cache}"

EMUBOX_CONFIG_DIR="${XDG_CONFIG_HOME}/emubox"
EMUBOX_DATA_DIR="${XDG_DATA_HOME}/emubox"
EMUBOX_CACHE_DIR="${XDG_CACHE_HOME}/emubox"

echo "  -> Creando estructura de directorios XDG..."

# 1. Configuration Directories
mkdir -p "${EMUBOX_CONFIG_DIR}"

# 2. Data Directories & Consoles ROM Folders
mkdir -p "${EMUBOX_DATA_DIR}"/{saves,states,screenshots,covers,logs}
mkdir -p "${EMUBOX_DATA_DIR}"/roms/{snes,ps1,ps2,n64,genesis,gba,dreamcast,arcade,gamecube,psp,nds}

# 3. Cache Directory
mkdir -p "${EMUBOX_CACHE_DIR}"

# 4. Correct User Permissions
chmod 755 "${EMUBOX_CONFIG_DIR}" "${EMUBOX_DATA_DIR}" "${EMUBOX_CACHE_DIR}"

echo "  ✓ Directorio de configuración: ${EMUBOX_CONFIG_DIR}"
echo "  ✓ Directorio de ROMs y datos:   ${EMUBOX_DATA_DIR}"
echo "  ✓ Directorio de caché:          ${EMUBOX_CACHE_DIR}"
