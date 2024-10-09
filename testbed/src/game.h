#pragma once

#include <defines.h>
#include <game_types.h>

typedef struct game_state {
  f64 delta_time;
} game_state;

b8 game_init(game *game_instance);

b8 game_update(game *game_instance, f64 delta_time);

b8 game_render(game *game_instance, f64 delta_time);

b8 game_on_resize(game *game_instance, u32 width, u32 height);
