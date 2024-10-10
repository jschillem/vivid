#pragma once

#include <defines.h>

/**
 * MEMORY LAYOUT:
 *
 * +----------+----------+----------+------------------------------
 * | capacity | length   | stride   | elements         actual
 * | (u64)    | (u64)    | (u64)    | (void*)          data...
 * +----------+----------+----------+------------------------------
 *
 * capacity: number of elements that can be stored in the array.
 * length: number of elements currently in the array.
 * stride: size of each element in bytes.
 * elements: pointer to the actual data.
 */

enum { DARRAY_CAPACITY, DARRAY_LENGTH, DARRAY_STRIDE, DARRAY_FIELDS_LENGTH };

VAPI void *_darray_create(u64 capacity, u64 stride);
VAPI void _darray_destroy(void *darray);

VAPI u64 _darray_field_get(void *darray, u64 field);
VAPI void _darray_field_set(void *darray, u64 field, u64 value);

VAPI void *_darray_resize(void *darray);

VAPI void *_darray_push(void *darray, const void *element);
VAPI void _darray_pop(void *darray, void *dest);

VAPI void *_darray_insert(void *darray, u64 index, const void *element);
VAPI void _darray_remove(void *darray, u64 index, void *dest);

#define DARRAY_DEFAULT_CAPACITY 1
#define DARRAY_RESIZE_FACTOR 2

#define darray_create(type)                                                    \
  _darray_create(DARRAY_DEFAULT_CAPACITY, sizeof(type))

#define darray_reserve(type, capacity) _darray_create(capacity, sizeof(type))

#define darray_destroy(darray) _darray_destroy(darray)

#define darray_push(darray, element)                                           \
  {                                                                            \
    __auto_type _element = element;                                            \
    darray = _darray_push(darray, &_element);                                  \
  }

#define darray_pop(darray, dest) _darray_pop(darray, dest)

#define darray_insert(darray, index, element)                                  \
  {                                                                            \
    __auto_type _element = element;                                            \
    darray = _darray_insert(darray, index, &_element);                         \
  }

#define darray_remove(darray, index, dest) _darray_remove(darray, index, dest)

#define darray_clear(darray) _darray_field_set(darray, DARRAY_LENGTH, 0)

#define darray_length_set(darray, length)                                      \
  _darray_field_set(darray, DARRAY_LENGTH, length)

#define darray_capacity(darray) _darray_field_get(darray, DARRAY_CAPACITY)

#define darray_length(darray) _darray_field_get(darray, DARRAY_LENGTH)

#define darray_stride(darray) _darray_field_get(darray, DARRAY_STRIDE)
