#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - SCRIPT DE DIAGNOSTICO DE LOGS Y PRUEBA DIRECTA DE NPM
# ==============================================================================

set -euo pipefail

EMUBOX_DIR="/opt/emubox"
if [[ ! -d "${EMUBOX_DIR}" ]]; then
  EMUBOX_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi

echo "==============================================================================="
echo "  EMUBOX: DIAGNOSTICO DE ENTORNO Y REGISTRO DE ERRORES DE NPM"
echo "==============================================================================="
echo "Directorio de trabajo: ${EMUBOX_DIR}"
echo "Node version:          $(node --version 2>/dev/null || echo 'No disponible')"
echo "npm version:           $(npm --version 2>/dev/null || echo 'No disponible')"
echo "==============================================================================="
echo ""

echo "--- [1] Buscando 'npm error' en /var/log/emubox/npm-install.log ---"
if [[ -f /var/log/emubox/npm-install.log ]]; then
  grep -n -C 5 "npm error" /var/log/emubox/npm-install.log || echo "[OK] Sin coincidencias de 'npm error' en npm-install.log"
else
  echo "[AVISO] /var/log/emubox/npm-install.log no existe todavia."
fi
echo ""

echo "--- [2] Buscando 'npm error' en /var/log/emubox/npm-build.log ---"
if [[ -f /var/log/emubox/npm-build.log ]]; then
  grep -n -C 5 "npm error" /var/log/emubox/npm-build.log || echo "[OK] Sin coincidencias de 'npm error' en npm-build.log"
else
  echo "[AVISO] /var/log/emubox/npm-build.log no existe todavia."
fi
echo ""

echo "--- [3] Buscando 'npm error' en /var/log/emubox/npm-esbuild.log ---"
if [[ -f /var/log/emubox/npm-esbuild.log ]]; then
  grep -n -C 5 "npm error" /var/log/emubox/npm-esbuild.log || echo "[OK] Sin coincidencias de 'npm error' en npm-esbuild.log"
else
  echo "[AVISO] /var/log/emubox/npm-esbuild.log no existe todavia."
fi
echo ""

echo "--- [4] Ejecutando prueba directa de 'npm ci' en /tmp/emubox-npm-test.log ---"
cd "${EMUBOX_DIR}"
TEST_LOG="/tmp/emubox-npm-test.log"
: > "${TEST_LOG}"

set +e
npm ci --no-audit --no-fund > "${TEST_LOG}" 2>&1
TEST_EXIT_CODE=$?
set -e

echo "CODIGO DE SALIDA DE NPM CI: ${TEST_EXIT_CODE}"
echo ""
echo "--- Ultimas 50 lineas de la prueba de npm ci (/tmp/emubox-npm-test.log) ---"
tail -n 50 "${TEST_LOG}"
echo "==============================================================================="
