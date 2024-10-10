#include <containers/darray.h>

#include <core/logger.h>
#include <core/vmemory.h>

void *_darray_create(u64 capacity, u64 stride) {
  u64 header_size = DARRAY_FIELDS_LENGTH * sizeof(u64);
  u64 data_size = capacity * stride;
  u64 *darray = vallocate(header_size + data_size, MEMORY_TAG_DARRAY);
  vzero_memory(darray, header_size + data_size);
  darray[DARRAY_CAPACITY] = capacity;
  darray[DARRAY_LENGTH] = 0;
  darray[DARRAY_STRIDE] = stride;

  return (void *)(darray + DARRAY_FIELDS_LENGTH);
}

void _darray_destroy(void *darray) {
  u64 *header = (u64 *)darray - DARRAY_FIELDS_LENGTH;
  const u64 header_size = DARRAY_FIELDS_LENGTH * sizeof(u64);
  const u64 total_size =
      header_size + header[DARRAY_CAPACITY] * header[DARRAY_STRIDE];

  vfree(header, total_size, MEMORY_TAG_DARRAY);
}

u64 _darray_field_get(void *darray, u64 field) {
  u64 *header = (u64 *)darray - DARRAY_FIELDS_LENGTH;
  return header[field];
}

void _darray_field_set(void *darray, u64 field, u64 value) {
  u64 *header = (u64 *)darray - DARRAY_FIELDS_LENGTH;
  header[field] = value;
}

void *_darray_resize(void *darray) {
  u64 length = darray_length(darray);
  u64 stride = darray_stride(darray);
  void *temp =
      _darray_create(darray_capacity(darray) * DARRAY_RESIZE_FACTOR, stride);

  vcopy_memory(temp, darray, length * stride);

  darray_length_set(temp, length);
  darray_destroy(darray);

  return temp;
}

void *_darray_push(void *darray, const void *element) {
  u64 length = darray_length(darray);
  u64 capacity = darray_capacity(darray);
  u64 stride = darray_stride(darray);

  while (length >= capacity) {
    darray = _darray_resize(darray);
    capacity = darray_capacity(darray);
  }

  void *dest = (u8 *)darray + length * stride;
  vcopy_memory(dest, element, stride);
  darray_length_set(darray, length + 1);

  return darray;
}

void _darray_pop(void *darray, void *dest) {
  u64 length = darray_length(darray);
  u64 stride = darray_stride(darray);

  if (length > 0) {
    void *src = (u8 *)darray + ((length - 1) * stride);
    vcopy_memory(dest, src, stride);
    darray_length_set(darray, length - 1);
  } else {
    VERROR("darray_pop called on empty array");
  }
}

void _darray_remove(void *darray, u64 index, void *dest) {
  u64 length = darray_length(darray);
  u64 stride = darray_stride(darray);

  if (index >= length) {
    VERROR(
        "darray_remove called with index out of bounds: length=%lu, index=%lu",
        length, index);
    return;
  }

  void *src = (u8 *)darray + (index * stride);
  vcopy_memory(dest, src, stride);

  // if the element is not the last one, move elements after it to the left
  if (index != length - 1) {
    void *dest = (u8 *)darray + (index * stride);
    void *src = (u8 *)darray + ((index + 1) * stride);
    u64 count = length - index;
    vcopy_memory(dest, src, count * stride);

    darray_length_set(darray, length - 1);
  }
}

void *_darray_insert(void *darray, u64 index, const void *element) {
  u64 length = darray_length(darray);
  u64 stride = darray_stride(darray);
  if (index >= length) {
    VERROR(
        "darray_insert called with index out of bounds: length=%lu, index=%lu",
        length, index);
    return darray;
  }

  if (length >= darray_capacity(darray)) {
    darray = _darray_resize(darray);
  }

  if (index != length - 1) {
    void *dest = (u8 *)darray + ((index + 1) * stride);
    void *src = (u8 *)darray + (index * stride);
    u64 count = length - index;
    vcopy_memory(dest, src, count * stride);
  }

  void *dest = (u8 *)darray + (index * stride);
  vcopy_memory(dest, element, stride);
  darray_length_set(darray, length + 1);

  return darray;
}
