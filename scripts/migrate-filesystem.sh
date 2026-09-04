#!/usr/bin/env bash
set -Eeuo pipefail

if [[ ${EUID} -ne 0 ]]; then
    echo "Ejecuta este script como root: sudo $0" >&2
    exit 1
fi

EMUBOX_USER="${SUDO_USER:-emubox}"
EMUBOX_GROUP="${EMUBOX_USER}"

mkdir -p \
    /etc/emubox \
    /var/lib/emubox/{games,emulators,bios,saves,states,screenshots} \
    /var/cache/emubox/{shaders,metadata,covers,downloads} \
    /var/log/emubox \
    /run/emubox

copy_if_present() {
    local source="$1"
    local destination="$2"
    if [[ -d "$source" ]]; then
        mkdir -p "$destination"
        cp -a -n "$source"/. "$destination"/
    fi
}

# Copy legacy user data without overwriting canonical files.
copy_if_present "/home/${EMUBOX_USER}/.local/share/emubox/roms" "/var/lib/emubox/games"
copy_if_present "/home/${EMUBOX_USER}/.local/share/emubox/saves" "/var/lib/emubox/saves"
copy_if_present "/home/${EMUBOX_USER}/.local/share/emubox/states" "/var/lib/emubox/states"
copy_if_present "/home/${EMUBOX_USER}/.local/share/emubox/bios" "/var/lib/emubox/bios"
copy_if_present "/home/${EMUBOX_USER}/.local/share/emubox/screenshots" "/var/lib/emubox/screenshots"
copy_if_present "/home/${EMUBOX_USER}/.local/share/emubox/covers" "/var/cache/emubox/covers"
copy_if_present "/home/${EMUBOX_USER}/.config/emubox" "/etc/emubox"
copy_if_present "/home/${EMUBOX_USER}/.cache/emubox" "/var/cache/emubox"

if [[ -f /opt/emubox/data/config/config.json && ! -f /etc/emubox/config.json ]]; then
    cp -n /opt/emubox/data/config/config.json /etc/emubox/config.json
fi
if [[ -f /opt/emubox/data/emulators.json && ! -f /etc/emubox/emulators.json ]]; then
    cp -n /opt/emubox/data/emulators.json /etc/emubox/emulators.json
fi
if [[ ! -f /etc/emubox/download-links.txt ]]; then
    cp /opt/emubox/data/download-links.example.txt /etc/emubox/download-links.txt
fi

chown -R "${EMUBOX_USER}:${EMUBOX_GROUP}" /var/lib/emubox /var/cache/emubox /var/log/emubox /run/emubox
chmod 0755 /etc/emubox /var/lib/emubox /var/cache/emubox /var/log/emubox /run/emubox

echo "Filesystem canónico de EmuBox creado y datos legacy migrados sin sobrescritura."