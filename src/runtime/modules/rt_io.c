#include <unistd.h>
#include <string.h>

typedef struct {
    double tag;
    double num;
    void* ptr;
} SnaskValue;

void s_print(SnaskValue* v) {
    if ((int)v->tag == 3 && v->ptr) { // STR
        const char* s = (const char*)v->ptr;
        write(1, s, strlen(s));
        write(1, " ", 1);
    }
}

void s_println() {
    write(1, "\n", 1);
}
