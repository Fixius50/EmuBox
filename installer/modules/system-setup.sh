#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX MODULE - SYSTEM SETUP & PRE-FLIGHT
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=lib/logging.sh
. "${SCRIPT_DIR}/lib/logging.sh"
# shellcheck source=lib/detection.sh
. "${SCRIPT_DIR}/lib/detection.sh"

log_step "Comprobando sistema operativo y compatibilidad de arquitectura..."
detect_architecture
detect_distribution
GPU_DETECTED="$(detect_gpu)"
log_ok "Adaptador GPU detectado: ${GPU_DETECTED}"
