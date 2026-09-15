#include <errno.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>
#include <xf86drm.h>
#include <vmwgfx_drm.h>

int drmCloseBufferHandle(int descriptor, uint32_t handle)
{
    struct drm_gem_close generic = { .handle = handle };
    int result = drmIoctl(descriptor, DRM_IOCTL_GEM_CLOSE, &generic);
    if (result == 0 || errno != EINVAL) {
        return result;
    }

    int generic_error = errno;
    drmVersionPtr version = drmGetVersion(descriptor);
    bool legacy_vmwgfx = version != NULL && version->name != NULL
        && version->name_len == 6 && memcmp(version->name, "vmwgfx", 6) == 0;
    if (version != NULL) {
        drmFreeVersion(version);
    }
    if (!legacy_vmwgfx) {
        errno = generic_error;
        return result;
    }

    struct drm_vmw_surface_arg surface = {
        .sid = (int32_t)handle,
        .handle_type = DRM_VMW_HANDLE_LEGACY,
    };
    result = drmCommandWrite(descriptor, DRM_VMW_UNREF_SURFACE, &surface, sizeof(surface));
    if (result < 0) {
        errno = -result;
        return -1;
    }
    errno = 0;
    return result;
}