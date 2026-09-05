#!/usr/bin/env bash
set -euo pipefail
INSTALLER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$INSTALLER_DIR/lib/logging.sh"
source "$INSTALLER_DIR/lib/detection.sh"
source "$INSTALLER_DIR/lib/packages.sh"
detect_architecture
detect_distribution

install_optional_packages retroarch libretro-core-info duckstation-qt mgba-qt flycast
if [[ "$(get_emubox_architecture)" == x86_64 ]]; then
  install_optional_packages pcsx2
fi