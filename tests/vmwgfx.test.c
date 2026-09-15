#define _GNU_SOURCE
#include <assert.h>
#include <dlfcn.h>
#include <errno.h>
#include <fcntl.h>
#include <gbm.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <xf86drm.h>

typedef int (*close_buffer_fn)(int, uint32_t);

int main(int argc, char **argv)
{
    assert(argc == 2 || argc == 3);
    void *library = dlopen(argv[1], RTLD_NOW | RTLD_LOCAL);
    if (library == NULL) {
        fprintf(stderr, "%s\n", dlerror());
        return 1;
    }
    close_buffer_fn compatible_close = (close_buffer_fn)dlsym(library, "drmCloseBufferHandle");
    assert(compatible_close != NULL);
    assert(compatible_close(-1, 1) == -1 && errno == EBADF);
    int unrelated = open("/dev/null", O_RDWR | O_CLOEXEC);
    assert(unrelated >= 0);
    assert(compatible_close(unrelated, 1) == -1 && errno == ENOTTY);
    close(unrelated);

    if (argc == 2) {
        puts("DRM compatibility: invalid descriptors preserve actual errors.");
        dlclose(library);
        return 0;
    }

    int exporter = open(argv[2], O_RDWR | O_CLOEXEC);
    int importer = open(argv[2], O_RDWR | O_CLOEXEC);
    assert(exporter >= 0 && importer >= 0);
    drmVersionPtr version = drmGetVersion(importer);
    assert(version != NULL && strcmp(version->name, "vmwgfx") == 0);
    drmFreeVersion(version);
    struct gbm_device *device = gbm_create_device(exporter);
    assert(device != NULL);
    unsigned generic_failures = 0;
    for (unsigned iteration = 0; iteration < 128; iteration++) {
        struct gbm_bo *buffer = gbm_bo_create(device, 64, 64, GBM_FORMAT_XRGB8888, GBM_BO_USE_RENDERING);
        assert(buffer != NULL);
        uint32_t stride = 0;
        void *mapping = NULL;
        errno = 0;
        uint32_t *pixels = gbm_bo_map(buffer, 0, 0, 64, 64, GBM_BO_TRANSFER_WRITE, &stride, &mapping);
        assert(pixels != NULL);
        pixels[0] = 0x00123400 | iteration;
        gbm_bo_unmap(buffer, mapping);
        mapping = NULL;
        errno = 0;
        pixels = gbm_bo_map(buffer, 0, 0, 64, 64, GBM_BO_TRANSFER_READ, &stride, &mapping);
        assert(pixels != NULL);
        uint32_t original_pixel = pixels[0];
        assert(original_pixel == (0x00123400 | iteration));
        gbm_bo_unmap(buffer, mapping);
        int prime = gbm_bo_get_fd(buffer);
        assert(prime >= 0);
        uint32_t handle;
        assert(drmPrimeFDToHandle(importer, prime, &handle) == 0);
        if (drmCloseBufferHandle(importer, handle) != 0) {
            assert(errno == EINVAL);
            generic_failures++;
            assert(compatible_close(importer, handle) == 0);
        }
        assert(compatible_close(importer, handle) == -1);
        assert(drmPrimeFDToHandle(importer, prime, &handle) == 0);
        assert(compatible_close(importer, handle) == 0);
        assert(compatible_close(importer, handle) == -1);
        mapping = NULL;
        errno = 0;
        pixels = gbm_bo_map(buffer, 0, 0, 64, 64, GBM_BO_TRANSFER_READ, &stride, &mapping);
        assert(pixels != NULL);
        if (pixels[0] != original_pixel) {
            fprintf(stderr, "iteration=%u before=%08x after=%08x\n", iteration, original_pixel, pixels[0]);
        }
        assert(pixels[0] == original_pixel);
        gbm_bo_unmap(buffer, mapping);
        close(prime);
        gbm_bo_destroy(buffer);
    }
    gbm_device_destroy(device);
    close(importer);
    close(exporter);
    dlclose(library);
    printf("128 PRIME cycles: %u generic failures repaired; double-close rejected; pixels preserved.\n", generic_failures);
    return 0;
}