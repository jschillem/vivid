#include <platform/platform.h>

#include <defines.h>

#ifdef VPLATFORM_LINUX

#include <core/logger.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/time.h>

#if _POSIX_C_SOURCE >= 199309L
#include <time.h>
#else
#include <unistd.h>
#endif

#include <X11/XKBlib.h>
#include <X11/Xlib-xcb.h>
#include <X11/Xlib.h>
#include <X11/keysym.h>
#include <xcb/xcb.h>

typedef struct internal_state {
  Display *display;
  xcb_connection_t *connection;
  xcb_window_t window;
  xcb_screen_t *screen;
  xcb_atom_t wm_protocols;
  xcb_atom_t wm_delete_window;
} internal_state;

static const char *color_strings[] = {"0;41", "1;31", "1;33",
                                      "1;32", "1;34", "1;30"};

b8 platform_init(platform_state *plat_state, const char *title, u32 x, u32 y,
                 u32 width, u32 height) {
  plat_state->internal_state = platform_allocate(sizeof(internal_state), FALSE);
  internal_state *state = (internal_state *)plat_state->internal_state;

  // connect to X server
  state->display = XOpenDisplay(NULL);

  XAutoRepeatOff(state->display);

  i32 screen_p = 0;
  state->connection = XGetXCBConnection(state->display);

  if (xcb_connection_has_error(state->connection)) {
    VFATAL("Failed to connect to X server via XCB.");
    return FALSE;
  }

  const xcb_setup_t *setup = xcb_get_setup(state->connection);
  xcb_screen_iterator_t iter = xcb_setup_roots_iterator(setup);
  for (i32 i = 0; i < screen_p; ++i) {
    xcb_screen_next(&iter);
  }

  state->screen = iter.data;

  state->window = xcb_generate_id(state->connection);

  u32 event_mask = XCB_CW_BACK_PIXEL | XCB_CW_EVENT_MASK;

  u32 event_values = XCB_EVENT_MASK_BUTTON_PRESS |
                     XCB_EVENT_MASK_BUTTON_RELEASE | XCB_EVENT_MASK_KEY_PRESS |
                     XCB_EVENT_MASK_KEY_RELEASE | XCB_EVENT_MASK_EXPOSURE |
                     XCB_EVENT_MASK_POINTER_MOTION |
                     XCB_EVENT_MASK_STRUCTURE_NOTIFY;

  u32 value_list[] = {state->screen->black_pixel, event_values};

  xcb_void_cookie_t window_cookie =
      xcb_create_window(state->connection, XCB_COPY_FROM_PARENT, state->window,
                        state->screen->root, x, y, width, height, 0,
                        XCB_WINDOW_CLASS_INPUT_OUTPUT,
                        state->screen->root_visual, event_mask, value_list);

  xcb_change_property(state->connection, XCB_PROP_MODE_REPLACE, state->window,
                      XCB_ATOM_WM_NAME, XCB_ATOM_STRING, 8, strlen(title),
                      title);

  xcb_intern_atom_cookie_t wm_delete_cookie = xcb_intern_atom(
      state->connection, 0, strlen("WM_DELETE_WINDOW"), "WM_DELETE_WINDOW");

  xcb_intern_atom_cookie_t wm_protocols_cookie = xcb_intern_atom(
      state->connection, 0, strlen("WM_PROTOCOLS"), "WM_PROTOCOLS");

  xcb_intern_atom_reply_t *wm_delete_reply =
      xcb_intern_atom_reply(state->connection, wm_delete_cookie, NULL);

  xcb_intern_atom_reply_t *wm_protocols_reply =
      xcb_intern_atom_reply(state->connection, wm_protocols_cookie, NULL);

  state->wm_delete_window = wm_delete_reply->atom;
  state->wm_protocols = wm_protocols_reply->atom;

  xcb_change_property(state->connection, XCB_PROP_MODE_REPLACE, state->window,
                      wm_protocols_reply->atom, 4, 32, 1,
                      &wm_delete_reply->atom);

  xcb_map_window(state->connection, state->window);

  // flush the request
  i32 stream_result = xcb_flush(state->connection);
  if (stream_result <= 0) {
    VFATAL("Failed to flush XCB connection: %d", stream_result);
    return FALSE;
  }

  return TRUE;
}

void platform_shutdown(platform_state *plat_state) {
  internal_state *state = (internal_state *)plat_state->internal_state;

  XAutoRepeatOn(state->display);
  xcb_destroy_window(state->connection, state->window);

  platform_free(plat_state->internal_state, FALSE);
}

b8 platform_pump_messages(platform_state *plat_state) {
  internal_state *state = (internal_state *)plat_state->internal_state;

  xcb_generic_event_t *event;
  xcb_client_message_event_t *cm;

  b8 quit = FALSE;

  while ((event = xcb_poll_for_event(state->connection))) {
    switch (event->response_type & ~0x80) {
    case XCB_KEY_PRESS:
    case XCB_KEY_RELEASE: {
      // TODO: key press/release
    } break;

    case XCB_BUTTON_PRESS:
    case XCB_BUTTON_RELEASE: {
      // TODO: button press/release
    } break;

    case XCB_MOTION_NOTIFY: {
      // TODO: mouse motion
    } break;

    case XCB_CONFIGURE_NOTIFY: {
      // TODO: window resize
    } break;

    case XCB_CLIENT_MESSAGE: {
      cm = (xcb_client_message_event_t *)event;
      if (cm->data.data32[0] == state->wm_delete_window) {
        quit = TRUE;
      }
    } break;
    default:
      break;
    }

    free(event);
  }

  return !quit;
}

void *platform_allocate(u64 size, b8 aligned) { return malloc(size); }
void platform_free(void *block, b8 aligned) { free(block); }
void *platform_zero_memory(void *block, u64 size) {
  return platform_set_memory(block, 0, size);
}
void *platform_copy_memory(void *dest, const void *src, u64 size) {
  return memcpy(dest, src, size);
}
void *platform_set_memory(void *dest, u8 value, u64 size) {
  return memset(dest, value, size);
}

void platform_console_write(const char *message, u8 color) {
  fprintf(stdout, "\033[%sm%s\033[0m", color_strings[color], message);
}
void platform_console_write_error(const char *message, u8 color) {
  fprintf(stderr, "\033[%sm%s\033[0m", color_strings[color], message);
}

f64 platform_get_absolute_time() {
  struct timespec now;
  clock_gettime(CLOCK_MONOTONIC, &now);
  return (f64)now.tv_sec + (f64)now.tv_nsec * 1e-9;
}

void platform_sleep(u64 ms) {
#if _POSIX_C_SOURCE >= 199309L
  struct timespec sleep_time = {0};
  sleep_time.tv_sec = ms / 1000;
  sleep_time.tv_nsec = (ms % 1000) * 1000 * 1000;
  nanosleep(&sleep_time, NULL);
#else
  if (ms >= 1000) {
    sleep(ms / 1000);
  }

  usleep((ms % 1000) * 1000);
#endif
}

#endif // VPLATFORM_LINUX
