#!/usr/bin/env bash

emubox_gpu_vendor() {
  local text=" ${1,,} "
  case "$text" in
    *vmware*|*vmwgfx*|*virtualbox*|*virtio*|*qxl*|*virgl*|*svga3d*|*venus*) echo virtual ;;
    *amdgpu*|*radeon*|*' amd '*|*'1002:'*) echo amd ;;
    *nvidia*|*nouveau*|*'10de:'*) echo nvidia ;;
    *intel*|*i915*|*' xe '*|*'8086:'*) echo intel ;;
    *broadcom*|*v3d*|*vc4*) echo broadcom ;;
    *mali*|*panfrost*|*panthor*|*lima*) echo mali ;;
    *qualcomm*|*adreno*|*freedreno*|*' msm '*) echo qualcomm ;;
    *apple*|*asahi*|*' agx '*) echo apple ;;
    *) echo unknown ;;
  esac
}

emubox_vulkan_renderer() {
  awk '
    function usable() { return hardware && name != "" && tolower(name) !~ /llvmpipe|softpipe|swrast|software rasterizer|swiftshader|lavapipe|microsoft basic render|gdi generic|mesa software/ }
    function emit() { if (usable()) { print name; found=1; exit } }
    /^[[:space:]]*GPU[0-9]+:/ { emit(); hardware=0; name="" }
    /deviceType[[:space:]]*=/ { hardware=($0 ~ /DISCRETE_GPU|INTEGRATED_GPU|VIRTUAL_GPU/) }
    /deviceName[[:space:]]*=/ { sub(/^[^=]*=[[:space:]]*/, ""); name=$0 }
    END { if (!found && usable()) print name }
  '
}

select_emubox_compositor() {
  local preference="$1" backend="$2" drm="$3" cage="$4" gamescope_ready="$5"
  if [[ "$drm" != 1 ]]; then echo unavailable
  elif [[ "$preference" == gamescope && "$backend" != software && "$gamescope_ready" == 1 ]]; then echo gamescope
  elif [[ "$cage" == 1 && ( "$backend" == opengl || "$backend" == software ) ]]; then echo cage
  elif [[ "$backend" != software && "$gamescope_ready" == 1 ]]; then echo gamescope
  else echo unavailable; fi
}

select_emubox_backend() {
  if [[ "$2" == 1 ]]; then echo opengl
  elif [[ "$1" == 1 ]]; then echo vulkan
  else echo software; fi
}

configure_emubox_cursor() {
  if [[ "$1" == vmwgfx && ! -v WLR_NO_HARDWARE_CURSORS ]]; then
    export WLR_NO_HARDWARE_CURSORS=1
  fi
  if [[ "$1" == vmwgfx && ! -v WLR_DRM_NO_ATOMIC ]]; then
    export WLR_DRM_NO_ATOMIC=1
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
      if [[ "$2" == 1 || "$3" == 1 ]]; then printf '%s\n' auto; else printf '%s\n' software; fi
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

emubox_opengl_renderer() {
  awk '
    /OpenGL (core profile renderer|compatibility profile renderer|ES profile renderer|renderer string):/ {
      sub(/^[^:]*:[[:space:]]*/, "")
      if ($0 == "") next
      if (first == "") first=$0
      if (tolower($0) !~ /llvmpipe|softpipe|swrast|software rasterizer|swiftshader|lavapipe|microsoft basic render|gdi generic|mesa software/) { print; found=1; exit }
    }
    END { if (!found && first != "") print first }
  '
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
  local card driver info renderer
  for card in /sys/class/drm/card[0-9]*; do
    [[ "$(basename "$card")" != *-* && -e "$card/device" ]] || continue
    [[ -e "/dev/dri/$(basename "$card")" ]] && HAS_DRM=1
    driver=$(basename "$(readlink -f "$card/device/driver" 2>/dev/null)" 2>/dev/null || true)
    info=$(cat "$card/device/uevent" 2>/dev/null || true)
    if [[ "$GPU_VENDOR" == unknown ]]; then
      GPU_VENDOR=$(emubox_gpu_vendor "$driver $info")
      GPU_DRIVER="${driver:-unknown}"
      GPU_DEVICE="$(basename "$card")"
      RENDERER_DESC="$GPU_DRIVER"
    fi
  done
  if command -v vulkaninfo >/dev/null 2>&1; then
    info=$(LC_ALL=C timeout 10s vulkaninfo --summary 2>/dev/null) || info=''
    renderer=$(printf '%s\n' "$info" | emubox_vulkan_renderer)
    if [[ -n "$renderer" ]]; then
      HAS_HW_VULKAN=1
      RENDERER_DESC="$renderer"
      local vendor
      vendor=$(emubox_gpu_vendor "$renderer")
      [[ "$vendor" == unknown ]] || GPU_VENDOR="$vendor"
    fi
  fi
  if command -v eglinfo >/dev/null 2>&1; then
    info=$(LC_ALL=C timeout 10s eglinfo -B 2>/dev/null || true)
    renderer=$(printf '%s\n' "$info" | emubox_opengl_renderer)
    if [[ -n "$renderer" ]]; then
      HAS_OPENGL=1
      OPENGL_RENDERER="$renderer"
      emubox_software_renderer "$renderer" || HAS_HW_OPENGL=1
      if [[ "$HAS_HW_OPENGL" == 1 || "$HAS_HW_VULKAN" != 1 ]]; then RENDERER_DESC="$renderer"; fi
    fi
  fi
  if [[ "$GPU_VENDOR" == unknown ]] && command -v lspci >/dev/null 2>&1; then
    info=$(lspci -mm 2>/dev/null | grep -Ei 'VGA compatible controller|3D controller|Display controller' || true)
    GPU_VENDOR=$(emubox_gpu_vendor "$info")
    GPU_DEVICE="${info:-unknown}"
  fi
  command -v gamescope >/dev/null 2>&1 && HAS_GAMESCOPE=1
  command -v cage >/dev/null 2>&1 && HAS_CAGE=1
  GRAPHICS_BACKEND=$(select_emubox_backend "$HAS_HW_VULKAN" "$HAS_HW_OPENGL")
  [[ "$GRAPHICS_BACKEND" == software ]] || HAS_ACCELERATION=1
  local renderer_vendor
  renderer_vendor=$(emubox_gpu_vendor "$RENDERER_DESC")
  [[ "$renderer_vendor" == unknown ]] || GPU_VENDOR="$renderer_vendor"
  GPU_KIND=physical
  [[ "$GPU_VENDOR" != virtual ]] || GPU_KIND=virtual
  [[ "$HAS_ACCELERATION" == 1 ]] || GPU_KIND=software
  systemd-detect-virt --vm --quiet 2>/dev/null && IS_VIRTUAL_MACHINE=1
  if [[ "$HAS_HW_VULKAN:$HAS_DRM:$HAS_GAMESCOPE" == 1:1:1 ]]; then GAMESCOPE_READY=1; fi
  EMUBOX_COMPOSITOR=$(select_emubox_compositor "${EMUBOX_COMPOSITOR_PREFERENCE:-auto}" "$GRAPHICS_BACKEND" "$HAS_DRM" "$HAS_CAGE" "$GAMESCOPE_READY")
  DEVICE_MODEL=$(tr -d '\000\n' 2>/dev/null < /sys/firmware/devicetree/base/model || cat /sys/class/dmi/id/product_name 2>/dev/null || echo unknown)
}