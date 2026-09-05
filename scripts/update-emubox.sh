#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - SCRIPT MAESTRO DE ACTUALIZACIÓN ATÓMICA Y NO DESTRUCTIVA
# ==============================================================================
#
# Flujo arquitectónico (Punto 6 y Punto 10):
#   1. Registra trazas atómicas en /var/log/emubox/update.log y terminal.
#   2. Comprueba si hay cambios locales sin confirmar antes de hacer pull.
#   3. Ejecuta git fetch origin main y compara HEAD vs origin/main.
#   4. Si no hay cambios -> Sale limpiamente informando que el sistema está al día.
#   5. Si hay cambios -> Aplica 'git pull --ff-only origin main'.
#   6. Delega la compilación a 'scripts/build.sh' (Fuente única de verdad).
#   7. Aislamiento de fallos: Si build.sh falla, EmuBox NO se reinicia y la versión
#      previa funcional permanece intacta en /opt/emubox/bin/emubox.
#   8. Si build.sh tiene éxito -> Ejecuta 'scripts/setup-autostart.sh' para
#      sincronizar lanzadores de consola y recarga la sesión en TTY1.
# ==============================================================================

set -euo pipefail

EMUBOX_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_FILE="${EMUBOX_LOG_DIR:-/var/log/emubox}/update.log"
BRANCH="main"
if [[ "$EUID" -eq 0 ]]; then
  BUILD_USER="${SUDO_USER:-emubox}"
  [[ "$BUILD_USER" != root ]] || { echo 'Update must run as a non-root user.' >&2; exit 1; }
  exec runuser -u "$BUILD_USER" -- bash "$0" "$@"
fi
source "$EMUBOX_DIR/installer/lib/architecture.sh"
BUILD_ARCH=$(get_emubox_architecture)
[[ "$BUILD_ARCH" != unsupported ]] || { echo 'Unsupported architecture' >&2; exit 1; }
export BUILD_ARCH
export TARGET="$(get_emubox_target "$BUILD_ARCH")"

# Asegurar directorio de logs
mkdir -p "$(dirname "${LOG_FILE}")" 2>/dev/null || true
if [[ ! -w "$(dirname "$LOG_FILE")" || ( -e "$LOG_FILE" && ! -w "$LOG_FILE" ) ]]; then
  LOG_FILE="$(mktemp -d "${TMPDIR:-/tmp}/emubox-update-XXXXXXXX")/update.log"
fi

# Redirigir stdout y stderr a terminal y archivo de log
exec > >(tee -a "${LOG_FILE}" 2>/dev/null || cat)
exec 2>&1

echo "======================================================================"
echo "[EmuBox Update] Proceso de Actualización: $(date '+%Y-%m-%d %H:%M:%S')"
echo "======================================================================"

if [[ ! -d "${EMUBOX_DIR}/.git" ]]; then
  echo "[ERROR] ${EMUBOX_DIR} no es un repositorio Git válido." >&2
  exit 1
fi

cd "${EMUBOX_DIR}"

# 1. Comprobar si hay cambios locales sin commitear
echo "[1/5] Verificando estado del árbol de trabajo local..."
if [[ -n "$(git status --porcelain 2>/dev/null)" ]]; then
  echo "[AVISO] Existen modificaciones locales no confirmadas en ${EMUBOX_DIR}."
  echo "[ERROR] Guarda o confirma tus cambios antes de actualizar." >&2
  exit 1
fi

# 2. Consultar repositorio remoto
echo "[2/5] Consultando actualizaciones remotas en GitHub (${BRANCH})..."
if ! git fetch origin "${BRANCH}"; then
  echo "[ERROR] No se pudo conectar con el repositorio remoto. Verifica la conexión de red." >&2
  exit 1
fi

LOCAL_COMMIT="$(git rev-parse HEAD)"
REMOTE_COMMIT="$(git rev-parse "origin/${BRANCH}")"

if [[ "${LOCAL_COMMIT}" == "${REMOTE_COMMIT}" ]]; then
  echo "----------------------------------------------------------------------"
  echo -e "\033[1;32m[OK] EmuBox ya se encuentra en la versión más reciente (${LOCAL_COMMIT:0:7}).\033[0m"
  echo "No se requieren acciones de compilación."
  echo "----------------------------------------------------------------------"
  exit 0
