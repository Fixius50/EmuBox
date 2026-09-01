#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - EVENT-DRIVEN DRM TO WAYLAND SYNC (0% CPU, ZERO POLLING)
# ==============================================================================
#
# Arquitectura:
#   1. Escucha exclusivamente eventos udev del kernel (SUBSYSTEM=drm, HOTPLUG=1).
#   2. Cero consumo de CPU en reposo (bloqueado en el socket de eventos).
#   3. Detección agnóstica de hardware: busca cualquier conector conectado en
#      cualquier tarjeta gráfica (/sys/class/drm/card*-*/status == connected).
#   4. Sincroniza la salida en Cage/wlroots mediante wlr-randr en la sesión Wayland.
#   5. Funciona idénticamente en VMs (VirtualBox/KVM/VMware) y hardware físico (AMD/Intel/NVIDIA).
# ==============================================================================

set -u

# 1. Asegurar entorno de la sesión Wayland
WAYLAND_SOCKET="${WAYLAND_DISPLAY:-wayland-0}"
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"

export XDG_RUNTIME_DIR="${RUNTIME_DIR}"
export WAYLAND_DISPLAY="${WAYLAND_SOCKET}"

for _ in {1..50}; do
  if [[ -S "${RUNTIME_DIR}/${WAYLAND_SOCKET}" ]]; then
    break
  fi
  sleep 0.1
done

if ! command -v wlr-randr >/dev/null 2>&1; then
  exit 0
fi

# 2. Localizar dinámicamente el conector DRM conectado y su ruta de modos (sin asumir card0)
find_active_drm_output() {
  local status_file
  for status_file in /sys/class/drm/card*-*/status; do
    if [[ -f "$status_file" ]]; then
      if grep -q "^connected" "$status_file" 2>/dev/null; then
        local conn_dir
        conn_dir="$(dirname "$status_file")"
        local conn_name
        conn_name="$(basename "$conn_dir" | sed -E 's/^card[0-9]+-//')"
        local modes_file="${conn_dir}/modes"
        
        if [[ -f "$modes_file" && -s "$modes_file" ]]; then
          echo "${conn_name}:${modes_file}"
          return 0
        fi
      fi
    fi
  done

  # Fallback si ningún conector tiene status 'connected' explícito
  local any_modes
  any_modes="$(ls /sys/class/drm/card*-*/modes 2>/dev/null | head -n 1 || true)"
  if [[ -n "$any_modes" && -f "$any_modes" ]]; then
    local conn_name
    conn_name="$(basename "$(dirname "$any_modes")" | sed -E 's/^card[0-9]+-//')"
    echo "${conn_name}:${any_modes}"
    return 0
  fi

  echo "Virtual-1:/sys/class/drm/card0-Virtual-1/modes"
}

LAST_APPLIED=""

sync_drm_to_wayland() {
  local output_info
  output_info="$(find_active_drm_output)"
  
  local connector="${output_info%%:*}"
  local modes_file="${output_info#*:}"

  if [[ -f "$modes_file" && -s "$modes_file" ]]; then
    local current_mode
    current_mode="$(head -n 1 "$modes_file" 2>/dev/null || true)"

    if [[ "$current_mode" =~ ^([0-9]+)x([0-9]+) ]]; then
      local w="${BASH_REMATCH[1]}"
      local h="${BASH_REMATCH[2]}"
      local target="${w}x${h}"

      if [[ "$target" != "$LAST_APPLIED" && "$w" -gt 320 && "$h" -gt 240 ]]; then
        # 1. Intentar aplicar como modo personalizado (resoluciones arbitrarias de VM)
        if ! wlr-randr --output "$connector" --custom-mode "${w}x${h}@60" 2>/dev/null; then
          # 2. Fallback a modo estándar (hardware físico / monitores fijos)
          wlr-randr --output "$connector" --mode "${target}" 2>/dev/null || \
          wlr-randr --output "$connector" --preferred 2>/dev/null || true
        fi
        LAST_APPLIED="$target"
      fi
    fi
  fi
}

# Sincronización inicial al levantar la sesión
sync_drm_to_wayland

# 3. Bucle reactivo puro bloqueado en el socket de udev (0% CPU)
udevadm monitor --property --subsystem-match=drm 2>/dev/null | while read -r line; do
  if [[ "$line" == "HOTPLUG=1" ]]; then
    # Margen breve (100ms) para que sysfs estabilice los descriptores tras el evento del kernel
    sleep 0.1
    sync_drm_to_wayland
  fi
done
