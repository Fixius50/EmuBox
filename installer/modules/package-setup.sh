#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX MODULE - PACKAGE DEPENDENCIES
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=lib/logging.sh
. "${SCRIPT_DIR}/lib/logging.sh"
# shellcheck source=lib/packages.sh
. "${SCRIPT_DIR}/lib/packages.sh"

REQUIRED_PACKAGES=(
  "sudo"
  "cage"
  "foot"
  "pipewire"
  "pipewire-pulse"
  "wireplumber"
  "rtkit"
  "dosfstools"
  "e2fsprogs"
  "mesa"
  "mesa-utils"
  "xdg-user-dirs"
  "libevdev"
)

log_step "Comprobando e instalando paquetes necesarios..."
install_packages_if_missing "${REQUIRED_PACKAGES[@]}"
install_packages_if_missing libarchive cabextract innoextract unshield bubblewrap util-linux
if [[ "$(uname -m)" == x86_64 ]]; then
  install_packages_if_missing wine
fi
install_optional_packages gamescope vulkan-tools aria2
install_optional_packages libretro-snes9x libretro-mupen64plus-next libretro-genesis-plus-gx mgba-qt ppsspp gamemode mangohud
