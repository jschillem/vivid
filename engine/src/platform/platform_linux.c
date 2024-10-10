#include <platform/platform.h>

#include <defines.h>

#ifdef VPLATFORM_LINUX

#include <core/event.h>
#include <core/input.h>
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

keys translate_keycode(u32 x_keycode);

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

  xcb_destroy_window(state->connection, state->window);

  XAutoRepeatOn(state->display);

  XCloseDisplay(state->display);

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
      xcb_key_press_event_t *kp = (xcb_key_press_event_t *)event;
      b8 is_down = kp->response_type == XCB_KEY_PRESS;
      xcb_keycode_t keycode = kp->detail;
      KeySym keysym = XkbKeycodeToKeysym(state->display, (KeyCode)keycode, 0,
                                         keycode & ShiftMask ? 1 : 0);

      keys key = translate_keycode(keysym);

      input_process_key(key, is_down);
    } break;

    case XCB_BUTTON_PRESS:
    case XCB_BUTTON_RELEASE: {
      xcb_button_press_event_t *bp = (xcb_button_press_event_t *)event;
      b8 is_down = bp->response_type == XCB_BUTTON_PRESS;
      buttons mouse_button = BUTTON_MAX_BUTTONS;
      switch (bp->detail) {
      case XCB_BUTTON_INDEX_1:
        mouse_button = BUTTON_LEFT;
        break;
      case XCB_BUTTON_INDEX_2:
        mouse_button = BUTTON_MIDDLE;
        break;
      case XCB_BUTTON_INDEX_3:
        mouse_button = BUTTON_RIGHT;
        break;
      }

      if (mouse_button != BUTTON_MAX_BUTTONS) {
        input_process_button(mouse_button, is_down);
      }
    } break;

    case XCB_MOTION_NOTIFY: {
      xcb_motion_notify_event_t *motion = (xcb_motion_notify_event_t *)event;

      input_process_mouse_move(motion->event_x, motion->event_y);
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

// Translate X11 keysym to engine key.
keys translate_keycode(u32 x_keycode) {
  switch (x_keycode) {
  case XK_BackSpace:
    return KEY_BACKSPACE;
  case XK_Return:
    return KEY_ENTER;
  case XK_Tab:
    return KEY_TAB;
  // case XK_Shift:
  //  return KEY_SHIFT;
  // case XK_Control:
  //  return KEY_CONTROL;
  case XK_Pause:
    return KEY_PAUSE;
  case XK_Caps_Lock:
    return KEY_CAPS_LOCK;
  case XK_Escape:
    return KEY_ESCAPE;
  // Not supported:
  // case : return KEY_CONVERT;
  // case : return KEY_NONCONVERT;
  // case : return KEY_ACCEPT;
  case XK_Mode_switch:
    return KEY_MODECHANGE;
  case XK_space:
    return KEY_SPACE;
  case XK_Prior:
    return KEY_PAGE_UP;
  case XK_Next:
    return KEY_PAGE_DOWN;
  case XK_End:
    return KEY_END;
  case XK_Home:
    return KEY_HOME;
  case XK_Left:
    return KEY_LEFT;
  case XK_Up:
    return KEY_UP;
  case XK_Right:
    return KEY_RIGHT;
  case XK_Down:
    return KEY_DOWN;
  case XK_Select:
    return KEY_SELECT;
  case XK_Print:
    return KEY_PRINT;
  case XK_Execute:
    return KEY_EXECUTE;
  // case XK_snapshot:
  //  return KEY_PRINT_SCREEN;
  case XK_Insert:
    return KEY_INSERT;
  case XK_Delete:
    return KEY_DELETE;
  case XK_Help:
    return KEY_HELP;
  case XK_Meta_L:
    return KEY_LEFT_WINDOWS;
  case XK_Meta_R:
    return KEY_RIGHT_WINDOWS;
  // case XK_app:
  //  return KEY_APPS;
  // case XK_sleep:
  //  return KEY_SLEEP;
  case XK_0:
    return KEY_0;
  case XK_1:
    return KEY_1;
  case XK_2:
    return KEY_2;
  case XK_3:
    return KEY_3;
  case XK_4:
    return KEY_4;
  case XK_5:
    return KEY_5;
  case XK_6:
    return KEY_6;
  case XK_7:
    return KEY_7;
  case XK_8:
    return KEY_8;
  case XK_9:
    return KEY_9;

  case XK_KP_0:
    return KEY_NUMPAD_0;
  case XK_KP_1:
    return KEY_NUMPAD_1;
  case XK_KP_2:
    return KEY_NUMPAD_2;
  case XK_KP_3:
    return KEY_NUMPAD_3;
  case XK_KP_4:
    return KEY_NUMPAD_4;
  case XK_KP_5:
    return KEY_NUMPAD_5;
  case XK_KP_6:
    return KEY_NUMPAD_6;
  case XK_KP_7:
    return KEY_NUMPAD_7;
  case XK_KP_8:
    return KEY_NUMPAD_8;
  case XK_KP_9:
    return KEY_NUMPAD_9;
  case XK_KP_Multiply:
    return KEY_MULTIPLY;
  case XK_KP_Add:
    return KEY_ADD;
  case XK_KP_Separator:
    return KEY_SEPARATOR;
  case XK_KP_Subtract:
    return KEY_SUBTRACT;
  case XK_KP_Decimal:
    return KEY_DECIMAL;
  case XK_KP_Divide:
    return KEY_DIVIDE;

  case XK_A:
  case XK_a:
    return KEY_A;
  case XK_B:
  case XK_b:
    return KEY_B;
  case XK_C:
  case XK_c:
    return KEY_C;
  case XK_D:
  case XK_d:
    return KEY_D;
  case XK_E:
  case XK_e:
    return KEY_E;
  case XK_F:
  case XK_f:
    return KEY_F;
  case XK_G:
  case XK_g:
    return KEY_G;
  case XK_H:
  case XK_h:
    return KEY_H;
  case XK_I:
  case XK_i:
    return KEY_I;
  case XK_J:
  case XK_j:
    return KEY_J;
  case XK_K:
  case XK_k:
    return KEY_K;
  case XK_L:
  case XK_l:
    return KEY_L;
  case XK_M:
  case XK_m:
    return KEY_M;
  case XK_N:
  case XK_n:
    return KEY_N;
  case XK_O:
  case XK_o:
    return KEY_O;
  case XK_P:
  case XK_p:
    return KEY_P;
  case XK_Q:
  case XK_q:
    return KEY_Q;
  case XK_R:
  case XK_r:
    return KEY_R;
  case XK_S:
  case XK_s:
    return KEY_S;
  case XK_T:
  case XK_t:
    return KEY_T;
  case XK_U:
  case XK_u:
    return KEY_U;
  case XK_V:
  case XK_v:
    return KEY_V;
  case XK_W:
  case XK_w:
    return KEY_W;
  case XK_X:
  case XK_x:
    return KEY_X;
  case XK_Y:
  case XK_y:
    return KEY_Y;
  case XK_Z:
  case XK_z:
    return KEY_Z;

  case XK_F1:
    return KEY_F1;
  case XK_F2:
    return KEY_F2;
  case XK_F3:
    return KEY_F3;
  case XK_F4:
    return KEY_F4;
  case XK_F5:
    return KEY_F5;
  case XK_F6:
    return KEY_F6;
  case XK_F7:
    return KEY_F7;
  case XK_F8:
    return KEY_F8;
  case XK_F9:
    return KEY_F9;
  case XK_F10:
    return KEY_F10;
  case XK_F11:
    return KEY_F11;
  case XK_F12:
    return KEY_F12;
  case XK_F13:
    return KEY_F13;
  case XK_F14:
    return KEY_F14;
  case XK_F15:
    return KEY_F15;
  case XK_F16:
    return KEY_F16;
  case XK_F17:
    return KEY_F17;
  case XK_F18:
    return KEY_F18;
  case XK_F19:
    return KEY_F19;
  case XK_F20:
    return KEY_F20;
  case XK_F21:
    return KEY_F21;
  case XK_F22:
    return KEY_F22;
  case XK_F23:
    return KEY_F23;
  case XK_F24:
    return KEY_F24;

  case XK_Num_Lock:
    return KEY_NUM_LOCK;
  case XK_Scroll_Lock:
    return KEY_SCROLL_LOCK;

  case XK_KP_Equal:
    return KEY_NUMPAD_EQUAL;

  case XK_Shift_L:
    return KEY_LSHIFT;
  case XK_Shift_R:
    return KEY_RSHIFT;
  case XK_Control_L:
    return KEY_LCONTROL;
  case XK_Control_R:
    return KEY_RCONTROL;
  // case XK_Menu: return KEY_LMENU;
  case XK_Menu:
    return KEY_RMENU;

  case XK_semicolon:
    return KEY_SEMICOLON;
  case XK_plus:
    return KEY_PLUS;
  case XK_comma:
    return KEY_COMMA;
  case XK_minus:
    return KEY_MINUS;
  case XK_period:
    return KEY_PERIOD;
  case XK_slash:
    return KEY_SLASH;
  case XK_grave:
    return KEY_GRAVE;
  default:
    return 0;
  }
}

#endif // VPLATFORM_LINUX
