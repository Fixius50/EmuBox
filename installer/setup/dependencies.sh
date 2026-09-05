#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX SETUP - SYSTEM DEPENDENCIES & PACKAGE INSTALLER (ARCH LINUX)
# ==============================================================================

set -euo pipefail
INSTALLER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$INSTALLER_DIR/lib/logging.sh"
source "$INSTALLER_DIR/lib/detection.sh"
source "$INSTALLER_DIR/lib/packages.sh"
detect_architecture
detect_distribution

# Core System & Compositor Dependencies
CORE_PKGS=(
  "cage"
  "pipewire"
  "pipewire-pulse"
  "pipewire-alsa"
  "wireplumber"
  "vulkan-icd-loader"
  "xdg-user-dirs"
  "libevdev"
)

echo "  -> Comprobando gestor de paquetes de Arch Linux (pacman)..."

install_packages_if_missing "${CORE_PKGS[@]}"
install_optional_packages gamescope vulkan-tools mesa-utils
