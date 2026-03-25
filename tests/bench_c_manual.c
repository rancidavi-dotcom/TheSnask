#include <stdio.h>
#include <stdlib.h>
#include <time.h>

typedef struct {
    double x;
    double y;
    double z;
} Point;

int main() {
    printf("Iniciando benchmark C (Manual Malloc/Free)...\n");
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);
    double total = 0.0;
    
    for (int i = 0; i < 5000000; i++) {
        Point* p = (Point*)malloc(sizeof(Point));
        p->x = 1.0;
        p->y = 2.0;
        p->z = 3.0;
        total += p->x + p->y + p->z;
        free(p);
    }
    
    clock_gettime(CLOCK_MONOTONIC, &end);
    double elapsed = (double)(end.tv_sec - start.tv_sec) + (double)(end.tv_nsec - start.tv_nsec) / 1e9;
    printf("Resultado C: %g\n", total);
    printf("Tempo C: %g s\n", elapsed);
    return 0;
}
