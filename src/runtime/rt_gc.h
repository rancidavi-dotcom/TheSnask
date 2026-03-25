#ifndef RT_GC_H
#define RT_GC_H

#include <stddef.h>

void snask_gc_init(void);
void snask_gc_cleanup(void);
void snask_gc_track_ptr(void* p);
void* snask_gc_malloc(size_t n);
void* snask_gc_realloc(void* oldp, size_t n);
char* snask_gc_strdup(const char* s);
char* snask_gc_strndup(const char* s, size_t n);

#endif // RT_GC_H
