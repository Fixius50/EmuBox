#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/installer/lib/graphics.sh"
[[ "$(emubox_gpu_vendor 'VGA compatible controller')" == unknown ]]
[[ "$(emubox_gpu_vendor 'AMD Radeon')" == amd ]]
[[ "$(emubox_gpu_vendor 'Intel i915')" == intel ]]
[[ "$(emubox_gpu_vendor NVIDIA)" == nvidia ]]
[[ "$(emubox_gpu_vendor 'Mali panfrost')" == arm ]]
[[ "$(emubox_gpu_vendor v3d)" == broadcom ]]
[[ "$(emubox_gpu_vendor Adreno)" == qualcomm ]]
[[ "$(emubox_gpu_vendor 'Apple AGX')" == apple ]]
for vulkan in 0 1; do
  for drm in 0 1; do
    for gamescope in 0 1; do
      expected=cage
      [[ "$vulkan:$drm:$gamescope" != 1:1:1 ]] || expected=gamescope
      [[ "$(select_emubox_compositor "$vulkan" "$drm" "$gamescope")" == "$expected" ]]
    done
  done
done
software=$'GPU0:\n deviceType = PHYSICAL_DEVICE_TYPE_CPU\n deviceName = llvmpipe'
[[ -z "$(printf '%s\n' "$software" | emubox_vulkan_renderer)" ]]
[[ "$(printf '%s\n' "$software" $'GPU1:\n deviceType = PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU\n deviceName = Mali' | emubox_vulkan_renderer)" == Mali ]]
echo 'Graphics and compositor matrix: OK'