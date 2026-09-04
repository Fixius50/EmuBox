#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX SETUP - XDG BASE DIRECTORY HIERARCHY INITIALIZER
# ==============================================================================

set -euo pipefail

EMUBOX_CONFIG_DIR="/etc/emubox"
EMUBOX_DATA_DIR="/var/lib/emubox"
EMUBOX_CACHE_DIR="/var/cache/emubox"
EMUBOX_LOGS_DIR="/var/log/emubox"
EMUBOX_RUNTIME_DIR="/run/emubox"

echo "  -> Creando estructura de directorios del appliance..."

# 1. Configuration, persistent data, cache, logs and runtime directories
mkdir -p "${EMUBOX_CONFIG_DIR}" "${EMUBOX_RUNTIME_DIR}"

# 2. Persistent data and console game folders
mkdir -p "${EMUBOX_DATA_DIR}"/{games,emulators,bios,saves,states,screenshots}
mkdir -p "${EMUBOX_DATA_DIR}"/games/{snes,ps1,ps2,ps3,n64,genesis,gba,dreamcast,arcade,gamecube,wii,wiiu,psp,nds}

# 3. Regenerable cache and temporary downloads
mkdir -p "${EMUBOX_CACHE_DIR}"/{shaders,metadata,covers,downloads}

# 4. System logs
mkdir -p "${EMUBOX_LOGS_DIR}"

# 5. Correct permissions
chmod 755 "${EMUBOX_CONFIG_DIR}" "${EMUBOX_DATA_DIR}" "${EMUBOX_CACHE_DIR}" "${EMUBOX_LOGS_DIR}" "${EMUBOX_RUNTIME_DIR}"

echo "  ✓ Directorio de configuración: ${EMUBOX_CONFIG_DIR}"
echo "  ✓ Directorio de ROMs y datos:   ${EMUBOX_DATA_DIR}"
echo "  ✓ Directorio de caché:          ${EMUBOX_CACHE_DIR}"
echo "  ✓ Directorio de logs:            ${EMUBOX_LOGS_DIR}"
echo "  ✓ Directorio runtime:            ${EMUBOX_RUNTIME_DIR}"
