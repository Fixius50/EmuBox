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
    local gpu_info
    gpu_info="$(lspci -nnk 2>/dev/null | grep -A3 -Ei 'VGA compatible controller|3D controller|Display controller' || true)"
    if echo "${gpu_info}" | grep -Eiq 'VMware|vmwgfx|VirtualBox|vboxvideo|virtio|qxl'; then
      gpu="generic"
    elif echo "${gpu_info}" | grep -Eiq 'NVIDIA|nvidia|nouveau'; then
      gpu="nvidia"
    elif echo "${gpu_info}" | grep -Eiq 'Intel|i915|xe'; then
      gpu="intel"
    elif echo "${gpu_info}" | grep -Eiq 'AMD|ATI|Radeon|amdgpu'; then
      gpu="amd"
    fi
  fi
  echo "${gpu}"
}
