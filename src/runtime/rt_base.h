#ifndef RT_BASE_H
#define RT_BASE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// Base types for the Snask Runtime
typedef enum { 
    SNASK_NIL, 
    SNASK_NUM, 
    SNASK_BOOL, 
    SNASK_STR, 
    SNASK_OBJ,
    SNASK_RESOURCE,
    SNASK_BYTES
} SnaskType;

typedef struct {
    double tag;
    double num;
    void* ptr;
} SnaskValue;

typedef struct {
    char** names;
    SnaskValue* values;
    int count;
} SnaskObject;

typedef struct {
    unsigned char* data;
    size_t len;
    size_t original_len;
} SnaskBytes;

// Common macros for value creation
#define MAKE_NIL() (SnaskValue){ .tag = (double)SNASK_NIL, .num = 0.0, .ptr = NULL }
#define MAKE_BOOL(b) (SnaskValue){ .tag = (double)SNASK_BOOL, .num = (b) ? 1.0 : 0.0, .ptr = NULL }
#define MAKE_NUM(n) (SnaskValue){ .tag = (double)SNASK_NUM, .num = (n), .ptr = NULL }
#define MAKE_STR(s) (SnaskValue){ .tag = (double)SNASK_STR, .num = 0.0, .ptr = (s) }
#define MAKE_OBJ(o) (SnaskValue){ .tag = (double)SNASK_OBJ, .num = (double)(o)->count, .ptr = (o) }
#define MAKE_RESOURCE(r) (SnaskValue){ .tag = (double)SNASK_RESOURCE, .num = 0.0, .ptr = (r) }
#define MAKE_BYTES(r) (SnaskValue){ .tag = (double)SNASK_BYTES, .num = 0.0, .ptr = (r) }

#endif // RT_BASE_H
