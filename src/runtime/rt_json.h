#ifndef RT_JSON_H
#define RT_JSON_H

#include "rt_base.h"

// Standard JSON
void json_stringify(SnaskValue* out, SnaskValue* v);
void json_stringify_pretty(SnaskValue* out, SnaskValue* v);
void json_parse(SnaskValue* out, SnaskValue* data);

// SJSON / SNIF (Snask Interchange Format)
void sjson_new_object(SnaskValue* out);
void sjson_new_array(SnaskValue* out);
void sjson_parse_ex(SnaskValue* out, SnaskValue* text_val);
void sjson_type(SnaskValue* out, SnaskValue* v);
void sjson_arr_len(SnaskValue* out, SnaskValue* arr);
void sjson_arr_get(SnaskValue* out, SnaskValue* arr, SnaskValue* idx_val);
void sjson_arr_set(SnaskValue* out, SnaskValue* arr, SnaskValue* idx_val, SnaskValue* value);
void sjson_arr_push(SnaskValue* out, SnaskValue* arr, SnaskValue* value);
void sjson_path_get(SnaskValue* out, SnaskValue* root, SnaskValue* path);

// SNIF aliases
void snif_new_object(SnaskValue* out);
void snif_new_array(SnaskValue* out);
void snif_parse_ex(SnaskValue* out, SnaskValue* text_val);
void snif_type(SnaskValue* out, SnaskValue* v);
void snif_arr_len(SnaskValue* out, SnaskValue* arr);
void snif_arr_get(SnaskValue* out, SnaskValue* arr, SnaskValue* idx_val);
void snif_arr_set(SnaskValue* out, SnaskValue* arr, SnaskValue* idx_val, SnaskValue* value);
void snif_arr_push(SnaskValue* out, SnaskValue* arr, SnaskValue* value);
void snif_path_get(SnaskValue* out, SnaskValue* root, SnaskValue* path);

#endif // RT_JSON_H
