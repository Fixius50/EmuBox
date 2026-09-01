#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX MODULE - INPUT & UDEV RULES SETUP
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=lib/logging.sh
. "${SCRIPT_DIR}/lib/logging.sh"
# shellcheck source=lib/permissions.sh
. "${SCRIPT_DIR}/lib/permissions.sh"

log_step "Configurando permisos de entrada y mandos de consola..."
setup_user_groups
setup_udev_rules
