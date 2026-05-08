// Snask Nano Runtime - Minimalist version for --tiny builds
#include <unistd.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <stdio.h>

typedef enum { SNASK_NIL, SNASK_NUM, SNASK_BOOL, SNASK_STR, SNASK_OBJ } SnaskType;

typedef struct {
    double tag;
    double num;
    void* ptr;
} SnaskValue;

// s_write — write syscall wrapper used by intrinsic codegen
long s_write(long fd, const void* buf, long len) {
    return write((int)fd, buf, (size_t)len);
}

// num_to_str — converts number to string (called by intrinsic print codegen)
void num_to_str(SnaskValue* out, SnaskValue* n) {
    if (!n || (int)n->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; out->num = 0.0; out->ptr = NULL; return; }
    char buf[128];
    int len = snprintf(buf, sizeof(buf), "%.15g", n->num);
    if (len > 0) {
        char* s = malloc((size_t)len + 1);
        if (s) { memcpy(s, buf, (size_t)len); s[len] = '\0'; }
        out->tag = (double)SNASK_STR; out->num = 0.0; out->ptr = s;
    } else {
        out->tag = (double)SNASK_NIL; out->num = 0.0; out->ptr = NULL;
    }
}

// Minimal comparison
void s_eq(SnaskValue* out, SnaskValue* a, SnaskValue* b) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    if (a->tag != b->tag) { out->num = 0.0; return; }
    if ((int)a->tag == SNASK_STR) {
        out->num = (strcmp((char*)a->ptr, (char*)b->ptr) == 0) ? 1.0 : 0.0;
    } else {
        out->num = (a->num == b->num) ? 1.0 : 0.0;
    }
}

// Placeholder for memory (malloc)
void* snask_gc_malloc(size_t n) { return malloc(n); }
char* snask_gc_strdup(const char* s) { return strdup(s); }

// No-op GC cleanup for nano
void snask_gc_cleanup() {}

// Entry point helper for --ultra-tiny (no libc)
#ifdef SNASK_ULTRA_TINY
void _start() {
    // In ultra-tiny without libc, we would need to call main directly
    // and handle exit syscalls. This is handled by ultra_start.S
}
#endif
