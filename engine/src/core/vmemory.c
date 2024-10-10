#include <core/vmemory.h>

#include <core/logger.h>
#include <core/vstring.h>
#include <platform/platform.h>

#include <stdio.h>

typedef struct memory_stats {
  u64 total_allocated;
  u64 tagged_allocations[MEMORY_TAG_MAX_COUNT];
} memory_stats;

static memory_stats stats;

static const char *memory_tag_strings[MEMORY_TAG_MAX_COUNT] = {
    "UNKNOWN     ", "ARRAY       ", "DARRAY      ", "DICT        ",
    "RING_QUEUE  ", "BST         ", "STRING      ", "APPLICATION ",
    "JOB         ", "TEXTURE     ", "MAT_INST    ", "RENDERER    ",
    "GAME        ", "TRANSFORM   ", "ENTITY      ", "ENTITY_NODE ",
    "SCENE       ",
};

void memory_init() { platform_zero_memory(&stats, sizeof(stats)); }

void memory_shutdown() {}

void *vallocate(u64 size, memory_tag tag) {
  if (tag == MEMORY_TAG_UNKNOWN) {
    VWARN("vallocate called with MEMORY_TAG_UNKNOWN. Re-classify this "
          "allocation.");
  }

  stats.total_allocated += size;
  stats.tagged_allocations[tag] += size;

  // TODO: Memory alignment.
  void *block = platform_allocate(size, FALSE);
  platform_zero_memory(block, size);

  return block;
}

void vfree(void *block, u64 size, memory_tag tag) {
  if (tag == MEMORY_TAG_UNKNOWN) {
    VWARN("vfree called with MEMORY_TAG_UNKNOWN. Re-classify this allocation.");
  }

  stats.total_allocated -= size;
  stats.tagged_allocations[tag] -= size;

  // TODO: Memory alignment.
  platform_free(block, FALSE);
}

void *vzero_memory(void *block, u64 size) {
  return platform_zero_memory(block, size);
}

void *vcopy_memory(void *dest, const void *src, u64 size) {
  return platform_copy_memory(dest, src, size);
}

void *vset_memory(void *dest, u8 value, u64 size) {
  return platform_set_memory(dest, value, size);
}

char *get_memory_usage_string() {
  const u64 gib = 1024 * 1024 * 1024;
  const u64 mib = 1024 * 1024;
  const u64 kib = 1024;

  char buffer[8192] = "System memory usage (tagged):\n";
  u64 offset = vstrlen(buffer);

  for (u32 i = 0; i < MEMORY_TAG_MAX_COUNT; ++i) {
    char unit[4] = "xiB";
    float amount = 1.0f;

    if (stats.tagged_allocations[i] >= gib) {
      amount = (float)stats.tagged_allocations[i] / (float)gib;
      unit[0] = 'G';
    } else if (stats.tagged_allocations[i] >= mib) {
      amount = (float)stats.tagged_allocations[i] / (float)mib;
      unit[0] = 'M';
    } else if (stats.tagged_allocations[i] >= kib) {
      amount = (float)stats.tagged_allocations[i] / (float)kib;
      unit[0] = 'K';
    } else {
      amount = (float)stats.tagged_allocations[i];
      unit[0] = 'B';
      unit[1] = '\0';
    }

    offset += sprintf(buffer + offset, "%s: %.2f %s\n", memory_tag_strings[i],
                      amount, unit);
  }

  char *out_string = vstrdup(buffer);
  return out_string;
}
