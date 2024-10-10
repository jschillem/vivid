#include <core/application.h>

#include <core/event.h>
#include <core/input.h>
#include <core/logger.h>
#include <core/vmemory.h>
#include <game_types.h>
#include <platform/platform.h>

// TODO: Custom string library.
#include <string.h>

typedef struct application_state {
  game *game_instance;
  b8 is_running;
  b8 is_suspended;
  platform_state platform;
  u32 width;
  u32 height;
  f64 last_frame_time;
} application_state;

static b8 is_initialized = FALSE;
static application_state app_state;

b8 application_on_event(u16 code, void *sender, void *listener_instance,
                        event_context context);

b8 application_on_key(u16 code, void *sender, void *listener_instance,
                      event_context context);

b8 application_init(game *game_instance) {
  if (is_initialized) {
    VERROR("application_init called more than once");
    return FALSE;
  }

  app_state.game_instance = game_instance;

  app_state.is_running = TRUE;
  app_state.is_suspended = FALSE;

  /* initialize the subsystems. */

  if (platform_init(&app_state.platform, game_instance->app_config.name,
                    game_instance->app_config.start_pos_x,
                    game_instance->app_config.start_pos_y,
                    game_instance->app_config.width,
                    game_instance->app_config.height)) {
    VINFO("Platform initialized successfully.");
  } else {
    VFATAL("Platform failed to initialize.");
    return FALSE;
  }

  if (logger_init()) {
    VINFO("Logger initialized successfully.");
  } else {
    VERROR("Failed to initialize logger.");
    return FALSE;
  }

  if (events_init()) {
    VINFO("Event system initialized successfully.");
  } else {
    VERROR("Failed to initialize event system.");
    return FALSE;
  }

  input_init();
  VINFO("Input system initialized.");

  event_register(EVENT_CODE_APPLICATION_QUIT, NULL, application_on_event);
  event_register(EVENT_CODE_KEY_PRESSED, NULL, application_on_key);
  event_register(EVENT_CODE_KEY_RELEASED, NULL, application_on_key);

  // TODO: Remove these messages.
  VFATAL("A test fatal log message: %d", 42);
  VERROR("A test error log message: %d", 42);
  VWARN("A test warn log message: %d", 42);
  VINFO("A test info log message: %d", 42);
  VDEBUG("A test debug log message: %d", 42);
  VTRACE("A test trace log message: %d", 42);

  // Initialize the game.
  if (!app_state.game_instance->initialize(app_state.game_instance)) {
    VFATAL("Failed to initialize the game.");
    return FALSE;
  }

  app_state.game_instance->on_resize(app_state.game_instance, app_state.width,
                                     app_state.height);

  is_initialized = TRUE;

  return TRUE;
}

b8 application_run() {
  char *memory_usage = get_memory_usage_string();

  VINFO(memory_usage);

  vfree(memory_usage, strlen(memory_usage) + 1, MEMORY_TAG_STRING);

  while (app_state.is_running) {
    if (!platform_pump_messages(&app_state.platform)) {
      app_state.is_running = FALSE;
      break;
    }

    if (!app_state.is_suspended) {
      if (!app_state.game_instance->update(app_state.game_instance, 0.0f)) {
        VFATAL("Game update failed. Exiting.");
        app_state.is_running = FALSE;
        break;
      }

      if (!app_state.game_instance->render(app_state.game_instance, 0.0f)) {
        VFATAL("Game render failed. Exiting.");
        app_state.is_running = FALSE;
        break;
      }

      // NOTE: Input update/state copying should always be handleled
      // after any input should be recorded; I.E. before this line.
      // As a safety, input is the last thing to be updated before
      // the next frame.
      input_update(0.0f);
    }
  }

  app_state.is_running = FALSE;

  event_unregister(EVENT_CODE_APPLICATION_QUIT, NULL, application_on_event);
  event_unregister(EVENT_CODE_KEY_PRESSED, NULL, application_on_key);
  event_unregister(EVENT_CODE_KEY_RELEASED, NULL, application_on_key);

  input_shutdown();
  VINFO("Input system shutdown.");

  events_shutdown();
  VINFO("Event system shutdown.");

  logger_shutdown();
  VINFO("Logger shutdown.");

  platform_shutdown(&app_state.platform);
  VINFO("Platform shutdown.");

  return TRUE;
}

b8 application_on_event(u16 code, void *sender, void *listener_instance,
                        event_context context) {
  switch (code) {
  case EVENT_CODE_APPLICATION_QUIT:
    VINFO("Application quit event received. Shutting down.");
    app_state.is_running = FALSE;
    return TRUE;
  }

  return FALSE;
}

b8 application_on_key(u16 code, void *sender, void *listener_instance,
                      event_context context) {
  if (code == EVENT_CODE_KEY_PRESSED) {
    keys key = (keys)context.data.u16[0];

    if (key == KEY_ESCAPE) {
      // NOTE: Technically firing an event to itself, but there might be many
      // different listener instances.
      event_context ctx = {};
      event_fire(EVENT_CODE_APPLICATION_QUIT, NULL, ctx);

      // Return TRUE to indicate that the event was handled.
      return TRUE;
    } else if (key == KEY_A) {
      VDEBUG("[EXPLICIT] Key A was pressed.");
    } else {
      VDEBUG("Key %c was pressed.", key);
    }
  } else if (code == EVENT_CODE_KEY_RELEASED) {
    keys key = (keys)context.data.u16[0];

    if (key == KEY_B) {
      VDEBUG("[EXPLICIT] Key B was released.");
    } else {
      VDEBUG("Key %c was released.", key);
    }
  }

  return FALSE;
}
