#!/usr/bin/env bash

secure_boot_fstab() {
  awk '
    /^[[:space:]]*#/ || NF == 0 { print; next }
    $2 == "/boot" {
      matches++
      if ($3 != "vfat" || NF < 6) { invalid = 1; next }
      count = split($4, options, ",")
      retained = ""
      for (option_index = 1; option_index <= count; option_index++) {
        if (options[option_index] !~ /^(fmask|dmask|umask)=/)
          retained = retained (retained == "" ? "" : ",") options[option_index]
      }
      $4 = retained ",fmask=0077,dmask=0077"
      $6 = 2
      print
      next
    }
    { print }
    END { if (matches != 1 || invalid) exit 1 }
  ' OFS='\t' "$1"
}

repair_boot() {
  set -euo pipefail
  if [[ "${1:-}" != --apply || "$#" != 1 ]]; then
    printf 'Uso: sudo bash scripts/repair-boot.sh --apply\n' >&2
    return 1
  fi
  if [[ "$EUID" != 0 ]]; then
    printf 'Se requieren privilegios de administrador. No se ha cambiado nada.\n' >&2
    return 1
  fi
  for tool in fsck.fat findmnt blockdev cp install mktemp awk sync mount umount systemctl; do
    command -v "$tool" >/dev/null || {
      printf 'Falta %s. Instala dosfstools/util-linux antes del mantenimiento.\n' "$tool" >&2
      return 1
    }
  done
  [[ -f /etc/fstab && ! -L /etc/fstab ]] || return 1
  local device configured_device filesystem mounts backup staged_fstab
  device=$(findmnt --mountpoint /boot --noheadings --raw --output SOURCE)
  configured_device=$(findmnt --fstab --evaluate --mountpoint /boot --noheadings --raw --output SOURCE)
  filesystem=$(findmnt --mountpoint /boot --noheadings --raw --output FSTYPE)
  mounts=$(findmnt --source "$device" --noheadings --raw --output TARGET)
  if [[ "$filesystem" != vfat || ! -b "$device" || "$mounts" != /boot ||
        "$(readlink -f "$device")" != "$(readlink -f "$configured_device")" ||
        "$(findmnt --submounts --mountpoint /boot --noheadings --raw --output TARGET)" != /boot ]]; then
    printf 'Se requiere /boot separado, FAT, sin submontajes y coincidente con fstab.\n' >&2
    return 1
  fi

  install -d -m 0700 /var/backups/emubox
  backup=$(mktemp -d /var/backups/emubox/boot-XXXXXXXX)
  cp -a /etc/fstab "$backup/fstab"
  staged_fstab=$(mktemp /etc/.fstab-emubox-XXXXXXXX)
  local detached=0 recovered=0 committed=0
  trap '
    status=$?
    if [[ "$detached" == 1 ]]; then
      if [[ "$committed" == 1 ]]; then
        cp -a "$backup/fstab" /etc/fstab
        systemctl daemon-reload || true
      fi
      if [[ "$recovered" == 1 ]]; then
        mount /boot || printf "ERROR: /boot sigue desmontado; revisar desde consola.\n" >&2
      else
        mount -o ro /boot || printf "ERROR: /boot sigue desmontado; usar rescate.\n" >&2
      fi
    fi
    rm -f -- "$staged_fstab"
    printf "Respaldo conservado en %s\n" "$backup"
    exit "$status"
  ' EXIT
  secure_boot_fstab "$backup/fstab" > "$staged_fstab"
  chmod --reference=/etc/fstab "$staged_fstab"
  chown --reference=/etc/fstab "$staged_fstab"
  findmnt --verify --tab-file "$staged_fstab"
  printf 'Se reparara %s; respaldo privado: %s\n' "$device" "$backup"
  sync -f /boot
  umount /boot
  detached=1
  if findmnt --source "$device" --noheadings --output TARGET; then
    printf 'El dispositivo sigue montado; no se ejecuta fsck.\n' >&2
    exit 1
  fi
  cp --sparse=always -- "$device" "$backup/boot-before.img"
  [[ "$(stat -c %s "$backup/boot-before.img")" == "$(blockdev --getsize64 "$device")" ]]
  sync -f "$backup/boot-before.img"
  local repair_status=0
  fsck.fat -a "$device" > "$backup/fsck.log" 2>&1 || repair_status=$?
  cat "$backup/fsck.log"
  if (( repair_status > 1 )); then
    printf 'FAT requiere intervencion manual; no se fuerza la reparacion.\n' >&2
    exit 1
  fi
  fsck.fat -n "$device"
  recovered=1
  mv -f -- "$staged_fstab" /etc/fstab
  committed=1
  sync -f /etc/fstab
  systemctl daemon-reload
  mount /boot
  detached=0
  findmnt --mountpoint /boot --output TARGET,SOURCE,FSTYPE,OPTIONS
  [[ "$(stat -c %a /boot)" == 700 ]]
  if [[ -f /boot/loader/random-seed ]]; then
    [[ "$(stat -c %a /boot/loader/random-seed)" == 700 ]]
  fi
  printf 'FAT comprobada y /boot privado. Respaldo: %s\n' "$backup"
  printf 'No se ha reiniciado el equipo ni la UI.\n'
  trap - EXIT
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  repair_boot "$@"
fi