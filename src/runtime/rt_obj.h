#ifndef RT_OBJ_H
#define RT_OBJ_H

#include "rt_base.h"

typedef void (*SnaskOMDestructor)(void*);

typedef enum {
    SNASK_OM_RESOURCE_LIVE = 1,
    SNASK_OM_RESOURCE_CLOSED = 2
} SnaskOMResourceState;

typedef struct SnaskOMResource {
    void* c_ptr;
    SnaskOMDestructor destructor;
    SnaskOMResourceState state;
    const char* type_name;
    const char* zone_name;
    struct SnaskOMResource* depends_on;
} SnaskOMResource;

// Comparisons
int snask_value_strict_eq(SnaskValue* a, SnaskValue* b);
int snask_value_eq_loose(SnaskValue* a, SnaskValue* b);

// Exported to compiler
void s_eq_strict(SnaskValue* out, SnaskValue* a, SnaskValue* b);
void s_eq(SnaskValue* out, SnaskValue* a, SnaskValue* b);
void s_ne(SnaskValue* out, SnaskValue* a, SnaskValue* b);

// Object management
void s_alloc_obj(SnaskValue* out, SnaskValue* size_val, char** names);
void s_arena_alloc_obj(SnaskValue* out, SnaskValue* size_val, char** names);
void s_arena_reset(void);
void s_zone_enter(const char* name);
void s_zone_leave(void);
SnaskOMResource* s_zone_register(void* c_ptr, SnaskOMDestructor destructor, const char* type_name);
SnaskOMResource* s_zone_register_dep(void* c_ptr, SnaskOMDestructor destructor, const char* type_name, SnaskOMResource* depends_on);
void* s_om_resource_ptr(SnaskValue* value, const char* expected_type);
SnaskOMResource* s_om_resource_handle(SnaskValue* value, const char* expected_type);
bool s_om_resource_release(SnaskValue* value, const char* expected_type);
void s_free_obj(SnaskValue* obj_val);
void s_get_member(SnaskValue* out, SnaskValue* obj_val, SnaskValue* idx_val);
void s_set_member(SnaskValue* obj_val, SnaskValue* idx_val, SnaskValue* val);
void snask_iter_get(SnaskValue* out, SnaskValue* obj_val, SnaskValue* idx_val);
void zlib_compress(SnaskValue* out, SnaskValue* input);
void zlib_decompress(SnaskValue* out, SnaskValue* input);

// Internal creators
SnaskValue make_nil(void);
SnaskValue make_bool(bool b);
SnaskValue make_str(char* s);
SnaskValue make_obj(SnaskObject* o);

#endif // RT_OBJ_H
