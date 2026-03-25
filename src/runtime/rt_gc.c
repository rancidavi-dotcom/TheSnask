#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <stdbool.h>
#include "rt_gc.h"

// Static state for garbage collection tracking
static pthread_mutex_t snask_gc_mu = PTHREAD_MUTEX_INITIALIZER;
static void** snask_gc_ptrs = NULL;
static size_t snask_gc_len = 0;
static size_t snask_gc_cap = 0;
static bool snask_gc_inited = false;

void snask_gc_cleanup(void) {
    pthread_mutex_lock(&snask_gc_mu);
    if (snask_gc_ptrs) {
        for (size_t i = 0; i < snask_gc_len; i++) {
            if (snask_gc_ptrs[i]) free(snask_gc_ptrs[i]);
        }
        free(snask_gc_ptrs);
        snask_gc_ptrs = NULL;
    }
    snask_gc_len = 0;
    snask_gc_cap = 0;
    pthread_mutex_unlock(&snask_gc_mu);
}

static void snask_gc_init_if_needed(void) {
    if (snask_gc_inited) return;
    snask_gc_inited = true;
    atexit(snask_gc_cleanup);
}

void snask_gc_init(void) {
    snask_gc_init_if_needed();
}

void snask_gc_track_ptr(void* p) {
    if (!p) return;
    snask_gc_init_if_needed();
    pthread_mutex_lock(&snask_gc_mu);
    if (snask_gc_len == snask_gc_cap) {
        size_t new_cap = snask_gc_cap ? snask_gc_cap * 2 : 1024;
        void** n = (void**)realloc(snask_gc_ptrs, new_cap * sizeof(void*));
        if (!n) { 
            pthread_mutex_unlock(&snask_gc_mu); 
            return; 
        }
        snask_gc_ptrs = n;
        snask_gc_cap = new_cap;
    }
    snask_gc_ptrs[snask_gc_len++] = p;
    pthread_mutex_unlock(&snask_gc_mu);
}

void* snask_gc_realloc(void* oldp, size_t n) {
    snask_gc_init_if_needed();
    void* newp = realloc(oldp, n);
    if (!newp) return NULL;
    
    pthread_mutex_lock(&snask_gc_mu);
    if (snask_gc_ptrs) {
        for (size_t i = 0; i < snask_gc_len; i++) {
            if (snask_gc_ptrs[i] == oldp) {
                snask_gc_ptrs[i] = newp;
                pthread_mutex_unlock(&snask_gc_mu);
                return newp;
            }
        }
    }
    pthread_mutex_unlock(&snask_gc_mu);
    
    snask_gc_track_ptr(newp);
    return newp;
}

void* snask_gc_malloc(size_t n) {
    snask_gc_init_if_needed();
    void* p = malloc(n);
    snask_gc_track_ptr(p);
    return p;
}

char* snask_gc_strdup(const char* s) {
    if (!s) return NULL;
    snask_gc_init_if_needed();
    char* p = strdup(s);
    snask_gc_track_ptr(p);
    return p;
}

char* snask_gc_strndup(const char* s, size_t n) {
    if (!s) return NULL;
    snask_gc_init_if_needed();
    char* p = (char*)malloc(n + 1);
    if (!p) return NULL;
    memcpy(p, s, n);
    p[n] = '\0';
    snask_gc_track_ptr(p);
    return p;
}
