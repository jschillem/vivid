#include <core/input.h>

#include <core/event.h>
#include <core/logger.h>
#include <core/vmemory.h>

typedef struct keyboard_state {
  b8 keys[KEY_MAX_KEYS];
} keyboard_state;

typedef struct mouse_state {
  u8 buttons[BUTTON_MAX_BUTTONS];
  i32 x;
  i32 y;
} mouse_state;

typedef struct input_state {
  keyboard_state keyboard_current;
  keyboard_state keyboard_previous;
  mouse_state mouse_current;
  mouse_state mouse_previous;
} input_state;

// Internal state of the input system.
static b8 is_initialized = FALSE;
static input_state state;

void input_init() {
  if (is_initialized) {
    VWARN("Input system already initialized.");
    return;
  }
  vzero_memory(&state, sizeof(input_state));

  is_initialized = TRUE;
}

void input_shutdown() {
  if (!is_initialized) {
    VWARN("Input system already shutdown.");
    return;
  }

  // TODO: Add shutdown routines when needed.

  is_initialized = FALSE;
}

void input_update(f64 delta_time) {
  if (!is_initialized) {
    VWARN("Input system not initialized.");
    return;
  }

  // Update previous state.
  vcopy_memory(&state.keyboard_previous, &state.keyboard_current,
               sizeof(keyboard_state));
  vcopy_memory(&state.mouse_previous, &state.mouse_current,
               sizeof(mouse_state));
}

void input_process_key(keys key, b8 is_down) {
  // only handle this if the key state has changed
  if (state.keyboard_current.keys[key] == is_down) {
    return;
  }

  state.keyboard_current.keys[key] = is_down;

  // fire the event
  event_context context;
  context.data.u16[0] = key;
  event_fire(is_down ? EVENT_CODE_KEY_PRESSED : EVENT_CODE_KEY_RELEASED, NULL,
             context);
}

void input_process_button(buttons button, b8 is_down) {
  // only handle this if the button state has changed
  if (state.mouse_current.buttons[button] == is_down) {
    return;
  }

  state.mouse_current.buttons[button] = is_down;

  // fire the event
  event_context context;
  context.data.u16[0] = button;
  event_fire(is_down ? EVENT_CODE_BUTTON_PRESSED : EVENT_CODE_BUTTON_RELEASED,
             NULL, context);
}

void input_process_mouse_move(i32 x, i32 y) {
  // only handle this if the mouse position has changed
  if (state.mouse_current.x == x && state.mouse_current.y == y) {
    return;
  }

  VDEBUG("Mouse pos: %i, %i!", x, y);

  state.mouse_current.x = x;
  state.mouse_current.y = y;

  // fire the event
  event_context context;
  context.data.u16[0] = x;
  context.data.u16[1] = y;
  event_fire(EVENT_CODE_MOUSE_MOVED, NULL, context);
}

void input_process_mouse_wheel(i8 z_delta) {
  // NOTE: no input state to check for mouse wheel delta change

  // fire the event
  event_context context;
  context.data.i8[0] = z_delta;
  event_fire(EVENT_CODE_MOUSE_WHEEL, NULL, context);
}

b8 input_is_key_down(keys key) {
  if (!is_initialized) {
    VWARN("Input system not initialized.");
    return FALSE;
  }

  return state.keyboard_current.keys[key];
}

b8 input_is_key_up(keys key) {
  if (!is_initialized) {
    VWARN("Input system not initialized.");
    return FALSE;
  }

  return !state.keyboard_current.keys[key];
}

b8 input_was_key_down(keys key) {
  if (!is_initialized) {
    VWARN("Input system not initialized.");
    return FALSE;
  }

  return state.keyboard_previous.keys[key];
}

b8 input_was_key_up(keys key) {
  if (!is_initialized) {
    VWARN("Input system not initialized.");
    return FALSE;
  }

  return !state.keyboard_previous.keys[key];
}

b8 input_is_button_down(buttons button) {
  if (!is_initialized) {
    VWARN("Input system not initialized.");
    return FALSE;
  }

  return state.mouse_current.buttons[button];
}

b8 input_is_button_up(buttons button) {
  if (!is_initialized) {
    VWARN("Input system not initialized.");
    return FALSE;
  }

  return !state.mouse_current.buttons[button];
}

b8 input_was_button_down(buttons button) {
  if (!is_initialized) {
    VWARN("Input system not initialized.");
    return FALSE;
  }

  return state.mouse_previous.buttons[button];
}

b8 input_was_button_up(buttons button) {
  if (!is_initialized) {
    VWARN("Input system not initialized.");
    return FALSE;
  }

  return !state.mouse_previous.buttons[button];
}

void input_get_mouse_position(i32 *x, i32 *y) {
  if (!is_initialized) {
    VWARN("Input system not initialized.");
    return;
  }

  *x = state.mouse_current.x;
  *y = state.mouse_current.y;
}

void input_get_previous_mouse_position(i32 *x, i32 *y) {
  if (!is_initialized) {
    VWARN("Input system not initialized.");
    return;
  }

  *x = state.mouse_previous.x;
  *y = state.mouse_previous.y;
}
