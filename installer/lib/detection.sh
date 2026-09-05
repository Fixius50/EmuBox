#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX INSTALLER - HARDWARE & ENVIRONMENT DETECTION
# ==============================================================================

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/architecture.sh"

detect_architecture() {
  local arch
  arch=$(get_emubox_architecture)
  if [[ "${arch}" == "unsupported" ]]; then
    log_error "EmuBox requiere x86_64 o aarch64 (detectado: $(uname -m))."
    return 1
  fi
  log_ok "Arquitectura validada: ${arch}"
  return 0
}

detect_distribution() {
  if [[ -f /etc/os-release ]]; then
    # shellcheck source=/dev/null
    . /etc/os-release
    if ! is_supported_emubox_distribution "${ID:-}" "$(uname -m)"; then
      log_error "Distribucion/CPU no soportada: ${ID:-unknown}/$(uname -m)."
      return 1
    else
      log_ok "Distribución validada: ${NAME}"
    fi
  else
    log_error "No se pudo leer /etc/os-release."
    return 1
  fi
}

detect_gpu() {
  source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/graphics.sh"
  detect_emubox_graphics
  echo "$GPU_VENDOR"
}
