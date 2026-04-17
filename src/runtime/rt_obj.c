#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <time.h>
#include <dlfcn.h>
#include "rt_base.h"
#include "rt_obj.h"
#include "rt_gc.h"

// Bridge to avoid implicit int truncation on x64
SnaskObject* jp_obj_new(int count);

int snask_value_strict_eq(SnaskValue* a, SnaskValue* b) {
    if (!a || !b) return 0;
    int ta = (int)a->tag;
    int tb = (int)b->tag;
    if (ta != tb) return 0;
    if (ta == SNASK_NUM || ta == SNASK_BOOL) return a->num == b->num;
    if (ta == SNASK_STR) return strcmp((char*)a->ptr, (char*)b->ptr) == 0;
    if (ta == SNASK_OBJ) return a->ptr == b->ptr;
    return 1; // NIL == NIL
}

int snask_value_eq_loose(SnaskValue* a, SnaskValue* b) {
    if (!a || !b) return 0;
    int ta = (int)a->tag;
    int tb = (int)b->tag;
    if (ta == tb) return snask_value_strict_eq(a, b);
    if (ta == SNASK_NIL || tb == SNASK_NIL) return 0;
    // NUM e BOOL
    if ((ta == SNASK_NUM || ta == SNASK_BOOL) && (tb == SNASK_NUM || tb == SNASK_BOOL))
        return a->num == b->num;
    return 0;
}

void s_eq(SnaskValue* out, SnaskValue* a, SnaskValue* b) {
    *out = MAKE_BOOL(snask_value_eq_loose(a, b));
}

void s_ne(SnaskValue* out, SnaskValue* a, SnaskValue* b) {
    *out = MAKE_BOOL(!snask_value_eq_loose(a, b));
}

void s_alloc_obj(SnaskValue* out, SnaskValue* size_val, char** names) {
    if ((int)size_val->tag != SNASK_NUM) { *out = MAKE_NIL(); return; }
    int count = (int)size_val->num;
    
    SnaskObject* obj = (SnaskObject*)malloc(sizeof(SnaskObject));
    obj->count = count;
    obj->names = names;
    obj->values = (SnaskValue*)calloc(count, sizeof(SnaskValue));
    
    *out = MAKE_OBJ(obj);
}

// --- Orchestrated Memory (Arena) v0.4.0 ---
// Using TLS (Thread-Local Storage) for Lock-Free Shadow Arenas
static __thread void* current_arena_ptr = NULL;
static __thread size_t arena_size = 0;
static __thread size_t arena_used = 0;

#define SNASK_SIMD_ALIGN 64
#define ALIGN_UP(s, a) (((s) + (a) - 1) & ~((a) - 1))

void s_arena_reset(void) {
    arena_used = 0;
}

void s_arena_alloc_obj(SnaskValue* out, SnaskValue* size_val, char** names) {
    if ((int)size_val->tag != SNASK_NUM) { *out = MAKE_NIL(); return; }
    int count = (int)size_val->num;
    
    if (!current_arena_ptr) {
        arena_size = 128 * 1024 * 1024; // 128MB per thread shadow arena
        // Use aligned_alloc for SIMD-ready base pointer
        current_arena_ptr = aligned_alloc(SNASK_SIMD_ALIGN, arena_size);
        if (!current_arena_ptr) { *out = MAKE_NIL(); return; }
        memset(current_arena_ptr, 0, arena_size);
    }
    
    size_t obj_sz = ALIGN_UP(sizeof(SnaskObject), SNASK_SIMD_ALIGN);
    size_t values_sz = ALIGN_UP(count * sizeof(SnaskValue), SNASK_SIMD_ALIGN);
    size_t total = obj_sz + values_sz;
    
    // Safety check for arena overflow -> Fallback to Heap
    if (arena_used + total > arena_size) {
        s_alloc_obj(out, size_val, names);
        return;
    }
    
    SnaskObject* obj = (SnaskObject*)((char*)current_arena_ptr + arena_used);
    arena_used += obj_sz;
    
    obj->values = (SnaskValue*)((char*)current_arena_ptr + arena_used);
    arena_used += values_sz;
    
    obj->count = count;
    obj->names = names;
    
    // Fast initialization
    for (int i = 0; i < count; i++) obj->values[i] = MAKE_NIL();
    
    *out = MAKE_OBJ(obj);
}

