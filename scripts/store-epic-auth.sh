#!/usr/bin/env bash
set -u

ROOT=/var/lib/emubox/stores/epic
LEGENDARY="${ROOT}/runtime/bin/legendary"

umask 077
mkdir -p "${ROOT}/home" "${ROOT}/config" "${ROOT}/legendary"
chmod 700 "${ROOT}" "${ROOT}/home" "${ROOT}/config" "${ROOT}/legendary"

if [[ ! -x "${LEGENDARY}" ]]; then
  printf '%s\n' 'El adaptador Epic no esta instalado.' >&2
  printf '%s\n' 'Ejecuta como administrador: /opt/emubox/scripts/setup-store-adapters.sh epic' >&2
  read -r -p 'Pulsa Intro para cerrar...'
  exit 1
fi

export HOME="${ROOT}/home"
export XDG_CONFIG_HOME="${ROOT}/config"
export LEGENDARY_CONFIG_PATH="${ROOT}/legendary"
export LC_ALL=C

"${LEGENDARY}" auth
status=$?
printf '\n%s\n' 'Vuelve a Tiendas y elige sincronizar para importar tu biblioteca.'
read -r -p 'Pulsa Intro para cerrar...'
exit "${status}"