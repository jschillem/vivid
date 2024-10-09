#pragma once

#include <defines.h>

#define LOG_WARN_ENABLED 1
#define LOG_INFO_ENABLED 1

#ifdef _DEBUG
#define LOG_DEBUG_ENABLED 1
#define LOG_TRACE_ENABLED 1
#endif

typedef enum log_level {
  LOG_LEVEL_FATAL,
  LOG_LEVEL_ERROR,
  LOG_LEVEL_WARN,
  LOG_LEVEL_INFO,
  LOG_LEVEL_DEBUG,
  LOG_LEVEL_TRACE,
} log_level;

b8 logger_init();
void logger_shutdown();

VAPI void log_output(log_level level, const char *message, ...);

#define VFATAL(message, ...) log_output(LOG_LEVEL_FATAL, message, ##__VA_ARGS__)
#define VERROR(message, ...) log_output(LOG_LEVEL_ERROR, message, ##__VA_ARGS__)
#if LOG_WARN_ENABLED
#define VWARN(message, ...) log_output(LOG_LEVEL_WARN, message, ##__VA_ARGS__)
#else
#define VWARN(message, ...)
#endif
#if LOG_INFO_ENABLED
#define VINFO(message, ...) log_output(LOG_LEVEL_INFO, message, ##__VA_ARGS__)
#else
#define VINFO(message, ...)
#endif
#if LOG_DEBUG_ENABLED
#define VDEBUG(message, ...) log_output(LOG_LEVEL_DEBUG, message, ##__VA_ARGS__)
#else
#define VDEBUG(message, ...)
#endif
#if LOG_TRACE_ENABLED
#define VTRACE(message, ...) log_output(LOG_LEVEL_TRACE, message, ##__VA_ARGS__)
#else
#define VTRACE(message, ...)
#endif
