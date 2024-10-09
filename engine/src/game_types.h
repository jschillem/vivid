#pragma once

#include <core/application.h>

typedef struct game {
  // The application configuration.
  application_config app_config;

  // Function pointer to the game's initialization function.
  b8 (*initialize)(struct game *game_instance);

  // Function pointer to the game's update function.
  b8 (*update)(struct game *game_instance, f64 delta_time);

  // Function pointer to the game's render function.
  b8 (*render)(struct game *game_instance, f64 delta_time);

  // Function pointer to handle resizing the game window. If applicable.
  b8 (*on_resize)(struct game *game_instance, u32 width, u32 height);

  // Game-specific state. Created and managed by the game.
  void *state;
} game;
