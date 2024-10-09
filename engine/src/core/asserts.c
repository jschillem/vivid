#include <core/asserts.h>

#include <core/logger.h>

void report_assertion_failure(const char *expression, const char *message,
                              const char *file, i32 line) {
  if (message == NULL) {
    VFATAL("Assertion Failure: %s\nFile: %s\nLine: %d", expression, file, line);
  } else {
    VFATAL("Assertion Failure: %s\nMessage: %s\nFile: %s\nLine: %d", expression,
           message, file, line);
  }
}
