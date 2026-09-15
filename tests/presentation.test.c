#define _POSIX_C_SOURCE 200809L
#define WLR_USE_UNSTABLE
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <drm_fourcc.h>
#include <wayland-server-core.h>
#include <wlr/backend/headless.h>
#include <wlr/render/allocator.h>
#include <wlr/render/wlr_renderer.h>
#include <wlr/render/wlr_texture.h>
#include <wlr/types/wlr_output.h>
#include <wlr/types/wlr_scene.h>

struct presentation_test {
    struct wl_display *display;
    struct wlr_output *output;
    struct wlr_scene_output *scene_output;
    struct wlr_scene_rect *patch;
    struct wl_listener frame;
    unsigned frames;
};

static void render_frame(struct wl_listener *listener, void *data)
{
    (void)data;
    struct presentation_test *test = wl_container_of(listener, test, frame);
    struct wlr_output_state state;
    wlr_output_state_init(&state);
    assert(wlr_scene_output_build_state(test->scene_output, &state, NULL));
    assert(state.buffer != NULL);
    const pixman_box32_t *damage = pixman_region32_extents(&state.damage);
    printf("frame=%u damage=%d,%d-%d,%d\n", test->frames, damage->x1, damage->y1, damage->x2, damage->y2);
    assert(damage->x1 == 0 && damage->y1 == 0 && damage->x2 == 96 && damage->y2 == 64);
    struct wlr_texture *texture = wlr_texture_from_buffer(test->output->renderer, state.buffer);
    assert(texture != NULL);
    uint32_t pixels[96 * 64] = {0};
    const struct wlr_texture_read_pixels_options read_options = {
        .data = pixels,
        .format = DRM_FORMAT_XRGB8888,
        .stride = 96 * sizeof(uint32_t),
    };
    assert(wlr_texture_read_pixels(texture, &read_options));
    for (unsigned row = 0; row < 64; row++) {
        for (unsigned column = 0; column < 96; column++) {
            uint32_t expected = 0x00ff0000;
            if (row >= 16 && row < 24 && column >= 16 && column < 24) {
                expected = test->frames % 2 == 0 ? 0x0000ff00 : 0x000000ff;
            }
            assert((pixels[row * 96 + column] & 0x00ffffff) == expected);
        }
    }
    wlr_texture_destroy(texture);
    assert(wlr_output_commit_state(test->output, &state));
    wlr_output_state_finish(&state);
    test->frames++;
    if (test->frames == 8) {
        wl_display_terminate(test->display);
        return;
    }
    const float green[4] = {0, 1, 0, 1};
    const float blue[4] = {0, 0, 1, 1};
    wlr_scene_rect_set_color(test->patch, test->frames % 2 == 0 ? green : blue);
    wlr_output_schedule_frame(test->output);
}

static int deadline(void *data)
{
    struct presentation_test *test = data;
    wl_display_terminate(test->display);
    return 0;
}

int main(void)
{
    assert(setenv("WLR_SCENE_DEBUG_DAMAGE", "rerender", 1) == 0);
    assert(setenv("WLR_SCENE_DISABLE_DIRECT_SCANOUT", "1", 1) == 0);
    struct wl_display *display = wl_display_create();
    assert(display != NULL);
    struct wl_event_loop *loop = wl_display_get_event_loop(display);
    struct wlr_backend *backend = wlr_headless_backend_create(loop);
    assert(backend != NULL);
    struct wlr_renderer *renderer = wlr_renderer_autocreate(backend);
    assert(renderer != NULL);
    struct wlr_allocator *allocator = wlr_allocator_autocreate(backend, renderer);
    assert(allocator != NULL);
    struct wlr_output *output = wlr_headless_add_output(backend, 96, 64);
    assert(output != NULL && wlr_output_init_render(output, allocator, renderer));
    struct wlr_output_state state;
    wlr_output_state_init(&state);
    wlr_output_state_set_enabled(&state, true);
    wlr_output_state_set_custom_mode(&state, 96, 64, 60000);
    assert(wlr_output_commit_state(output, &state));
    wlr_output_state_finish(&state);
    struct wlr_scene *scene = wlr_scene_create();
    assert(scene != NULL);
    const float red[4] = {1, 0, 0, 1};
    const float green[4] = {0, 1, 0, 1};
    assert(wlr_scene_rect_create(&scene->tree, 96, 64, red) != NULL);
    struct wlr_scene_rect *patch = wlr_scene_rect_create(&scene->tree, 8, 8, green);
    assert(patch != NULL);
    wlr_scene_node_set_position(&patch->node, 16, 16);
    struct presentation_test test = {
        .display = display,
        .output = output,
        .scene_output = wlr_scene_output_create(scene, output),
        .patch = patch,
        .frame.notify = render_frame,
    };
    assert(test.scene_output != NULL);
    wl_signal_add(&output->events.frame, &test.frame);
    struct wl_event_source *timer = wl_event_loop_add_timer(loop, deadline, &test);
    assert(timer != NULL && wl_event_source_timer_update(timer, 5000) == 0);
    assert(wlr_backend_start(backend));
    wlr_output_schedule_frame(output);
    wl_display_run(display);
    assert(test.frames == 8);
    wl_event_source_remove(timer);
    wl_list_remove(&test.frame.link);
    wlr_scene_node_destroy(&scene->tree.node);
    wlr_backend_destroy(backend);
    wlr_allocator_destroy(allocator);
    wlr_renderer_destroy(renderer);
    wl_display_destroy(display);
    puts("Presentation: 8 full frames, 49152 pixels verified, small-region updates without mouse input.");
    return 0;
}