#pragma once

#include <defines.h>

// Forward declaration of the game struct to avoid circular dependencies.
struct game;

typedef struct application_config {
  // Window starting position x axis, if applicable.
  u32 start_pos_x;
  // Window starting position y axis, if applicable.
  u32 start_pos_y;
  // Window starting width, if applicable.
  u32 width;
  // Window starting height, if applicable.
  u32 height;
  // The application name used in windowing. If applicable.
  char *name;
} application_config;

VAPI b8 application_init(struct game *game_instance);

VAPI b8 application_run();
