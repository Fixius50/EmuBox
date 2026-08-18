#!/usr/bin/env bash
# EmuBox Benchmarking Suite Runner for Arch Linux / Linux WebKitGTK
set -e

echo "========================================================"
echo "    EMUBOX FRONTEND BENCHMARKING LAB (LINUX TARGET)    "
echo "========================================================"

CANDIDATES=("solid" "svelte" "react" "vue" "next" "astro")
DATASET_SIZES=(20 100 500)

RESULTS_DIR="./benchmarks/results"
mkdir -p "$RESULTS_DIR"

echo "Verificando dependencias..."
node -v
npm -v

for FRAMEWORK in "${CANDIDATES[@]}"; do
  echo "--------------------------------------------------------"
  echo "Compilando y midiendo bundle: $FRAMEWORK..."
  if [ -d "$FRAMEWORK" ]; then
    cd "$FRAMEWORK"
    if [ ! -d "node_modules" ]; then
      echo "Instalando dependencias de $FRAMEWORK..."
      npm install --silent
    fi
    echo "Ejecutando build de producción..."
    npm run build
    
    # Medir payload de dist
    DIST_SIZE_KB=$(du -sk dist 2>/dev/null | cut -f1 || du -sk .next 2>/dev/null | cut -f1 || echo "0")
    echo "Tamaño de dist: ${DIST_SIZE_KB} KB"
    cd ..
  fi
done

echo "========================================================"
echo "Compilaciones completadas. Listo para ejecución de tests."
