#include <stdio.h>
#include <time.h>

typedef struct {
    double x;
    double y;
    double z;
} Point;

int main() {
    printf("Iniciando benchmark (Stack Local)...\n");
    clock_t start = clock();
    double total = 0.0;
    
    for (int i = 0; i < 5000000; i++) {
        // Aloca na Stack! Sem malloc. Perfeito para OM.
        Point p = {1.0, 2.0, 3.0};
        
        total += p.x + p.y + p.z;
    }
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    printf("Resultado C (Stack): %f\n", total);
    printf("Tempo C (Stack): %.4f s\n", time_spent);
    return 0;
}
