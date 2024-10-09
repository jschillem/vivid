#include "game.h"

#include <core/logger.h>

b8 game_init(game *game_instance) {
  VDEBUG("Game initialized");
  return TRUE;
}

b8 game_update(game *game_instance, f64 delta_time) { return TRUE; }

b8 game_render(game *game_instance, f64 delta_time) { return TRUE; }

b8 game_on_resize(game *game_instance, u32 width, u32 height) { return TRUE; }
