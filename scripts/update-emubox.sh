#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - SCRIPT MAESTRO DE ACTUALIZACIÓN ATÓMICA (UPDATE ORCHESTRATOR)
# ==============================================================================
#
# Flujo arquitectónico:
#   1. Redirige trazas a /var/log/emubox/update.log y terminal.
#   2. Comprueba conectividad con GitHub (Fixius50/EmuBox:main).
#   3. Ejecuta git fetch y evalúa si hay cambios pendientes (HEAD vs origin/main).
#   4. Si no hay cambios -> Sale limpiamente informando que el sistema está al día.
#   5. Si hay cambios -> Realiza 'git pull --ff-only origin main'.
#   6. Delega la compilación a 'scripts/build.sh' (Fuente única de verdad).
#   7. Protección de fallo: Si build.sh falla, EmuBox NO se reinicia y la versión
#      previa funcional permanece intacta.
#   8. Si build.sh tiene éxito -> Ejecuta 'scripts/setup-autostart.sh' para
#      sincronizar lanzadores de consola y reinicia la sesión Cage de forma segura.
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
echo "[EmuBox Update] Iniciando proceso de actualización: $(date '+%Y-%m-%d %H:%M:%S')"
echo "======================================================================"

if [[ ! -d "${EMUBOX_DIR}/.git" ]]; then
  echo "[ERROR] ${EMUBOX_DIR} no es un repositorio Git válido." >&2
  exit 1
fi

cd "${EMUBOX_DIR}"

# 1. Comprobar remoto y rama
echo "[1/4] Consultando actualizaciones remotas en GitHub (${BRANCH})..."
if ! git fetch origin "${BRANCH}"; then
  echo "[ERROR] No se pudo conectar con el repositorio remoto. Verifica tu conexión de red." >&2
  exit 1
fi

LOCAL_COMMIT="$(git rev-parse HEAD)"
REMOTE_COMMIT="$(git rev-parse "origin/${BRANCH}")"

if [[ "${LOCAL_COMMIT}" == "${REMOTE_COMMIT}" ]]; then
  echo "[OK] EmuBox ya se encuentra en la versión más reciente (${LOCAL_COMMIT:0:7})."
  echo "No se requieren acciones de compilación."
  exit 0
fi

echo "[2/4] Nuevos cambios detectados (${LOCAL_COMMIT:0:7} -> ${REMOTE_COMMIT:0:7}). Aplicando pull..."
if ! git pull --ff-only origin "${BRANCH}"; then
  echo "[ERROR] 'git pull --ff-only' falló. Puede haber cambios locales sin commitear." >&2
  echo "[INFO] La versión actual en ejecución NO ha sido alterada." >&2
  exit 1
fi

# 2. Delegar compilación al script maestro build.sh
echo "[3/4] Compilando nueva versión mediante scripts/build.sh..."
chmod +x "${EMUBOX_DIR}/scripts/build.sh"

if ! bash "${EMUBOX_DIR}/scripts/build.sh"; then
  echo "======================================================================" >&2
  echo "[ERROR CRÍTICO] La compilación falló durante 'scripts/build.sh'." >&2
  echo "[SEGURIDAD] La sesión activa NO será reiniciada para evitar caídas de la interfaz." >&2
  echo "[LOGS] Revisa los detalles en ${LOG_FILE}" >&2
  echo "======================================================================" >&2
  exit 1
fi

# 3. Sincronizar scripts de arranque y sesión
echo "[4/4] Sincronizando lanzadores de consola y entorno..."
if [[ -f "${EMUBOX_DIR}/scripts/setup-autostart.sh" ]]; then
  chmod +x "${EMUBOX_DIR}/scripts/setup-autostart.sh"
  if [[ "$EUID" -eq 0 ]]; then
    bash "${EMUBOX_DIR}/scripts/setup-autostart.sh"
  elif command -v sudo >/dev/null 2>&1; then
    sudo bash "${EMUBOX_DIR}/scripts/setup-autostart.sh" || true
  fi
fi

echo "======================================================================"
echo "[ÉXITO] EmuBox actualizado y compilado correctamente: ${REMOTE_COMMIT:0:7}"
echo "======================================================================"

# 4. Reinicio de sesión controlado si se ejecuta como root / systemd
if [[ "$EUID" -eq 0 ]] && systemctl is-active --quiet getty@tty1; then
  echo "[INFO] Reiniciando sesión de consola en TTY1 para cargar la nueva versión..."
  systemctl restart getty@tty1 || true
fi
