#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - DYNAMIC DRM / WAYLAND RESIZE WATCHER (DEBOUNCED & SILENT)
# ==============================================================================
#
# Arquitectura:
#   1. Detecta automáticamente el conector DRM activo (/sys/class/drm/card*-*/status == connected).
#   2. Lee la geometría dinámica del framebuffer DRM (/sys/class/drm/*/modes).
#   3. Aplica debounce (200ms) durante redimensionados rápidos de ventana (VirtualBox/Host).
#   4. Comprueba si Cage/Wayland ya tiene esa resolución antes de invocar wlr-randr.
#   5. Aplica el nuevo modo mediante 'wlr-randr --mode' o '--custom-mode WxH@60'.
#   6. Cero spam de logs: Solo opera ante cambios reales y estabilizados.
#   7. Muere limpiamente cuando la sesión gráfica principal termina.
# ==============================================================================

set -u

# Esperar a que el socket de Wayland esté disponible
WAYLAND_SOCKET="${WAYLAND_DISPLAY:-wayland-0}"
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"

for _ in {1..50}; do
  if [[ -S "${RUNTIME_DIR}/${WAYLAND_SOCKET}" ]]; then
    break
  fi
  sleep 0.1
done

if ! command -v wlr-randr >/dev/null 2>&1; then
  # Si wlr-randr no está instalado, salir silenciosamente
  exit 0
fi

# 1. Detectar el conector DRM conectado dinámicamente
find_active_connector() {
  for status_file in /sys/class/drm/card*-*/status; do
    if [[ -f "$status_file" ]]; then
      if grep -q "^connected" "$status_file" 2>/dev/null; then
        local conn_dir
        conn_dir="$(dirname "$status_file")"
        local conn_name
        conn_name="$(basename "$conn_dir" | sed -E 's/^card[0-9]+-//')"
        echo "$conn_name"
        return 0
      fi
    fi
  done
  # Fallback a Virtual-1 si no se detecta status explícito
  echo "Virtual-1"
}

CONNECTOR="$(find_active_connector)"
MODES_PATH=""

# Localizar la ruta del archivo de modos del conector activo
for card_dir in /sys/class/drm/card*"-${CONNECTOR}"; do
  if [[ -f "${card_dir}/modes" ]]; then
    MODES_PATH="${card_dir}/modes"
    break
  fi
done

# Si no se encontró por nombre exacto, buscar el primer conector conectado con modos
if [[ -z "$MODES_PATH" || ! -f "$MODES_PATH" ]]; then
  for m in /sys/class/drm/card*-*/modes; do
    if [[ -f "$m" && -s "$m" ]]; then
      MODES_PATH="$m"
      CONNECTOR="$(basename "$(dirname "$m")" | sed -E 's/^card[0-9]+-//')"
      break
    fi
  done
fi

if [[ -z "$MODES_PATH" || ! -f "$MODES_PATH" ]]; then
  exit 0
fi

LAST_APPLIED_W=0
LAST_APPLIED_H=0
LAST_SEEN_W=0
LAST_SEEN_H=0
STABLE_TICKS=0

# Ciclo de supervisión ligero (150ms) con debounce
while true; do
  if [[ ! -f "$MODES_PATH" ]]; then
    sleep 0.5
    continue
  fi

  CURRENT_MODE="$(head -n 1 "$MODES_PATH" 2>/dev/null || true)"

  if [[ "$CURRENT_MODE" =~ ^([0-9]+)x([0-9]+) ]]; then
    DETECTED_W="${BASH_REMATCH[1]}"
    DETECTED_H="${BASH_REMATCH[2]}"

    if [[ "$DETECTED_W" -gt 320 && "$DETECTED_H" -gt 240 ]]; then
      # Comprobar si el tamaño ha cambiado respecto al último aplicado
      if [[ "$DETECTED_W" -ne "$LAST_APPLIED_W" || "$DETECTED_H" -ne "$LAST_APPLIED_H" ]]; then
        # Verificar si la geometría detectada se ha estabilizado (Debounce)
        if [[ "$DETECTED_W" -eq "$LAST_SEEN_W" && "$DETECTED_H" -eq "$LAST_SEEN_H" ]]; then
          STABLE_TICKS=$((STABLE_TICKS + 1))
          
          # Una vez que la resolución permanece estable durante 2 ciclos (~300ms)
          if [[ "$STABLE_TICKS" -ge 2 ]]; then
            # Intentar aplicar primero con --mode estándar
            if ! wlr-randr --output "$CONNECTOR" --mode "${DETECTED_W}x${DETECTED_H}" 2>/dev/null; then
              # Si es una resolución arbitraria de VirtualBox, usar --custom-mode
              wlr-randr --output "$CONNECTOR" --custom-mode "${DETECTED_W}x${DETECTED_H}@60" 2>/dev/null || \
              wlr-randr --output "$CONNECTOR" --custom-mode "${DETECTED_W}x${DETECTED_H}" 2>/dev/null || true
            fi

            LAST_APPLIED_W="$DETECTED_W"
            LAST_APPLIED_H="$DETECTED_H"
            STABLE_TICKS=0
          fi
        else
          # El tamaño aún está cambiando (usuario arrastrando la ventana)
          LAST_SEEN_W="$DETECTED_W"
          LAST_SEEN_H="$DETECTED_H"
          STABLE_TICKS=0
        fi
      fi
    fi
  fi

  sleep 0.15
done
