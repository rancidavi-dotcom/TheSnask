#include <stdlib.h>
#include <stdio.h>
#include <time.h>

int main() {
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);
    for (int i = 0; i < 1000000; i++) {
        volatile void* p = malloc(24);
        free((void*)p);
    }
    clock_gettime(CLOCK_MONOTONIC, &end);
    double elapsed = (double)(end.tv_sec - start.tv_sec) + (double)(end.tv_nsec - start.tv_nsec) / 1e9;
    printf("Tempo C (Manual Malloc/Free Only): %g s\n", elapsed);
    return 0;
}
