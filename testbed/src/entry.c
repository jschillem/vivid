#include "game.h"

#include <core/vmemory.h>
#include <entry.h>

// define the function to create the game
b8 create_game(game *out_game_instance) {
  out_game_instance->app_config.name = "Vivid Engine Testbed";
  out_game_instance->app_config.width = 1280;
  out_game_instance->app_config.height = 720;
  out_game_instance->app_config.start_pos_x = 100;
  out_game_instance->app_config.start_pos_y = 100;
  out_game_instance->initialize = game_init;
  out_game_instance->update = game_update;
  out_game_instance->render = game_render;
  out_game_instance->on_resize = game_on_resize;

  // Create the game state.
  out_game_instance->state = vallocate(sizeof(game_state), MEMORY_TAG_GAME);

  return TRUE;
}
