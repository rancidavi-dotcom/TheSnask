#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <time.h>
#include "rt_base.h"
#include "rt_gc.h"
#include "rt_io.h"

void s_print(SnaskValue* v) {
    if (!v) {
        printf("nil");
        return;
    }
    int tag = (int)v->tag;
#ifdef SNASK_TINY
    if (tag == SNASK_STR && v->ptr) {
        const char* s = (const char*)v->ptr;
        write(1, s, strlen(s));
        return;
    }
    if (tag == SNASK_BOOL) {
        const char* s = v->num ? "true" : "false";
        write(1, s, strlen(s));
        return;
    }
    if (tag == SNASK_NIL) {
        write(1, "nil", 3);
        return;
    }
    if (tag == SNASK_NUM) {
        char buf[64];
        int n = snprintf(buf, sizeof(buf), "%.15g", v->num);
        if (n > 0) write(1, buf, (size_t)n);
        return;
    }
    write(1, "<obj>", 5);
#else
    if (tag == SNASK_NUM) printf("%.15g", v->num);
    else if (tag == SNASK_STR) printf("%s", (char*)v->ptr ? (char*)v->ptr : "");
    else if (tag == SNASK_BOOL) printf("%s", v->num ? "true" : "false");
    else if (tag == SNASK_OBJ) printf("<obj at %p>", v->ptr);
    else printf("nil");
#endif
}

void s_println(void) {
#ifdef SNASK_TINY
    write(1, "\n", 1);
#else
    printf("\n"); fflush(stdout);
#endif
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
