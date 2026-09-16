#!/usr/bin/env bash

configure_emubox_host_logging() {
  if [[ "$1" == oracle && ! -v SVGA_NO_LOGGING ]]; then
    export SVGA_NO_LOGGING=1
  fi
}

configure_emubox_cage_command() {
  CAGE_COMMAND=(cage)
  [[ "$1" == vmwgfx && "${EMUBOX_VMWGFX_COMPAT:-1}" == 1 ]] || return 0
  local library="$2/bin/libemubox-vmwgfx.so" loader=''
  case "$3" in
    x86_64) loader=/lib64/ld-linux-x86-64.so.2 ;;
    aarch64) loader=/lib/ld-linux-aarch64.so.1 ;;
  esac
  if [[ -r "$library" && -x "$loader" ]]; then
    CAGE_COMMAND=("$loader" --preload "$library" "$(command -v cage)")
  else
    printf '[AVISO] Adaptador vmwgfx no disponible; Cage usara libdrm del sistema.\n' >&2
  fi
}

select_emubox_compositor() {
  local preference="$1" backend="$2" drm="$3" cage="$4" gamescope_ready="$5"
  if [[ "$drm" != 1 ]]; then echo unavailable
  elif [[ "$preference" == gamescope && "$backend" != software && "$gamescope_ready" == 1 ]]; then echo gamescope
  elif [[ "$cage" == 1 && ( "$backend" == opengl || "$backend" == software || "$backend" == auto ) ]]; then echo cage
  elif [[ "$backend" != software && "$gamescope_ready" == 1 ]]; then echo gamescope
  else echo unavailable; fi
}

configure_emubox_cursor() {
  if [[ "$1" == vmwgfx && ! -v WLR_NO_HARDWARE_CURSORS ]]; then
    export WLR_NO_HARDWARE_CURSORS=1
  fi
  if [[ "$1" == vmwgfx && ! -v WLR_DRM_NO_ATOMIC ]]; then
    export WLR_DRM_NO_ATOMIC=1
  fi
}

configure_emubox_presentation() {
  [[ "$1" == vmwgfx ]] || return 0
  if [[ ! -v WLR_SCENE_DEBUG_DAMAGE ]]; then
    export WLR_SCENE_DEBUG_DAMAGE=rerender
  fi
  if [[ ! -v WLR_SCENE_DISABLE_DIRECT_SCANOUT ]]; then
    export WLR_SCENE_DISABLE_DIRECT_SCANOUT=1
  fi
}

configure_emubox_webkit() {
  [[ "$1" == vmwgfx ]] || return 0
  if [[ ! -v WEBKIT_DISABLE_DMABUF_RENDERER ]]; then
    export WEBKIT_DISABLE_DMABUF_RENDERER=0
  fi
  if [[ "$WEBKIT_DISABLE_DMABUF_RENDERER" == 0 && ! -v WEBKIT_DMABUF_RENDERER_FORCE_SHM ]]; then
    export WEBKIT_DMABUF_RENDERER_FORCE_SHM=1
  fi
}

configure_emubox_render_mode() {
  case "$1" in
    auto)
      unset LIBGL_ALWAYS_SOFTWARE WLR_RENDERER WEBKIT_DISABLE_COMPOSITING_MODE
      if emubox_software_renderer "${GALLIUM_DRIVER:-}"; then unset GALLIUM_DRIVER; fi
      if emubox_software_renderer "${MESA_LOADER_DRIVER_OVERRIDE:-}"; then unset MESA_LOADER_DRIVER_OVERRIDE; fi
      ;;
    software)
      export LIBGL_ALWAYS_SOFTWARE=1
      export WLR_RENDERER=pixman
      export WEBKIT_DISABLE_COMPOSITING_MODE=1
      ;;
    *) printf '[ERROR] Modo grafico no valido: usa auto o software\n' >&2; return 1 ;;
  esac
}

select_emubox_render_mode() {
  case "$1" in
    auto|software)
      if [[ "$2" == 1 || "$3" == 1 ]]; then printf '%s\n' auto
      elif [[ "${4:-indeterminate}" == software || "$1" == software ]]; then printf '%s\n' software
      else printf '%s\n' auto; fi
      ;;
    *) printf '[ERROR] Modo grafico no valido: usa auto o software\n' >&2; return 1 ;;
  esac
}

emubox_software_renderer() {
  case "${1,,}" in
    *llvmpipe*|*softpipe*|*swrast*|*'software rasterizer'*|*swiftshader*|*lavapipe*|*'microsoft basic render'*|*'gdi generic'*|*'mesa software'*) return 0 ;;
    *) return 1 ;;
  esac
}

detect_emubox_graphics() {
  GPU_VENDOR=unknown
  GPU_DRIVER=unknown
  GPU_DEVICE=unknown
  RENDERER_DESC=unknown
  HAS_HW_VULKAN=0
  HAS_OPENGL=0
  HAS_HW_OPENGL=0
  OPENGL_RENDERER=unknown
  HAS_DRM=0
  HAS_GAMESCOPE=0
  HAS_CAGE=0
  GAMESCOPE_READY=0
  HAS_ACCELERATION=0
  IS_VIRTUAL_MACHINE=0
  GRAPHICS_DETECTION_STATE=indeterminate
  GRAPHICS_PROBE_REASONS=native_probe_unavailable
  GRAPHICS_BACKEND=auto
  GRAPHICS_OPERATIONAL_BACKEND=auto
  GRAPHICS_FALLBACK_REASON=''
  GPU_KIND=unknown
  GPU_ACTIVE_DEVICE=unknown
  DEVICE_MODEL=unknown
  EMUBOX_COMPOSITOR=unavailable
  local root binary output key value card
  root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
  binary="${EMUBOX_BIN:-$root/bin/emubox}"
  if [[ -x "$binary" ]] && output=$(timeout 30s "$binary" --graphics-session) && [[ "$output" == GRAPHICS_DETECTION_STATE=* ]]; then
    while IFS='=' read -r key value; do
      case "$key" in
        GRAPHICS_DETECTION_STATE|GRAPHICS_PROBE_REASONS|GRAPHICS_BACKEND|GRAPHICS_OPERATIONAL_BACKEND|GRAPHICS_FALLBACK_REASON|GPU_VENDOR|GPU_DRIVER|GPU_DEVICE|GPU_ACTIVE_DEVICE|GPU_KIND|RENDERER_DESC|DEVICE_MODEL|HAS_HW_VULKAN|HAS_OPENGL|HAS_HW_OPENGL|OPENGL_RENDERER|HAS_ACCELERATION|HAS_DRM|HAS_CAGE|HAS_GAMESCOPE|GAMESCOPE_READY|IS_VIRTUAL_MACHINE|EMUBOX_COMPOSITOR)
          printf -v "$key" '%s' "$value" ;;
      esac
    done <<< "$output"
  else
    for card in /dev/dri/card[0-9]*; do [[ ! -e "$card" ]] || HAS_DRM=1; done
    command -v cage >/dev/null 2>&1 && HAS_CAGE=1
    [[ "${EMUBOX_RENDER_MODE:-auto}" != software ]] || { GRAPHICS_OPERATIONAL_BACKEND=software; GRAPHICS_FALLBACK_REASON=explicit_software_fallback; }
    EMUBOX_COMPOSITOR=$(select_emubox_compositor auto "$GRAPHICS_OPERATIONAL_BACKEND" "$HAS_DRM" "$HAS_CAGE" 0)
  fi
}