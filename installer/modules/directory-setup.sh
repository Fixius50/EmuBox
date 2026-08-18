#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX MODULE - XDG DIRECTORY STRUCTURE SETUP
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=lib/logging.sh
. "${SCRIPT_DIR}/lib/logging.sh"
# shellcheck source=lib/paths.sh
. "${SCRIPT_DIR}/lib/paths.sh"

log_step "Creando jerarquía de directorios XDG..."

mkdir -p "${EMUBOX_CONFIG_DIR}"
mkdir -p "${EMUBOX_DATA_DIR}"/{saves,states,screenshots,covers,bios,logs}
mkdir -p "${EMUBOX_DATA_DIR}"/roms/{snes,ps1,ps2,n64,genesis,gba,dreamcast,arcade,gamecube,psp,nds}
mkdir -p "${EMUBOX_CACHE_DIR}"
mkdir -p "${EMUBOX_LOGS_DIR}"

chmod 755 "${EMUBOX_CONFIG_DIR}" "${EMUBOX_DATA_DIR}" "${EMUBOX_CACHE_DIR}" "${EMUBOX_LOGS_DIR}"

log_ok "Directorios XDG inicializados correctamente."
