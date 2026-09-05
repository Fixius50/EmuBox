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
  "libretro-core-info"
  "retroarch"
  "xdg-user-dirs"
  "libevdev"
)

# Optional Native Standalone Emulators (if available in official repos / AUR)
EMU_PKGS=(
  "duckstation-qt"
  "mgba-qt"
  "flycast"
)

echo "  -> Comprobando gestor de paquetes de Arch Linux (pacman)..."

install_packages_if_missing "${CORE_PKGS[@]}"
install_optional_packages gamescope vulkan-tools mesa-utils "${EMU_PKGS[@]}"
if [[ "$(get_emubox_architecture)" == x86_64 ]]; then
  install_optional_packages pcsx2
fi
