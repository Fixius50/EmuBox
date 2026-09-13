#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/installer/lib/graphics.sh"
for backend in opengl vulkan software; do
  for drm in 0 1; do
    for gamescope in 0 1; do
      expected=unavailable
      if [[ "$drm" == 1 ]]; then
        if [[ "$backend" != vulkan ]]; then expected=cage
        elif [[ "$gamescope" == 1 ]]; then expected=gamescope; fi
      fi
      [[ "$(select_emubox_compositor auto "$backend" "$drm" 1 "$gamescope")" == "$expected" ]]
    done
  done
done
[[ "$(select_emubox_compositor gamescope opengl 1 1 0)" == cage ]]
[[ "$(select_emubox_compositor gamescope opengl 1 1 1)" == gamescope ]]
[[ "$(select_emubox_compositor auto software 1 0 1)" == unavailable ]]
echo 'Graphics and compositor matrix: OK'
svga='SVGA3D; build: RELEASE; LLVM;'
! emubox_software_renderer "$svga"
emubox_software_renderer 'llvmpipe (LLVM 22)'
echo 'OpenGL accelerated and software renderer matrix: OK'
( unset WLR_NO_HARDWARE_CURSORS; configure_emubox_cursor vmwgfx; [[ "$WLR_NO_HARDWARE_CURSORS" == 1 ]] )
( unset WLR_NO_HARDWARE_CURSORS; configure_emubox_cursor amdgpu; [[ ! -v WLR_NO_HARDWARE_CURSORS ]] )
( export WLR_NO_HARDWARE_CURSORS=0; configure_emubox_cursor vmwgfx; [[ "$WLR_NO_HARDWARE_CURSORS" == 0 ]] )
echo 'vmwgfx cursor fallback and explicit preferences: OK'
( unset WLR_DRM_NO_ATOMIC; configure_emubox_cursor vmwgfx; [[ "$WLR_DRM_NO_ATOMIC" == 1 ]] )
( unset WLR_DRM_NO_ATOMIC; configure_emubox_cursor amdgpu; [[ ! -v WLR_DRM_NO_ATOMIC ]] )
( export WLR_DRM_NO_ATOMIC=0; configure_emubox_cursor vmwgfx; [[ "$WLR_DRM_NO_ATOMIC" == 0 ]] )
echo 'vmwgfx legacy DRM and explicit preferences: OK'
( unset LIBGL_ALWAYS_SOFTWARE WLR_RENDERER WEBKIT_DISABLE_COMPOSITING_MODE; configure_emubox_render_mode auto; [[ ! -v LIBGL_ALWAYS_SOFTWARE && ! -v WLR_RENDERER && ! -v WEBKIT_DISABLE_COMPOSITING_MODE ]] )
( configure_emubox_render_mode software; [[ "$LIBGL_ALWAYS_SOFTWARE" == 1 && "$WLR_RENDERER" == pixman && "$WEBKIT_DISABLE_COMPOSITING_MODE" == 1 ]] )
( ! configure_emubox_render_mode invalid )
echo 'Explicit software diagnostic mode and unchanged automatic mode: OK'
[[ "$(select_emubox_render_mode auto 1 0)" == auto ]]
[[ "$(select_emubox_render_mode auto 0 1)" == auto ]]
[[ "$(select_emubox_render_mode auto 0 0)" == auto ]]
[[ "$(select_emubox_render_mode auto 0 0 software)" == software ]]
[[ "$(select_emubox_render_mode software 0 0 indeterminate)" == software ]]
[[ "$(select_emubox_compositor auto auto 1 1 0)" == cage ]]
[[ "$(select_emubox_render_mode software 1 1)" == auto ]]
( configure_emubox_render_mode software; configure_emubox_render_mode auto; [[ ! -v LIBGL_ALWAYS_SOFTWARE && ! -v WLR_RENDERER && ! -v WEBKIT_DISABLE_COMPOSITING_MODE ]] )
( export GALLIUM_DRIVER=llvmpipe MESA_LOADER_DRIVER_OVERRIDE=swrast; configure_emubox_render_mode auto; [[ ! -v GALLIUM_DRIVER && ! -v MESA_LOADER_DRIVER_OVERRIDE ]] )
( export GALLIUM_DRIVER=zink MESA_LOADER_DRIVER_OVERRIDE=iris; configure_emubox_render_mode auto; [[ "$GALLIUM_DRIVER" == zink && "$MESA_LOADER_DRIVER_OVERRIDE" == iris ]] )
! select_emubox_render_mode invalid 1 1
echo 'GPU Vulkan/OpenGL (including VM 3D) first, CPU only without acceleration: OK'
( export EMUBOX_BIN=/nonexistent/emubox EMUBOX_RENDER_MODE=auto; detect_emubox_graphics;
  [[ "$GRAPHICS_DETECTION_STATE" == indeterminate && "$GRAPHICS_OPERATIONAL_BACKEND" == auto && "$GPU_ACTIVE_DEVICE" == unknown ]] )
( export EMUBOX_BIN=/nonexistent/emubox EMUBOX_RENDER_MODE=software; detect_emubox_graphics;
  [[ "$GRAPHICS_DETECTION_STATE" == indeterminate && "$GRAPHICS_OPERATIONAL_BACKEND" == software && "$GRAPHICS_FALLBACK_REASON" == explicit_software_fallback ]] )