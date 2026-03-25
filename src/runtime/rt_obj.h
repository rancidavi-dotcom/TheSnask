#ifndef RT_OBJ_H
#define RT_OBJ_H

#include "rt_base.h"

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
void s_free_obj(SnaskValue* obj_val);
void s_get_member(SnaskValue* out, SnaskValue* obj_val, SnaskValue* idx_val);
void s_set_member(SnaskValue* obj_val, SnaskValue* idx_val, SnaskValue* val);

// Internal creators
SnaskValue make_nil(void);
SnaskValue make_bool(bool b);
SnaskValue make_str(char* s);
SnaskValue make_obj(SnaskObject* o);

#endif // RT_OBJ_H
