#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX OS - MASTER INSTALLER & BOOTSTRAP ORCHESTRATOR FOR ARCH LINUX
#  Idempotent, Safe, Modular and Compliant with the appliance filesystem layout
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/logging.sh
. "${SCRIPT_DIR}/lib/logging.sh"
# shellcheck source=lib/paths.sh
. "${SCRIPT_DIR}/lib/paths.sh"

main() {
  exec bash "${SCRIPT_DIR}/../scripts/setup-arch.sh" "$@"
}

main "$@"
