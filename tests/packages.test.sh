#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/installer/lib/packages.sh"
log_error() { printf '%s\n' "$*"; }
log_warn() { printf '%s\n' "$*"; }
pacman() {
  case "$1:$2" in
    -Qi:installed|-Si:available) return 0 ;;
    -Qi:*|-Si:*) return 1 ;;
    -S:*) return 0 ;;
    *) return 1 ;;
  esac
}
sudo() { "$@"; }
install_packages_if_missing installed available
! install_packages_if_missing unavailable
install_optional_packages unavailable
echo 'Required and optional package availability: OK'