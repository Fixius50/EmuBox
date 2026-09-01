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
  local pkgs=("$@")
  local missing=()

  for p in "${pkgs[@]}"; do
    if ! is_pkg_installed "${p}"; then
      missing+=("${p}")
    fi
  done

  if [[ ${#missing[@]} -gt 0 ]]; then
    log_step "Instalando paquetes de Arch Linux: ${missing[*]}"
    if command -v sudo >/dev/null 2>&1; then
      sudo pacman -S --needed --noconfirm "${missing[@]}" || log_warn "Algunos paquetes no se pudieron instalar automáticamente."
    else
      log_warn "sudo no disponible para pacman."
    fi
  else
    log_ok "Todos los paquetes ya están instalados."
  fi
}
