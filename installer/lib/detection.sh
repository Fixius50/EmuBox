#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX INSTALLER - HARDWARE & ENVIRONMENT DETECTION
# ==============================================================================

detect_architecture() {
  local arch
  arch=$(uname -m)
  if [[ "${arch}" != "x86_64" ]]; then
    log_error "EmuBox requiere arquitectura x86_64 (detectado: ${arch})."
    return 1
  fi
  log_ok "Arquitectura validada: ${arch}"
  return 0
}

detect_distribution() {
  if [[ -f /etc/os-release ]]; then
    # shellcheck source=/dev/null
    . /etc/os-release
    if [[ "${ID:-}" != "arch" && "${ID_LIKE:-}" != *"arch"* ]]; then
      log_warn "Distribución detectada (${NAME:-Desconocida}) no es Arch Linux nativo."
    else
      log_ok "Distribución validada: ${NAME}"
    fi
  else
    log_warn "No se pudo leer /etc/os-release. Continuando..."
  fi
}

detect_gpu() {
  local gpu="generic"
  if command -v lspci >/dev/null 2>&1; then
    if lspci | grep -i 'vga\|3d\|display' | grep -iq 'amd\|radeon'; then
      gpu="amd"
    elif lspci | grep -i 'vga\|3d\|display' | grep -iq 'nvidia'; then
      gpu="nvidia"
    elif lspci | grep -i 'vga\|3d\|display' | grep -iq 'intel'; then
      gpu="intel"
    fi
  fi
  echo "${gpu}"
}
