#!/usr/bin/env bash

get_emubox_architecture() {
  case "${1:-$(uname -m)}" in
    x86_64) printf '%s\n' x86_64 ;;
    aarch64|arm64) printf '%s\n' aarch64 ;;
    *) printf '%s\n' unsupported ;;
  esac
}

get_emubox_target() {
  case "$(get_emubox_architecture "${1:-$(uname -m)}")" in
    x86_64) printf '%s\n' x86_64-unknown-linux-gnu ;;
    aarch64) printf '%s\n' aarch64-unknown-linux-gnu ;;
    *) return 1 ;;
  esac
}

is_supported_emubox_distribution() {
  case "${1}:$(get_emubox_architecture "${2}")" in
    arch:x86_64|archarm:aarch64) return 0 ;;
    *) return 1 ;;
  esac
}

get_emubox_binary_architecture() {
  local header
  [[ -f "$1" ]] || { printf '%s\n' missing; return; }
  header=$(od -An -v -tu1 -N20 "$1" | xargs)
  case "$header" in
    '127 69 76 70 2 1 '*' 62 0') printf '%s\n' x86_64 ;;
    '127 69 76 70 2 1 '*' 183 0') printf '%s\n' aarch64 ;;
    *) printf '%s\n' unsupported ;;
  esac
}

validate_emubox_binary() {
  local binary_arch host_arch
  binary_arch=$(get_emubox_binary_architecture "$1")
  host_arch=$(get_emubox_architecture "${2:-$(uname -m)}")
  if [[ "$host_arch" == unsupported || "$binary_arch" != "$host_arch" ]]; then
    printf 'Incompatible binary: %s (host=%s, binary=%s)\n' "$1" "$host_arch" "$binary_arch" >&2
    return 1
  fi
}