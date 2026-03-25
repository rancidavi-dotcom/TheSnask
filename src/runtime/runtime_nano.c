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

// Minimal printing using write(2)
void s_print(SnaskValue* v) {
    int tag = (int)v->tag;
    if (tag == SNASK_STR && v->ptr) {
        const char* s = (const char*)v->ptr;
        write(1, s, strlen(s));
    } else if (tag == SNASK_BOOL) {
        const char* s = v->num ? "true" : "false";
        write(1, s, strlen(s));
    } else if (tag == SNASK_NIL) {
        write(1, "nil", 3);
    } else if (tag == SNASK_NUM) {
        char buf[32];
        int n = snprintf(buf, sizeof(buf), "%g", v->num);
        if (n > 0) write(1, buf, n);
    }
    write(1, " ", 1);
}

void s_println() {
    write(1, "\n", 1);
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
