#pragma once

#include <defines.h>

// Tags to identify memory allocations.
typedef enum memory_tag {
  MEMORY_TAG_UNKNOWN,
  MEMORY_TAG_ARRAY,
  MEMORY_TAG_DARRAY,
  MEMORY_TAG_DICT,
  MEMORY_TAG_RING_QUEUE,
  MEMORY_TAG_BST,
  MEMORY_TAG_STRING,
  MEMORY_TAG_APPLICATION,
  MEMORY_TAG_JOB,
  MEMORY_TAG_TEXTURE,
  MEMORY_TAG_MATERIAL_INSTANCE,
  MEMORY_TAG_RENDERER,
  MEMORY_TAG_GAME,
  MEMORY_TAG_TRANSFORM,
  MEMORY_TAG_ENTITY,
  MEMORY_TAG_ENTITIY_NODE,
  MEMORY_TAG_SCENE,

  MEMORY_TAG_MAX_COUNT
} memory_tag;

void memory_init();

void memory_shutdown();

// Allocates memory of the given size and tag.
VAPI void *vallocate(u64 size, memory_tag tag);

// Frees the memory block with the given tag.
VAPI void vfree(void *block, u64 size, memory_tag tag);

// Sets the memory block to zero.
VAPI void *vzero_memory(void *block, u64 size);

// Copies memory from src to dest.
VAPI void *vcopy_memory(void *dest, const void *src, u64 size);

// Sets the memory block to the given value.
VAPI void *vset_memory(void *dest, u8 value, u64 size);

// Provides a string representation of the memory usage.
VAPI char *get_memory_usage_string();
