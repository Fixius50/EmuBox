#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/installer/lib/graphics.sh"
( unset SVGA_NO_LOGGING; configure_emubox_host_logging oracle; [[ "$SVGA_NO_LOGGING" == 1 ]] )
( unset SVGA_NO_LOGGING; configure_emubox_host_logging vmware; [[ ! -v SVGA_NO_LOGGING ]] )
( unset SVGA_NO_LOGGING; configure_emubox_host_logging none; [[ ! -v SVGA_NO_LOGGING ]] )
( export SVGA_NO_LOGGING=0; configure_emubox_host_logging oracle; [[ "$SVGA_NO_LOGGING" == 0 ]] )
( configure_emubox_cage_command amdgpu /nonexistent x86_64; [[ "${CAGE_COMMAND[*]}" == cage ]] )
( export EMUBOX_VMWGFX_COMPAT=0; configure_emubox_cage_command vmwgfx /nonexistent x86_64; [[ "${CAGE_COMMAND[*]}" == cage ]] )
( configure_emubox_cage_command vmwgfx /nonexistent x86_64; [[ "${CAGE_COMMAND[*]}" == cage ]] )
if [[ -f bin/libemubox-vmwgfx.so && -x /lib64/ld-linux-x86-64.so.2 ]] && command -v cage >/dev/null; then
  ( unset LD_PRELOAD EMUBOX_VMWGFX_COMPAT;
    configure_emubox_cage_command vmwgfx "$PWD" x86_64;
    [[ "${CAGE_COMMAND[1]}" == --preload && "${CAGE_COMMAND[2]}" == "$PWD/bin/libemubox-vmwgfx.so" && ! -v LD_PRELOAD ]] )
fi
echo 'VirtualBox host logging and isolated vmwgfx compatibility selection: OK'
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
( unset WLR_SCENE_DEBUG_DAMAGE WLR_SCENE_DISABLE_DIRECT_SCANOUT;
  configure_emubox_presentation vmwgfx;
  [[ "$WLR_SCENE_DEBUG_DAMAGE" == rerender && "$WLR_SCENE_DISABLE_DIRECT_SCANOUT" == 1 ]] )
( unset WLR_SCENE_DEBUG_DAMAGE WLR_SCENE_DISABLE_DIRECT_SCANOUT;
  configure_emubox_presentation amdgpu;
  [[ ! -v WLR_SCENE_DEBUG_DAMAGE && ! -v WLR_SCENE_DISABLE_DIRECT_SCANOUT ]] )
( export WLR_SCENE_DEBUG_DAMAGE=none WLR_SCENE_DISABLE_DIRECT_SCANOUT=0;
  configure_emubox_presentation vmwgfx;
  [[ "$WLR_SCENE_DEBUG_DAMAGE" == none && "$WLR_SCENE_DISABLE_DIRECT_SCANOUT" == 0 ]] )
echo 'vmwgfx full composition and preserved explicit presentation settings: OK'
( unset WEBKIT_DISABLE_DMABUF_RENDERER WEBKIT_DMABUF_RENDERER_FORCE_SHM LIBGL_ALWAYS_SOFTWARE WLR_RENDERER WEBKIT_DISABLE_COMPOSITING_MODE;
  configure_emubox_render_mode auto;
  configure_emubox_webkit vmwgfx;
  [[ "$WEBKIT_DISABLE_DMABUF_RENDERER" == 0 && "$WEBKIT_DMABUF_RENDERER_FORCE_SHM" == 1 ]]
  [[ ! -v LIBGL_ALWAYS_SOFTWARE && ! -v WLR_RENDERER && ! -v WEBKIT_DISABLE_COMPOSITING_MODE ]] )
( unset WEBKIT_DISABLE_DMABUF_RENDERER WEBKIT_DMABUF_RENDERER_FORCE_SHM;
  configure_emubox_webkit amdgpu;
  [[ ! -v WEBKIT_DISABLE_DMABUF_RENDERER && ! -v WEBKIT_DMABUF_RENDERER_FORCE_SHM ]] )
( export WEBKIT_DISABLE_DMABUF_RENDERER=1; unset WEBKIT_DMABUF_RENDERER_FORCE_SHM;
  configure_emubox_webkit vmwgfx;
  [[ "$WEBKIT_DISABLE_DMABUF_RENDERER" == 1 && ! -v WEBKIT_DMABUF_RENDERER_FORCE_SHM ]] )
( export WEBKIT_DISABLE_DMABUF_RENDERER=0 WEBKIT_DMABUF_RENDERER_FORCE_SHM=0;
  configure_emubox_webkit vmwgfx;
  [[ "$WEBKIT_DISABLE_DMABUF_RENDERER" == 0 && "$WEBKIT_DMABUF_RENDERER_FORCE_SHM" == 0 ]] )
( configure_emubox_render_mode software; unset WEBKIT_DISABLE_DMABUF_RENDERER WEBKIT_DMABUF_RENDERER_FORCE_SHM;
  configure_emubox_webkit vmwgfx;
  [[ "$LIBGL_ALWAYS_SOFTWARE" == 1 && "$WLR_RENDERER" == pixman && "$WEBKIT_DISABLE_COMPOSITING_MODE" == 1 ]] )
echo 'vmwgfx WebKit shared-memory frames, GPU policy and explicit overrides: OK'
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