void s_get_member(SnaskValue* out, SnaskValue* obj_val, SnaskValue* idx_val) {
    if (!obj_val || (int)obj_val->tag != SNASK_OBJ) { *out = MAKE_NIL(); return; }
    if (!idx_val || (int)idx_val->tag != SNASK_NUM) { *out = MAKE_NIL(); return; }
    
    SnaskObject* obj = (SnaskObject*)obj_val->ptr;
    int idx = (int)idx_val->num;
    
    if (idx < 0 || idx >= obj->count) { *out = MAKE_NIL(); return; }
    *out = obj->values[idx];
}

void s_set_member(SnaskValue* obj_val, SnaskValue* idx_val, SnaskValue* val) {
    if (!obj_val || (int)obj_val->tag != SNASK_OBJ) return;
    if (!idx_val || (int)idx_val->tag != SNASK_NUM) return;
    
    SnaskObject* obj = (SnaskObject*)obj_val->ptr;
    int idx = (int)idx_val->num;
    
    if (idx < 0 || idx >= obj->count) return;
    obj->values[idx] = *val;
}

void s_promote(SnaskValue* out, SnaskValue* val) {
    if (!val || (int)val->tag != SNASK_OBJ || !val->ptr) {
        *out = val ? *val : MAKE_NIL();
        return;
    }
    
    SnaskObject* old_obj = (SnaskObject*)val->ptr;
    int count = old_obj->count;
    
    // Allocate in Managed Heap (tracked by GC)
    SnaskObject* new_obj = (SnaskObject*)snask_gc_malloc(sizeof(SnaskObject));
    new_obj->count = count;
    new_obj->names = old_obj->names; // Names are usually static/constants
    new_obj->values = (SnaskValue*)snask_gc_malloc(count * sizeof(SnaskValue));
    
    for (int i = 0; i < count; i++) {
        SnaskValue* old_v = &old_obj->values[i];
        if ((int)old_v->tag == SNASK_OBJ) {
            // Recursive promotion for deep graph preservation
            s_promote(&new_obj->values[i], old_v);
        } else {
            new_obj->values[i] = *old_v;
        }
    }
    
    *out = MAKE_OBJ(new_obj);
}

SnaskValue make_nil(void) { return MAKE_NIL(); }
SnaskValue make_bool(bool b) { return MAKE_BOOL(b); }
SnaskValue make_str(char* s) { return MAKE_STR(s); }
SnaskValue make_obj(SnaskObject* o) { return MAKE_OBJ(o); }

static int snask_name_is_index(const char* name, int idx) {
    if (!name || *name == '\0') return 0;
    char buf[32];
    snprintf(buf, sizeof(buf), "%d", idx);
    return strcmp(name, buf) == 0;
}

void is_nil(SnaskValue* out, SnaskValue* val) {
    *out = MAKE_BOOL(val->tag == SNASK_NIL);
}

void s_len(SnaskValue* out, SnaskValue* val) {
    if (!val) { *out = MAKE_NUM(0); return; }
    if (val->tag == SNASK_STR) *out = MAKE_NUM((double)strlen((char*)val->ptr));
    else if (val->tag == SNASK_OBJ && val->ptr) *out = MAKE_NUM((double)((SnaskObject*)val->ptr)->count);
    else *out = MAKE_NUM(0);
}

void sqlite_query(SnaskValue* out, SnaskValue* h, SnaskValue* sql) {
    if (sql && sql->ptr) printf("ℹ️ [ZENITH SQL] %s\n", (char*)sql->ptr);
    SnaskObject* r = jp_obj_new(0);
    *out = MAKE_OBJ(r);
}

