#!/usr/bin/env bash
set -Eeuo pipefail

provider="${1:-epic}"
[[ "${provider}" == "epic" ]] || {
  printf '%s\n' 'Uso: setup-store-adapters.sh epic' >&2
  exit 2
}
[[ "${EUID}" -eq 0 ]] || {
  printf '%s\n' 'Este instalador debe ejecutarse como root.' >&2
  exit 1
}

root=/var/lib/emubox/stores/epic
runtime="${root}/runtime"
user=emubox
version=0.21.1

pacman -S --needed --noconfirm python-pip
install -d -m 0700 -o "${user}" -g "${user}" "${root}"
runuser -u "${user}" -- env HOME="${root}/home" XDG_CONFIG_HOME="${root}/config" \
  python3 -m venv "${runtime}"
runuser -u "${user}" -- "${runtime}/bin/python" -m pip install --disable-pip-version-check --no-cache-dir \
  "legendary-gl==${version}"
chmod 0755 "${runtime}/bin/legendary"
printf '%s\n' "Adaptador Epic Legendary ${version} instalado en ${runtime}."