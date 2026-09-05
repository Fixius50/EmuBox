#!/usr/bin/env bash

emubox_gpu_vendor() {
  local text=" ${1,,} "
  case "$text" in
    *vmware*|*vmwgfx*|*virtualbox*|*virtio*|*qxl*) echo virtual ;;
    *amdgpu*|*radeon*|*' amd '*|*'1002:'*) echo amd ;;
    *nvidia*|*nouveau*|*'10de:'*) echo nvidia ;;
    *intel*|*i915*|*' xe '*|*'8086:'*) echo intel ;;
    *broadcom*|*v3d*|*vc4*) echo broadcom ;;
    *mali*|*panfrost*|*panthor*|*lima*) echo arm ;;
    *qualcomm*|*adreno*|*freedreno*|*' msm '*) echo qualcomm ;;
    *apple*|*asahi*|*' agx '*) echo apple ;;
    *) echo unknown ;;
  esac
}

emubox_vulkan_renderer() {
  awk '
    function emit() { if (hardware && name != "") { print name; found=1; exit } }
    /^GPU[0-9]+:/ { emit(); hardware=0; name="" }
    /deviceType[[:space:]]*=/ { hardware=($0 ~ /DISCRETE_GPU|INTEGRATED_GPU|VIRTUAL_GPU/) }
    /deviceName[[:space:]]*=/ { sub(/^[^=]*=[[:space:]]*/, ""); name=$0 }
    END { if (!found && hardware && name != "") print name }
  '
}

select_emubox_compositor() {
  if [[ "$1" == 1 && "$2" == 1 && "$3" == 1 ]]; then echo gamescope; else echo cage; fi
}

emubox_software_renderer() {
  case "${1,,}" in
    *llvmpipe*|*softpipe*|*swrast*|*'software rasterizer'*|*swiftshader*) return 0 ;;
    *) return 1 ;;
  esac
}

emubox_opengl_renderer() {
  awk '
    /OpenGL (core profile renderer|compatibility profile renderer|ES profile renderer|renderer string):/ {
      sub(/^[^:]*:[[:space:]]*/, "")
      if ($0 == "") next
      if (first == "") first=$0
      if (tolower($0) !~ /llvmpipe|softpipe|swrast|software rasterizer|swiftshader/) { print; found=1; exit }
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
    info=$(LC_ALL=C vulkaninfo --summary 2>/dev/null) || info=''
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
      [[ "$HAS_HW_VULKAN" == 1 ]] || RENDERER_DESC="$renderer"
    fi
  fi
  if [[ "$GPU_VENDOR" == unknown ]] && command -v lspci >/dev/null 2>&1; then
    info=$(lspci -mm 2>/dev/null | grep -Ei 'VGA compatible controller|3D controller|Display controller' || true)
    GPU_VENDOR=$(emubox_gpu_vendor "$info")
    GPU_DEVICE="${info:-unknown}"
  fi
  command -v gamescope >/dev/null 2>&1 && HAS_GAMESCOPE=1
  EMUBOX_COMPOSITOR=$(select_emubox_compositor "$HAS_HW_VULKAN" "$HAS_DRM" "$HAS_GAMESCOPE")
  DEVICE_MODEL=$(tr -d '\000\n' 2>/dev/null < /sys/firmware/devicetree/base/model || cat /sys/class/dmi/id/product_name 2>/dev/null || echo unknown)
}