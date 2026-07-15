#include <stdio.h>
typedef struct CPU
{
    unsigned char A;
    unsigned char PC;
}CPU;

#include <stdio.h>
#include <stdint.h>

// Struct interna simples para simular o SnaskValue String da linguagem
typedef struct {
    uint8_t type_tag;
    char* data;
} SnaskString;

// Binding para a funcao print() do Snask
void s_print(SnaskString* str) {
    if (str && str->data) {
        printf("%s", str->data);
    }
}

// Binding para println()
void s_println(SnaskString* str) {
    if (str && str->data) {
        printf("%s\n", str->data);
    }
}

// Dummy snaskgui functions para evitar erro de linkage, caso algum resto tenha ficado
void snaskgui_init() {}
void snaskgui_window() {}
void snaskgui_should_close() {}
void snaskgui_present_rgba() {}
void snaskgui_poll() {}

int main() {
    printf("Snask Runtime inicializado para TUI!\n");
    return 0;
}
      
