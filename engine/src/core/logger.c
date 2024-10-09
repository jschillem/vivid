#include <core/logger.h>

#include <defines.h>
#include <platform/platform.h>

// TODO: temporary
#include <stdarg.h>
#include <stdio.h>
#include <string.h>

b8 logger_init() {
  // TODO: create log file.
  return TRUE;
}
void logger_shutdown() {
  // TODO: cleanup logging/writing queued logs.
}

void log_output(log_level level, const char *message, ...) {
  const char *level_strings[6] = {"[FATAL]: ", "[ERROR]: ", "[WARN]: ",
                                  "[INFO]: ",  "[DEBUG]: ", "[TRACE]: "};
  b8 is_error = level < LOG_LEVEL_WARN;

  // NOTE: This imposes a limit on the length of the log message. However, this
  // has been set to 32000 characters, which should be more than enough for most
  // log messages.
  char out_msg[32000];
  platform_zero_memory(out_msg, sizeof(out_msg));

  __builtin_va_list args;
  va_start(args, message);
  vsnprintf(out_msg, sizeof(out_msg), message, args);
  va_end(args);

  char output[32000];
  platform_zero_memory(output, sizeof(output));
  sprintf(output, "%s%s\n", level_strings[level], out_msg);

  if (is_error) {
    platform_console_write_error(output, level);
  } else {
    platform_console_write(output, level);
  }
}
