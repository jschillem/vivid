#include <core/event.h>

#include <containers/darray.h>
#include <core/logger.h>
#include <core/vmemory.h>

typedef struct registered_event {
  void *listener;
  PFN_on_event callback;
} registered_event;

typedef struct event_code_entry {
  registered_event *events;
} event_code_entry;

// 16384 possible event codes, should be way more than enough.
#define MAX_MESSAGE_CODES 16384

typedef struct event_system_state {
  event_code_entry registered[MAX_MESSAGE_CODES];
} event_system_state;

/**
 * Event system internal state.
 */
static b8 is_initialized = FALSE;
static event_system_state state;

b8 events_init() {
  if (is_initialized) {
    VWARN("Event system already initialized.");
    return FALSE;
  }

  vzero_memory(&state, sizeof(event_system_state));

  is_initialized = TRUE;

  return TRUE;
}

void events_shutdown() {
  if (!is_initialized) {
    VWARN("Event system not initialized.");
    return;
  }

  for (u16 i = 0; i < MAX_MESSAGE_CODES; ++i) {
    if (state.registered[i].events) {
      darray_destroy(state.registered[i].events);
      state.registered[i].events = NULL;
    }
  }

  is_initialized = FALSE;
}

b8 event_register(u16 code, void *listener_instance, PFN_on_event on_event) {
  if (!is_initialized) {
    VERROR("Event system not initialized.");
    return FALSE;
  }

  if (state.registered[code].events == NULL) {
    state.registered[code].events = darray_create(registered_event);
  }

  u64 registered_length = darray_length(state.registered[code].events);
  for (u64 i = 0; i < registered_length; ++i) {
    if (state.registered[code].events[i].listener == listener_instance &&
        state.registered[code].events[i].callback == on_event) {
      VWARN("Event listener already registered.");
      return FALSE;
    }
  }

  registered_event event = {
      .listener = listener_instance,
      .callback = on_event,
  };

  darray_push(state.registered[code].events, event);

  return TRUE;
}

b8 event_unregister(u16 code, void *listener_instance, PFN_on_event on_event) {
  if (!is_initialized) {
    VERROR("Event system not initialized.");
    return FALSE;
  }

  if (state.registered[code].events == NULL) {
    VWARN("No events registered for code %d.", code);
    return FALSE;
  }

  u64 registered_length = darray_length(state.registered[code].events);
  for (u64 i = 0; i < registered_length; ++i) {
    if (state.registered[code].events[i].listener == listener_instance &&
        state.registered[code].events[i].callback == on_event) {
      registered_event event;
      darray_remove(state.registered[code].events, i, &event);
      return TRUE;
    }
  }

  VWARN("Event listener not found.");
  return FALSE;
}

b8 event_fire(u16 code, void *sender, event_context context) {
  if (!is_initialized) {
    VERROR("Event system not initialized.");
    return FALSE;
  }

  if (state.registered[code].events == NULL) {
    return FALSE;
  }

  u64 registered_length = darray_length(state.registered[code].events);
  for (u64 i = 0; i < registered_length; ++i) {
    if (state.registered[code].events[i].callback(
            code, sender, state.registered[code].events[i].listener, context)) {
      // Event was handled. do not call any more listeners.
      return TRUE;
    }
  }

  return FALSE;
}