fi

echo "[3/5] Nuevos cambios detectados: ${LOCAL_COMMIT:0:7} -> ${REMOTE_COMMIT:0:7}"
BACKUP_DIR=$(mktemp -d "${TMPDIR:-/tmp}/emubox-update-binary-XXXXXXXX")
RESTORE_BINARY=true
restore_binary() {
  if [[ -f "$BACKUP_DIR/emubox" ]]; then
    install -m 0755 "$BACKUP_DIR/emubox" "$EMUBOX_DIR/bin/emubox.next" &&
      mv -f "$EMUBOX_DIR/bin/emubox.next" "$EMUBOX_DIR/bin/emubox"
  else
    rm -f "$EMUBOX_DIR/bin/emubox"
  fi
}
cleanup_update() {
  if [[ "$RESTORE_BINARY" == true ]]; then
    restore_binary || { echo "Binary recovery failed: $BACKUP_DIR" >&2; return 1; }
  fi
  rm -rf "$BACKUP_DIR"
}
if [[ -f "$EMUBOX_DIR/bin/emubox" ]]; then
  validate_emubox_binary "$EMUBOX_DIR/bin/emubox" "$BUILD_ARCH"
  cp -p "$EMUBOX_DIR/bin/emubox" "$BACKUP_DIR/emubox"
fi
trap cleanup_update EXIT
if ! git pull --ff-only origin "${BRANCH}"; then
  echo "----------------------------------------------------------------------" >&2
  echo "[ERROR CRÍTICO] 'git pull --ff-only' no pudo aplicarse limpiamente." >&2
  echo "[SEGURIDAD] La versión actual en ejecución NO ha sido alterada." >&2
  echo "----------------------------------------------------------------------" >&2
  exit 1
fi
restore_binary

# 3. Delegar compilación al script maestro build.sh
echo "[4/5] Compilando nueva versión mediante scripts/build.sh..."
chmod +x "${EMUBOX_DIR}/scripts/build.sh"

if ! bash "${EMUBOX_DIR}/scripts/build.sh"; then
  echo "======================================================================" >&2
  echo "[ERROR CRÍTICO] La compilación falló durante 'scripts/build.sh'." >&2
  echo "El build conserva el binario anterior hasta validar el nuevo ELF." >&2
  echo "[SEGURIDAD] La sesión activa continuará ejecutando la versión estable previa." >&2
  echo "[LOGS] Revisa los detalles del fallo en: ${LOG_FILE}" >&2
  echo "======================================================================" >&2
  exit 1
fi
validate_emubox_binary "$EMUBOX_DIR/bin/emubox" "$BUILD_ARCH"
RESTORE_BINARY=false

# 4. Sincronizar scripts de arranque y sesión
echo "[5/5] Sincronizando lanzadores de consola y entorno..."
if [[ -f "${EMUBOX_DIR}/scripts/setup-autostart.sh" ]]; then
  chmod +x "${EMUBOX_DIR}/scripts/setup-autostart.sh"
  if [[ "$EUID" -eq 0 ]]; then
    bash "${EMUBOX_DIR}/scripts/setup-autostart.sh"
  elif command -v sudo >/dev/null 2>&1; then
    sudo bash "${EMUBOX_DIR}/scripts/setup-autostart.sh" || true
  fi
fi

echo "======================================================================"
echo -e "\033[1;32m[ÉXITO] EmuBox actualizado y compilado correctamente: ${REMOTE_COMMIT:0:7}\033[0m"
echo "======================================================================"

# 5. Reinicio de sesión de consola controlado
if systemctl is-active --quiet getty@tty1 2>/dev/null; then
  echo "[INFO] Reiniciando sesión de consola en TTY1 para desplegar la nueva versión..."
  if [[ "$EUID" -eq 0 ]]; then
    systemctl restart getty@tty1 || true
  elif command -v sudo >/dev/null 2>&1; then
    sudo systemctl restart getty@tty1 || true
  else
    echo "[AVISO] Para aplicar la nueva versión en pantalla, ejecuta: sudo systemctl restart getty@tty1"
  fi
fi