void json_get(SnaskValue* out, SnaskValue* obj_val, SnaskValue* idx_val) {
    if (obj_val->tag == SNASK_STR && obj_val->ptr && idx_val->tag == SNASK_NUM) {
        const char* s = (const char*)obj_val->ptr;
        int idx = (int)idx_val->num;
        int len = (int)strlen(s);
        if (idx < 0 || idx >= len) { *out = MAKE_NIL(); return; }
        char buf[2] = { s[idx], '\0' };
        *out = MAKE_STR(snask_gc_strdup(buf));
        return;
    }
    if (obj_val->tag != SNASK_OBJ || !obj_val->ptr) { *out = MAKE_NIL(); return; }
    SnaskObject* obj = (SnaskObject*)obj_val->ptr;
    
    if (idx_val->tag == SNASK_NUM) {
        int idx = (int)idx_val->num;
        if (idx < 0 || idx >= obj->count) { *out = MAKE_NIL(); return; }
        *out = obj->values[idx];
    } else if (idx_val->tag == SNASK_STR) {
        const char* key = (const char*)idx_val->ptr;
        for (int i = 0; i < obj->count; i++) {
            if (obj->names[i] && strcmp(obj->names[i], key) == 0) {
                *out = obj->values[i]; return;
            }
        }
        *out = MAKE_NIL();
    } else *out = MAKE_NIL();
}

void snask_iter_get(SnaskValue* out, SnaskValue* obj_val, SnaskValue* idx_val) {
    if (!obj_val || !idx_val) { *out = MAKE_NIL(); return; }
    if (obj_val->tag == SNASK_STR) {
        json_get(out, obj_val, idx_val);
        return;
    }
    if (obj_val->tag != SNASK_OBJ || !obj_val->ptr || idx_val->tag != SNASK_NUM) {
        *out = MAKE_NIL();
        return;
    }

    SnaskObject* obj = (SnaskObject*)obj_val->ptr;
    int idx = (int)idx_val->num;
    if (idx < 0 || idx >= obj->count) { *out = MAKE_NIL(); return; }

    if (obj->names && obj->names[idx] && !snask_name_is_index(obj->names[idx], idx)) {
        *out = MAKE_STR(snask_gc_strdup(obj->names[idx]));
        return;
    }

    *out = obj->values[idx];
}

void json_set(SnaskValue* out, SnaskValue* obj_val, SnaskValue* idx_val, SnaskValue* val) {
    if (obj_val->tag != SNASK_OBJ || !obj_val->ptr) { *out = MAKE_NIL(); return; }
    SnaskObject* obj = (SnaskObject*)obj_val->ptr;
    
    if (idx_val->tag == SNASK_NUM) {
        int idx = (int)idx_val->num;
        if (idx >= 0 && idx < obj->count) {
            obj->values[idx] = *val;
            *out = *val;
        } else *out = MAKE_NIL();
    } else if (idx_val->tag == SNASK_STR) {
        const char* key = (const char*)idx_val->ptr;
        for (int i = 0; i < obj->count; i++) {
            if (obj->names[i] && strcmp(obj->names[i], key) == 0) {
                obj->values[i] = *val; *out = *val; return;
            }
        }
        // Dynamically add new member
        int nc = obj->count + 1;
        obj->names = (char**)realloc(obj->names, (size_t)nc * sizeof(char*));
        obj->values = (SnaskValue*)realloc(obj->values, (size_t)nc * sizeof(SnaskValue));
        obj->names[obj->count] = snask_gc_strdup(key);
        obj->values[obj->count] = *val;
        obj->count = nc;
        *out = *val;
    } else *out = MAKE_NIL();
}

void s_call_by_name(SnaskValue* out, SnaskValue* name_val, SnaskValue* arg1, SnaskValue* arg2, SnaskValue* arg3) {
    if (name_val->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    char raw_name[512];
    strncpy(raw_name, (char*)name_val->ptr, 512);
    
    // Replace "::" with "_NS_" to match compiler sanitization
    char sym_buf[512];
    char* p = raw_name;
    char* d = sym_buf;
    while (*p && (d - sym_buf) < 500) {
        if (*p == ':' && *(p+1) == ':') {
            *d++ = '_'; *d++ = 'N'; *d++ = 'S'; *d++ = '_';
            p += 2;
        } else {
            *d++ = *p++;
        }
    }
    *d = '\0';

    char sym[512];
    snprintf(sym, 512, "f_%s", sym_buf);
    void* fp = dlsym(RTLD_DEFAULT, sym);
    if (!fp) {
        *out = MAKE_NIL(); return;
    }
    typedef void (*SnaskFn3)(SnaskValue*, SnaskValue*, SnaskValue*, SnaskValue*);
    SnaskFn3 f = (SnaskFn3)fp;
    f(out, arg1, arg2, arg3);
}
