#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX INSTALLER - ARCH LINUX PACKAGE MANAGEMENT
# ==============================================================================

is_pkg_installed() {
  local pkg="$1"
  if command -v pacman >/dev/null 2>&1; then
    pacman -Qi "${pkg}" >/dev/null 2>&1
  else
    false
  fi
}

install_packages_if_missing() {
  local package
  local missing=()
  for package in "$@"; do
    is_pkg_installed "$package" && continue
    if ! pacman -Si "$package" >/dev/null 2>&1; then
      log_error "Required package unavailable on $(uname -m): $package"
      return 1
    fi
    missing+=("$package")
  done
  [[ ${#missing[@]} -gt 0 ]] || return 0
  local pacman_cmd=("pacman" "-S" "--needed" "--noconfirm")
  if [[ "${VERBOSE:-0}" -eq 1 ]]; then
    pacman_cmd+=("-v")
    echo "  [pacman] Instalando/descargando paquetes (${#missing[@]}): ${missing[*]}"
  fi
  if [[ "$EUID" -eq 0 ]]; then
    "${pacman_cmd[@]}" "${missing[@]}"
  else
    sudo "${pacman_cmd[@]}" "${missing[@]}"
  fi
}

install_optional_packages() {
  local package
  for package in "$@"; do
    if is_pkg_installed "$package"; then
      [[ "${VERBOSE:-0}" -eq 1 ]] && echo "  [pacman] Paquete opcional ya disponible: $package"
      continue
    elif pacman -Si "$package" >/dev/null 2>&1; then
      install_packages_if_missing "$package" || log_warn "Optional package failed: $package"
    else
      log_warn "Optional package unavailable on $(uname -m): $package"
    fi
  done
}
