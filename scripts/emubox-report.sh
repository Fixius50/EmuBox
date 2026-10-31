#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - REPORTE AUTOMATICO DE DIAGNOSTICO Y ERRORES A GITHUB
# ==============================================================================

set -euo pipefail

REPO="Fixius50/EmuBox"
REPORTS_DIR="/var/log/emubox/reports"
mkdir -p "$REPORTS_DIR" 2>/dev/null || REPORTS_DIR="/tmp/emubox-reports"
mkdir -p "$REPORTS_DIR"

TIMESTAMP="$(date '+%Y-%m-%d %H:%M:%S UTC')"
FILE_TS="$(date '+%Y%m%d-%H%M%S')"
REPORT_FILE="${REPORTS_DIR}/report-${FILE_TS}.md"

echo "======================================================================"
echo "         📦 EmuBox - Generador de Reporte de Diagnostico             "
echo "======================================================================"
echo "Recopilando telemetria, logs del sistema y estado de hardware..."

ERROR_SUMMARY="${1:-Fallo durante la ejecucion de EmuBox}"

CPU_INFO="$(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2 | xargs || uname -m)"
MEM_INFO="$(free -h 2>/dev/null | awk '/^Mem:/ {print $2}' || echo 'Desconocida')"
KERNEL="$(uname -r)"

cat << EOF > "$REPORT_FILE"
### Reporte de Error - EmuBox Appliance
**Fecha:** ${TIMESTAMP}
**Incidencia:** \`${ERROR_SUMMARY}\`

#### Informacion del Sistema
- **Kernel:** ${KERNEL}
- **CPU:** ${CPU_INFO}
- **RAM:** ${MEM_INFO}
- **Virtualizacion:** $(systemd-detect-virt 2>/dev/null || echo "Desconocida")

#### Trazas recientes de arranque (/var/log/emubox/session.log)
\`\`\`text
$(tail -n 40 /var/log/emubox/session.log 2>/dev/null || echo "Sin session.log")
\`\`\`

#### Ultimos eventos de telemetria (/var/log/emubox/telemetry.jsonl)
\`\`\`json
$(tail -n 15 /var/log/emubox/telemetry.jsonl 2>/dev/null || echo "Sin telemetria")
\`\`\`
EOF

echo "✓ Reporte guardado localmente en: ${REPORT_FILE}"

TOKEN="${GITHUB_TOKEN:-${EMUBOX_REPORT_TOKEN:-}}"
if [[ -z "$TOKEN" && -f /etc/emubox/github-token ]]; then
  TOKEN="$(< /etc/emubox/github-token)"
fi

if [[ -n "$TOKEN" ]]; then
  echo "Enviando reporte automaticamente a https://api.github.com/repos/${REPO}/issues..."
  JSON_PAYLOAD=$(node -e "
    const fs = require('fs');
    const body = fs.readFileSync(process.argv[1], 'utf8');
    console.log(JSON.stringify({
      title: '[Auto-Report] ' + process.argv[2],
      body: body,
      labels: ['bug', 'automated-report']
    }));
  " "$REPORT_FILE" "$ERROR_SUMMARY" 2>/dev/null || true)

  if [[ -n "$JSON_PAYLOAD" ]]; then
    RESPONSE=$(curl -s -X POST \
      -H "Authorization: Bearer ${TOKEN}" \
      -H "Accept: application/vnd.github+json" \
      -H "User-Agent: EmuBox-Appliance" \
      https://api.github.com/repos/${REPO}/issues \
      -d "$JSON_PAYLOAD")

    ISSUE_URL=$(echo "$RESPONSE" | grep -o 'https://github.com/Fixius50/EmuBox/issues/[0-9]*' | head -n 1 || true)
    if [[ -n "$ISSUE_URL" ]]; then
      echo -e "\033[1;32m✓ Reporte publicado con exito en GitHub: ${ISSUE_URL}\033[0m"
      exit 0
    else
      echo "[AVISO] No se pudo crear el Issue directo en GitHub. Respuesta de la API:"
      echo "$RESPONSE" | head -n 5
    fi
  fi
fi

echo ""
echo "Para enviar el reporte al repositorio:"
echo "1. El reporte completo esta en: ${REPORT_FILE}"
echo "2. Puedes guardar un Personal Access Token de GitHub en /etc/emubox/github-token para que se envie 100% solo."
echo "3. O abre una nueva incidencia en: https://github.com/${REPO}/issues/new"
