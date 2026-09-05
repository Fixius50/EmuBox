#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX DIAGNOSE - SYSTEM HEALTH & CONFIGURATION AUDITOR
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/logging.sh
. "${SCRIPT_DIR}/lib/logging.sh"
# shellcheck source=lib/paths.sh
. "${SCRIPT_DIR}/lib/paths.sh"
# shellcheck source=lib/detection.sh
. "${SCRIPT_DIR}/lib/detection.sh"
source "${SCRIPT_DIR}/lib/graphics.sh"
detect_emubox_graphics

OUTPUT_REPORT="emubox-diagnostics.txt"

log_banner
log_step "Generando informe de diagnóstico en ${OUTPUT_REPORT}..."

{
  echo "==============================================================================="
  echo "                     EMUBOX OS - DIAGNOSTIC REPORT                             "
  echo "==============================================================================="
  echo "Fecha de generación: $(date)"
  echo "Usuario:             $(whoami)"
  echo "Kernel:              $(uname -r)"
  echo "Arquitectura:        $(get_emubox_architecture)"
  echo "Kernel architecture: $(uname -m)"
  echo "CPU cores:           $(getconf _NPROCESSORS_ONLN)"
  grep -E 'model name|Hardware|Model' /proc/cpuinfo | head -n 1 || true
  grep MemTotal /proc/meminfo || true
  echo "GPU: $GPU_VENDOR; renderer: $RENDERER_DESC; driver: $GPU_DRIVER"
  echo "DRM: $HAS_DRM; Vulkan: $HAS_HW_VULKAN; Gamescope: $HAS_GAMESCOPE"
  echo "Compositor: $EMUBOX_COMPOSITOR; device: $DEVICE_MODEL"
  echo ""
  echo "--- Appliance Filesystem Status ---"
  echo "Config Dir:  ${EMUBOX_CONFIG_DIR} (Existe: $([ -d "${EMUBOX_CONFIG_DIR}" ] && echo "SÍ" || echo "NO"))"
  echo "Data Dir:    ${EMUBOX_DATA_DIR} (Existe: $([ -d "${EMUBOX_DATA_DIR}" ] && echo "SÍ" || echo "NO"))"
  echo "Cache Dir:   ${EMUBOX_CACHE_DIR} (Existe: $([ -d "${EMUBOX_CACHE_DIR}" ] && echo "SÍ" || echo "NO"))"
  echo "Logs Dir:    ${EMUBOX_LOGS_DIR} (Existe: $([ -d "${EMUBOX_LOGS_DIR}" ] && echo "SÍ" || echo "NO"))"
  echo ""
  echo "--- Emuladores en el PATH ---"
  for bin in retroarch duckstation-qt pcsx2-qt mgba-qt flycast gamescope; do
    if command -v "$bin" >/dev/null 2>&1; then
      echo "  [INSTALADO] $bin -> $(command -v "$bin")"
    else
      echo "  [AUSENTE]   $bin"
    fi
  done
  echo ""
  echo "--- Mandos / Input (udev) ---"
  if [[ -f /etc/udev/rules.d/99-emubox-gamepads.rules ]]; then
    echo "Reglas udev presentes: SÍ"
  else
    echo "Reglas udev presentes: NO"
  fi
  echo "==============================================================================="
} | tee "${OUTPUT_REPORT}"

log_ok "Informe generado con éxito en ${OUTPUT_REPORT}."
