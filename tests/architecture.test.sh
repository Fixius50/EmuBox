#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/installer/lib/architecture.sh"

[[ "$(get_emubox_architecture x86_64)" == x86_64 ]]
[[ "$(get_emubox_architecture aarch64)" == aarch64 ]]
[[ "$(get_emubox_architecture arm64)" == aarch64 ]]
[[ "$(get_emubox_architecture armv7l)" == unsupported ]]
[[ "$(get_emubox_architecture riscv64)" == unsupported ]]
[[ "$(get_emubox_target aarch64)" == aarch64-unknown-linux-gnu ]]
[[ "$(get_emubox_target x86_64)" == x86_64-unknown-linux-gnu ]]
! get_emubox_target armv7l
for architecture in x86_64 aarch64 armv7l; do
  for distribution in arch archarm manjaro unknown; do
    case "$distribution:$architecture" in
      arch:x86_64|archarm:aarch64) is_supported_emubox_distribution "$distribution" "$architecture" ;;
      *) ! is_supported_emubox_distribution "$distribution" "$architecture" ;;
    esac
  done
done
printf '%s\n' 'Architecture and distribution matrix: OK'
fixture_dir=$(mktemp -d)
trap 'rm -rf "$fixture_dir"' EXIT
printf '\177ELF\002\001\001\000\000\000\000\000\000\000\000\000\002\000\076\000' > "$fixture_dir/x86"
printf '\177ELF\002\001\001\000\000\000\000\000\000\000\000\000\002\000\267\000' > "$fixture_dir/arm"
validate_emubox_binary "$fixture_dir/x86" x86_64
validate_emubox_binary "$fixture_dir/arm" aarch64
! validate_emubox_binary "$fixture_dir/x86" aarch64
! validate_emubox_binary "$fixture_dir/arm" x86_64
! validate_emubox_binary "$fixture_dir/missing" x86_64
printf '%s\n' 'Binary architecture matrix: OK'