#pragma once

#include <defines.h>

typedef struct event_context {
  // 128 bits
  union event_data {
    i64 i64[2];
    u64 u64[2];
    f64 f64[2];

    i32 i32[4];
    u32 u32[4];
    f32 f32[4];
    b32 b32[4];

    i16 i16[8];
    u16 u16[8];

    i8 i8[16];
    u8 u8[16];
    b8 b8[16];

    char c[16];
  } data;
} event_context;

// Should return true if the event was handled.
typedef b8 (*PFN_on_event)(u16 code, void *sender, void *listener_instance,
                           event_context context);

b8 events_init();
void events_shutdown();

/**
 * Register to listen for when events are sent with the given code.
 * Events with duplicate listener/callback pairs will not be registered
 * again and will cause this to return FALSE.
 *
 * @param code The event code to listen for.
 * @param listener_instance A pointer to a listener instance. can be NULL.
 * @param on_event The callback to call when the event is fired.
 * @return TRUE if the event was registered, FALSE otherwise.
 */
VAPI b8 event_register(u16 code, void *listener_instance,
                       PFN_on_event on_event);

/*
 * Unregister a listener from an event. If no matching listener/callback pair
 * is found, this will return FALSE.
 *
 * @param code The event code to unregister from.
 * @param listener_instance A pointer to the listener instance. can be NULL.
 * @param on_event The callback to unregister.
 * @return TRUE if the event was unregistered, FALSE otherwise.
 */
VAPI b8 event_unregister(u16 code, void *listener_instance,
                         PFN_on_event on_event);

/*
 * Fire an event with the given code. This will call all registered listeners
 * for the event code and pass the context to them. If any listener returns
 * TRUE, the event will be considered handled and no further listeners will
 * be called.
 *
 * @param code The event code to fire.
 * @param sender The sender of the event.
 * @param context The context to pass to the listeners.
 */
VAPI b8 event_fire(u16 code, void *sender, event_context context);

// System internal event codes. Application code should use codes beyond 255.
typedef enum system_event_code {
  // Shuts the application down on the next frame.
  EVENT_CODE_APPLICATION_QUIT = 0x01,

  // Keyboard key was pressed.
  /* Context usage:
   * u16 key_code = context.data.u16[0];
   */
  EVENT_CODE_KEY_PRESSED = 0x02,

  // Keyboard key was released.
  /* Context usage:
   * u16 key_code = context.data.u16[0];
   */
  EVENT_CODE_KEY_RELEASED = 0x03,

  // Mouse button was pressed.
  /* Context usage:
   * u16 button = context.data.u16[0];
   */
  EVENT_CODE_BUTTON_PRESSED = 0x04,

  // Mouse button was released.
  /* Context usage:
   * u16 button = context.data.u16[0];
   */
  EVENT_CODE_BUTTON_RELEASED = 0x05,

  // Mouse moved.
  /* Context usage:
   * u16 x = context.data.u16[0];
   * u16 y = context.data.u16[1];
   */
  EVENT_CODE_MOUSE_MOVED = 0x06,

  // Mouse wheel moved.
  /* Context usage:
   * u8 delta = context.data.i8[0];
   */
  EVENT_CODE_MOUSE_WHEEL = 0x07,

  // Window was resized.
  /* Context usage:
   * u16 width = context.data.u16[0];
   * u16 height = context.data.u16[1];
   */
  EVENT_CODE_WINDOW_RESIZED = 0x08,

  MAX_SYSTEM_EVENT_CODE = 0xFF
} system_event_code;
