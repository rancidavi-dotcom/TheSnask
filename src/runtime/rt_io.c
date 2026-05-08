#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <time.h>
#include "rt_base.h"
#include "rt_gc.h"
#include "rt_io.h"
#include "rt_obj.h"

// Syscall shim for __snask_write intrinsic
long s_write(long fd, const void* buf, long len) {
    return (long)write((int)fd, buf, (size_t)len);
}

void s_time(SnaskValue* out) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    out->tag = (double)SNASK_NUM;
    out->num = (double)ts.tv_sec + (double)ts.tv_nsec / 1e9;
    out->ptr = NULL;
}

// Helper para converter SnaskValue para string temporária (buffer local ou literal)
static const char* _val_to_str_tmp(SnaskValue* v, char* buf, size_t buf_size) {
    if (!v) return "nil";
    int tag = (int)v->tag;
    switch (tag) {
        case SNASK_STR: return (const char*)v->ptr ? (const char*)v->ptr : "";
        case SNASK_NIL: return "nil";
        case SNASK_BOOL: return v->num ? "true" : "false";
        case SNASK_NUM:
            snprintf(buf, buf_size, "%.15g", v->num);
            return buf;
        default: return "<obj>";
    }
}

void s_concat(SnaskValue* out, SnaskValue* s1, SnaskValue* s2) {
    char b1[64], b2[64];
    const char* str1 = _val_to_str_tmp(s1, b1, sizeof(b1));
    const char* str2 = _val_to_str_tmp(s2, b2, sizeof(b2));
    
    size_t l1 = strlen(str1);
    size_t l2 = strlen(str2);
    char* res = (char*)malloc(l1 + l2 + 1);
    if (!res) {
        *out = MAKE_NIL();
        return;
    }
    strcpy(res, str1);
    strcat(res, str2);
    *out = MAKE_STR(res);
}
