#include <core/application.h>

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
    }
  }

  app_state.is_running = FALSE;

  logger_shutdown();
  VINFO("Logger shutdown.");

  platform_shutdown(&app_state.platform);
  VINFO("Platform shutdown.");

  return TRUE;
}
