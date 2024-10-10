#pragma once

#include <core/application.h>
#include <core/logger.h>
#include <core/vmemory.h>
#include <game_types.h>

// Externally defined function to create a game instance.
extern b8 create_game(game *out_game_instance);

/**
 * The main entry point for the application.
 */
int main(void) {

  memory_init();

  // Request the game instance from the application.
  game game_instance;
  if (!create_game(&game_instance)) {
    VFATAL("Failed to create the game instance.");
    return -1;
  }

  // Ensure the function pointers are valid.
  if (!game_instance.initialize || !game_instance.update ||
      !game_instance.render || !game_instance.on_resize) {
    VFATAL("Game instance is missing required function pointers.");
    return -2;
  }

  // Initialize the application.
  if (!application_init(&game_instance)) {
    VFATAL("Failed to initialize the application.");
    return -3;
  }

  // Run the application. ()
  if (!application_run()) {
    VFATAL("Application did not shut down gracefully.");
    return -4;
  }

  memory_shutdown();

  vfree(game_instance.state, );

  return 0;
}
