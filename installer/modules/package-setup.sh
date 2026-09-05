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
  "vulkan-icd-loader"
  "vulkan-tools"
  "libretro-core-info"
  "retroarch"
  "xdg-user-dirs"
  "libevdev"
)

log_step "Comprobando e instalando paquetes necesarios..."
install_packages_if_missing "${REQUIRED_PACKAGES[@]}"
install_optional_packages gamescope vulkan-tools
