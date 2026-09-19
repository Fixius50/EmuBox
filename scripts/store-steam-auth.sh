#!/usr/bin/env bash
set -euo pipefail

ROOT=/var/lib/emubox/stores/steam
FILE="${ROOT}/credentials.json"
umask 077
mkdir -p "${ROOT}"
chmod 700 "${ROOT}"

printf '%s\n' 'Steam usa tu propia clave Web API para consultar tu biblioteca.'
printf '%s\n' 'Se abrirá la página oficial para generar la clave. La contraseña y Steam Guard solo se introducen en Steam.'
xdg-open 'https://steamcommunity.com/dev/apikey' >/dev/null 2>&1 || true
printf '%s\n' 'Introduce tu SteamID64 y la clave Web API que Steam ha generado.'
read -r -p 'SteamID64: ' steam_id
read -r -s -p 'Clave Web API: ' api_key
printf '\n'
[[ "${steam_id}" =~ ^[0-9]+$ && -n "${api_key}" ]] || { printf '%s\n' 'Datos Steam inválidos.' >&2; exit 1; }
printf '{"steam_id":"%s","api_key":"%s"}\n' "${steam_id}" "${api_key}" > "${FILE}"
chmod 600 "${FILE}"
printf '%s\n' 'Credenciales guardadas en almacenamiento privado. Vuelve a Tiendas y sincroniza.'
read -r -p 'Pulsa Intro para cerrar...'