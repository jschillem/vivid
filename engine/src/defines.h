#pragma once

// unsigned int types
typedef unsigned char u8;
typedef unsigned short u16;
typedef unsigned int u32;
typedef unsigned long long u64;

// signed int types
typedef signed char i8;
typedef signed short i16;
typedef signed int i32;
typedef signed long long i64;

// float types
typedef float f32;
typedef double f64;

// boolean type
typedef char b8;
typedef int b32;

#define NULL ((void *)0)

// #if defined(__clang__) || defined(__gcc__)
// #define STATIC_ASSERT _Static_assert
// #else
// #define STATIC_ASSERT static_assert
// #endif

#define STATIC_ASSERT(expr, message)                                           \
  typedef char static_assertion_##message[(expr) ? 1 : -1]

#define TRUE 1
#define FALSE 0

// platform detection
#if defined(WIN32) || defined(_WIN32) || defined(__WIN32__)
#define VPLATFORM_WINDOWS 1
#ifndef _WIN64
#error "64-bit Windows is required"
#endif
#elif defined(__linux__) || defined(__gnu_linux__)
#define VPLATFORM_LINUX 1
#ifdef __ANDROID__
#define VPLATFORM_ANDROID 1
#endif
#elif defined(__unix__)
#define VPLATFORM_UNIX 1
#elif defined(_POSIX_VERSION)
#define VPLATFORM_POSIX 1
#elif __APPLE__
#define VPLATFORM_APPLE 1
#include <TargetConditionals.h>
#if TARGET_IPHONE_SIMULATOR
#define VPLATFORM_IOS 1
#define VPLATFORM_IOS_SIMULATOR 1
#elif TARGET_OS_IPHONE
#define VPLATFORM_IOS 1
// iOS device
#elif TARGET_OS_MAC
// other kinds of Mac OS
#else
#error "Unknown Apple platform"
#endif
#else
#error "Unknown platform"
#endif

#ifdef VEXPORT
#ifdef _MSC_VER
#define VAPI __declspec(dllexport)
#else
#define VAPI __attribute__((visibility("default")))
#endif
#else
// Import
#ifdef _MSC_VER
#define VAPI __declspec(dllimport)
#else
#define VAPI
#endif
#endif

// static assertions
STATIC_ASSERT(sizeof(u8) == 1, u8_size);
STATIC_ASSERT(sizeof(u16) == 2, u16_size);
STATIC_ASSERT(sizeof(u32) == 4, u32_size);
STATIC_ASSERT(sizeof(u64) == 8, u64_size);

STATIC_ASSERT(sizeof(i8) == 1, i8_size);
STATIC_ASSERT(sizeof(i16) == 2, i16_size);
STATIC_ASSERT(sizeof(i32) == 4, i32_size);
STATIC_ASSERT(sizeof(i64) == 8, i64_size);

STATIC_ASSERT(sizeof(f32) == 4, f32_size);
STATIC_ASSERT(sizeof(f64) == 8, f64_size);

STATIC_ASSERT(sizeof(b8) == 1, b8_size);
STATIC_ASSERT(sizeof(b32) == 4, b32_size);
