#!/usr/bin/env bash
set -u

ROOT=/var/lib/emubox/stores/gog
PYTHON="${ROOT}/runtime/bin/python"

umask 077
mkdir -p "${ROOT}/home" "${ROOT}/config"
chmod 700 "${ROOT}" "${ROOT}/home" "${ROOT}/config"

if [[ ! -x "${PYTHON}" ]]; then
  printf '%s\n' 'El adaptador GOG no esta instalado.' >&2
  printf '%s\n' 'Ejecuta como administrador: /opt/emubox/scripts/setup-store-adapters.sh gog' >&2
  read -r -p 'Pulsa Intro para cerrar...'
  exit 1
fi

export HOME="${ROOT}/home"
export XDG_CONFIG_HOME="${ROOT}/config"
export GOG_AUTH_FILE="${ROOT}/auth.json"

"${PYTHON}" - <<'PY'
import contextlib
import io
import os
import sys
import webbrowser
from types import SimpleNamespace
from urllib.parse import parse_qs, quote, urlparse

from gogdl.auth import AuthorizationManager, CLIENT_ID, CODE_URL

query = parse_qs(urlparse(CODE_URL).query)
redirect = query["redirect_uri"][0]
url = "https://auth.gog.com/auth?client_id={}&redirect_uri={}&response_type=code".format(
    quote(CLIENT_ID, safe=""), quote(redirect, safe="")
)
print("Abriendo el inicio de sesion de GOG en el navegador...")
webbrowser.open(url)
code = input("Pega el codigo de autorizacion de GOG y pulsa Intro: ").strip()
if not code:
    raise SystemExit("No se recibio codigo de autorizacion.")
manager = AuthorizationManager(os.environ["GOG_AUTH_FILE"])
arguments = SimpleNamespace(authorization_code=code, client_id=None, client_secret=None)
with contextlib.redirect_stdout(io.StringIO()):
    manager.handle_cli(arguments, [])
print("GOG ha guardado la sesion privada. Vuelve a Tiendas y sincroniza.")
PY
status=$?
read -r -p 'Pulsa Intro para cerrar...'
exit "${status}"