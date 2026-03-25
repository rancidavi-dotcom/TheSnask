#include <stdlib.h>
#include <string.h>

typedef struct {
    double tag;
    double num;
    void* ptr;
} SnaskValue;

void s_eq(SnaskValue* out, SnaskValue* a, SnaskValue* b) {
    out->tag = 2.0; // BOOL
    out->ptr = NULL;
    if (a->tag != b->tag) { out->num = 0.0; return; }
    if ((int)a->tag == 3) { // STR
        out->num = (strcmp((char*)a->ptr, (char*)b->ptr) == 0) ? 1.0 : 0.0;
    } else {
        out->num = (a->num == b->num) ? 1.0 : 0.0;
    }
}
void* snask_gc_malloc(size_t n) { return malloc(n); }
char* snask_gc_strdup(const char* s) { return strdup(s); }
