#pragma once

#include <defines.h>

// comment out this line to disable assertions
#define VASSERTS_ENABLED

// disable assertions for non-debug builds
#ifdef VASSERTS_ENABLED
#if _MSC_VER
#include <intrin.h>
#define debugBreak() __debugbreak()
#else
#define debugBreak() __builtin_trap()
#endif

VAPI void report_assertion_failure(const char *expression, const char *message,
                                   const char *file, i32 line);

#define VASSERT(expr)                                                          \
  {                                                                            \
    if (!(expr)) {                                                             \
      report_assertion_failure(#expr, NULL, __FILE__, __LINE__);               \
      debugBreak();                                                            \
    }                                                                          \
  }

#define VASSERT_MSG(expr, msg)                                                 \
  {                                                                            \
    if (!(expr)) {                                                             \
      report_assertion_failure(#expr, msg, __FILE__, __LINE__);                \
      debugBreak();                                                            \
    }                                                                          \
  }

#ifdef _DEBUG
#define VASSERT_DEBUG(expr) VASSERT(expr)
#define VASSERT_DEBUG_MSG(expr, msg) VASSERT_MSG(expr, msg)
#else
#define VASSERT_DEBUG(expr)
#define VASSERT_DEBUG_MSG(expr, msg)
#endif
#else
#define VASSERT(expr)
#define VASSERT_MSG(expr, msg)
#define VASSERT_DEBUG(expr)
#define VASSERT_DEBUG_MSG(expr, msg)
#endif
