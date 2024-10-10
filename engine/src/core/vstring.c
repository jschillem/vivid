#include <core/vstring.h>

#include <core/vmemory.h>

#include <string.h>

char *vstrdup(const char *str) {
  u64 len = vstrlen(str) + 1;
  char *copy = vallocate(len, MEMORY_TAG_STRING);
  vcopy_memory(copy, str, len);

  return copy;
}

u64 vstrlen(const char *str) { return strlen(str); }
