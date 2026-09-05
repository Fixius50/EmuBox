#!/usr/bin/env bash
set -euo pipefail
project_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
fixture_dir=$(mktemp -d)
trap 'rm -rf "$fixture_dir"' EXIT
export PACKAGE_TEST_LOG="$fixture_dir/packages"
pacman() {
  printf '%s\n' "$*" >> "$PACKAGE_TEST_LOG"
  case "$1" in
    -Qi) return 1 ;;
    -Si|-S) return 0 ;;
    *) return 1 ;;
  esac
}
sudo() { "$@"; }
export -f pacman sudo
bash "$project_dir/installer/modules/package-setup.sh"
if grep -Eq 'retroarch|libretro|duckstation|pcsx2|rpcs3|dolphin|mgba|flycast' "$PACKAGE_TEST_LOG"; then
  echo 'An emulator was requested by the base appliance' >&2
  exit 1
fi
for package in cage pipewire wireplumber libevdev; do
  grep -q -- "-Si $package" "$PACKAGE_TEST_LOG"
done
printf '%s\n' 'Base appliance packages do not install emulators: OK'