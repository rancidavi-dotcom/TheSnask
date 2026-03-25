#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/utsname.h>
#include <unistd.h>
#include "rt_base.h"
#include "rt_gc.h"

void num_to_str(SnaskValue* out, SnaskValue* n) {
    if (!n || (int)n->tag != SNASK_NUM) { *out = MAKE_NIL(); return; }
    char buf[128];
    snprintf(buf, sizeof(buf), "%.15g", n->num);
    *out = MAKE_STR(snask_gc_strdup(buf));
}

void str_to_num(SnaskValue* out, SnaskValue* s) {
    if (!s || (int)s->tag != SNASK_STR || !s->ptr) { *out = MAKE_NIL(); return; }
    char* end = NULL;
    double v = strtod((const char*)s->ptr, &end);
    if (end == (char*)s->ptr) { *out = MAKE_NIL(); return; }
    *out = MAKE_NUM(v);
}

void os_platform(SnaskValue* out) {
    struct utsname u;
    if (uname(&u) != 0) { *out = MAKE_NIL(); return; }
    *out = MAKE_STR(snask_gc_strdup(u.sysname));
}

void os_arch(SnaskValue* out) {
    struct utsname u;
    if (uname(&u) != 0) { *out = MAKE_NIL(); return; }
    *out = MAKE_STR(snask_gc_strdup(u.machine));
}

void os_cwd(SnaskValue* out) {
    char buf[4096];
    if (!getcwd(buf, 4096)) { *out = MAKE_NIL(); return; }
    *out = MAKE_STR(snask_gc_strdup(buf));
}

void os_getenv(SnaskValue* out, SnaskValue* key) {
    if ((int)key->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    const char* v = getenv((const char*)key->ptr);
    if (!v) { *out = MAKE_NIL(); return; }
    *out = MAKE_STR(snask_gc_strdup(v));
}

void s_exit(SnaskValue* out, SnaskValue* code) {
    int status = 0;
    if (code && (int)code->tag == SNASK_NUM) status = (int)code->num;
    exit(status);
}
