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

EMUBOX_DIR="/opt/emubox"
LOG_FILE="/var/log/emubox/update.log"
BRANCH="main"

# Asegurar directorio de logs
mkdir -p "$(dirname "${LOG_FILE}")" 2>/dev/null || true

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
  echo "[AVISO] Guardando cambios temporales con 'git stash'..."
  git stash save "Auto-stash antes de update $(date '+%Y-%m-%d %H:%M:%S')" || true
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
if ! git pull --ff-only origin "${BRANCH}"; then
  echo "----------------------------------------------------------------------" >&2
  echo "[ERROR CRÍTICO] 'git pull --ff-only' no pudo aplicarse limpiamente." >&2
  echo "[SEGURIDAD] La versión actual en ejecución NO ha sido alterada." >&2
  echo "----------------------------------------------------------------------" >&2
# Guardar copia de seguridad del commit y binario actuales antes de compilar
PREV_COMMIT="${LOCAL_COMMIT}"
if [[ -f "${EMUBOX_DIR}/bin/emubox" ]]; then
  cp -f "${EMUBOX_DIR}/bin/emubox" "${EMUBOX_DIR}/bin/emubox.backup" 2>/dev/null || true
fi

# 3. Delegar compilación al script maestro build.sh
echo "[4/5] Compilando nueva versión mediante scripts/build.sh..."
if command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
  sudo chown -R "$(id -u):$(id -g)" "${EMUBOX_DIR}" 2>/dev/null || true
fi
chmod +x "${EMUBOX_DIR}/scripts/build.sh"

if ! bash "${EMUBOX_DIR}/scripts/build.sh"; then
  echo "======================================================================" >&2
  echo "[ERROR CRÍTICO] La compilación falló durante 'scripts/build.sh'." >&2
  echo "[ROLLBACK INTELIGENTE] Revirtiendo código al commit previo funcional (${PREV_COMMIT:0:7})..." >&2
  git reset --hard "${PREV_COMMIT}" || true
  if [[ -f "${EMUBOX_DIR}/bin/emubox.backup" ]]; then
    cp -f "${EMUBOX_DIR}/bin/emubox.backup" "${EMUBOX_DIR}/bin/emubox" 2>/dev/null || true
    chmod +x "${EMUBOX_DIR}/bin/emubox" 2>/dev/null || true
    echo "[ROLLBACK INTELIGENTE] Binario anterior restaurado con éxito." >&2
  fi
  echo "[SEGURIDAD] La sesión activa continuará ejecutando la versión estable previa." >&2
  echo "[LOGS] Revisa los detalles del fallo en: ${LOG_FILE}" >&2
  echo "======================================================================" >&2
  exit 1
fi

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
