#include <stdio.h>
#include <stdlib.h>
#include <time.h>

typedef struct {
    double x;
    double y;
    double z;
} Point;

int main() {
    printf("Iniciando benchmark (Heap Malloc)...\n");
    clock_t start = clock();
    double total = 0.0;
    
    for (int i = 0; i < 5000000; i++) {
        // Aloca struct Point na Heap constantemente
        Point* p = (Point*)malloc(sizeof(Point));
        p->x = 1.0;
        p->y = 2.0;
        p->z = 3.0;
        
        total += p->x + p->y + p->z;
        free(p); // C exige gerenciamento manual
    }
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    printf("Resultado C: %f\n", total);
    printf("Tempo C (Heap): %.4f s\n", time_spent);
    return 0;
}
