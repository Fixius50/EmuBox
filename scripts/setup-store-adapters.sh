#!/usr/bin/env bash
set -Eeuo pipefail

provider="${1:-epic}"
[[ "${provider}" == "epic" || "${provider}" == "gog" ]] || {
  printf '%s\n' 'Uso: setup-store-adapters.sh <epic|gog>' >&2
  exit 2
}
[[ "${EUID}" -eq 0 ]] || {
  printf '%s\n' 'Este instalador debe ejecutarse como root.' >&2
  exit 1
}

user=emubox
case "${provider}" in
  epic)
    root=/var/lib/emubox/stores/epic
    package='legendary-gl==0.21.1'
    executable=legendary
    ;;
  gog)
    root=/var/lib/emubox/stores/gog
    executable=gogdl
    ;;
esac
runtime="${root}/runtime"

pacman -S --needed --noconfirm python-pip
install -d -m 0700 -o "${user}" -g "${user}" "${root}"
runuser -u "${user}" -- env HOME="${root}/home" XDG_CONFIG_HOME="${root}/config" \
  python3 -m venv "${runtime}"
if [[ "${provider}" == "gog" ]]; then
  build_dir="$(mktemp -d)"
  trap 'rm -rf "${build_dir}"' EXIT
  chown "${user}:${user}" "${build_dir}"
  runuser -u "${user}" -- git clone --depth 1 --branch v1.3.0 --recurse-submodules \
    https://github.com/Heroic-Games-Launcher/heroic-gogdl.git "${build_dir}/gogdl"
  runuser -u "${user}" -- "${runtime}/bin/python" -m pip install --disable-pip-version-check --no-cache-dir \
    "${build_dir}/gogdl"
else
  runuser -u "${user}" -- "${runtime}/bin/python" -m pip install --disable-pip-version-check --no-cache-dir \
    "legendary-gl==0.21.1"
fi
chmod 0755 "${runtime}/bin/${executable}"
printf '%s\n' "Adaptador ${provider} instalado en ${runtime}."