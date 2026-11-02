#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX SETUP - SYSTEM DEPENDENCIES & PACKAGE INSTALLER (ARCH LINUX)
# ==============================================================================

set -euo pipefail

for arg in "$@"; do
  case "$arg" in
    -v|--verbose)
      export VERBOSE=1
      ;;
  esac
done
INSTALLER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$INSTALLER_DIR/lib/logging.sh"
source "$INSTALLER_DIR/lib/detection.sh"
source "$INSTALLER_DIR/lib/packages.sh"
detect_architecture
detect_distribution

# Core System & Compositor Dependencies
CORE_PKGS=(
  "cage"
  "wlr-randr"
  "gtk3"
  "webkit2gtk-4.1"
  "pipewire"
  "pipewire-pulse"
  "pipewire-alsa"
  "wireplumber"
  "rtkit"
  "dosfstools"
  "e2fsprogs"
  "mesa"
  "libdrm"
  "mesa-utils"
  "xdg-user-dirs"
  "libevdev"
  "libarchive"
  "cabextract"
  "innoextract"
  "unshield"
  "bubblewrap"
  "util-linux"
  "python-pip"
  "sqlite"
)

echo "  -> Comprobando gestor de paquetes de Arch Linux (pacman)..."

install_packages_if_missing "${CORE_PKGS[@]}"
if [[ "$(uname -m)" == x86_64 ]]; then
  install_packages_if_missing wine
fi
install_optional_packages gamescope vulkan-tools qbittorrent-nox mold sccache
install_optional_packages libretro-snes9x libretro-mupen64plus-next libretro-genesis-plus-gx mgba-qt ppsspp gamemode mangohud
