#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <time.h>
#include <dlfcn.h>
#include <sqlite3.h>
#include <zlib.h>
#include "rt_base.h"
#include "rt_obj.h"
#include "rt_gc.h"
#include "rt_json.h"

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

typedef struct SnaskZone {
    const char* name;
    SnaskOMResource** resources;
    size_t len;
    size_t cap;
    struct SnaskZone* parent;
} SnaskZone;

static __thread SnaskZone* current_zone = NULL;

#define SNASK_SIMD_ALIGN 64
#define ALIGN_UP(s, a) (((s) + (a) - 1) & ~((a) - 1))

void s_arena_reset(void) {
    arena_used = 0;
}

void s_zone_enter(const char* name) {
    SnaskZone* zone = (SnaskZone*)calloc(1, sizeof(SnaskZone));
    if (!zone) return;
    zone->name = name ? name : "<unnamed>";
    zone->cap = 8;
    zone->resources = (SnaskOMResource**)calloc(zone->cap, sizeof(SnaskOMResource*));
    if (!zone->resources) {
        free(zone);
        return;
    }
    zone->parent = current_zone;
    current_zone = zone;
}

static void s_zone_cleanup(SnaskZone* zone) {
    if (!zone) return;
    size_t live = zone->len;
    while (live > 0) {
        size_t cleaned_this_pass = 0;
        for (size_t i = zone->len; i > 0; i--) {
            SnaskOMResource* resource = zone->resources[i - 1];
            if (!resource || resource->state != SNASK_OM_RESOURCE_LIVE) continue;

            int has_live_dependent = 0;
            for (size_t j = 0; j < zone->len; j++) {
                SnaskOMResource* other = zone->resources[j];
                if (other && other->state == SNASK_OM_RESOURCE_LIVE && other->depends_on == resource) {
                    has_live_dependent = 1;
                    break;
                }
            }
            if (has_live_dependent) continue;

            if (resource->destructor && resource->c_ptr) {
                resource->destructor(resource->c_ptr);
                if (getenv("SNASK_OM_TRACE")) {
                    fprintf(stderr, "om cleanup %s in zone %s\n",
                        resource->type_name ? resource->type_name : "resource",
                        zone->name ? zone->name : "<unnamed>");
                }
            }
            resource->c_ptr = NULL;
            resource->state = SNASK_OM_RESOURCE_CLOSED;
            cleaned_this_pass++;
            live--;
        }

        if (cleaned_this_pass == 0) {
            for (size_t i = zone->len; i > 0; i--) {
                SnaskOMResource* resource = zone->resources[i - 1];
                if (!resource || resource->state != SNASK_OM_RESOURCE_LIVE) continue;
                if (resource->destructor && resource->c_ptr) resource->destructor(resource->c_ptr);
                resource->c_ptr = NULL;
                resource->state = SNASK_OM_RESOURCE_CLOSED;
                live--;
            }
            break;
        }
    }
}

void s_zone_leave(void) {
    SnaskZone* zone = current_zone;
    if (!zone) return;
    s_zone_cleanup(zone);
    current_zone = zone->parent;
    free(zone->resources);
    free(zone);
}

SnaskOMResource* s_zone_register_dep(void* c_ptr, SnaskOMDestructor destructor, const char* type_name, SnaskOMResource* depends_on) {
    if (!current_zone || !c_ptr || !destructor) return NULL;
    if (current_zone->len == current_zone->cap) {
        size_t next_cap = current_zone->cap * 2;
        SnaskOMResource** next = (SnaskOMResource**)realloc(
            current_zone->resources,
            next_cap * sizeof(SnaskOMResource*)
        );
        if (!next) return NULL;
        current_zone->resources = next;
        current_zone->cap = next_cap;
    }

    SnaskOMResource* resource = (SnaskOMResource*)calloc(1, sizeof(SnaskOMResource));
    if (!resource) return NULL;
    resource->c_ptr = c_ptr;
    resource->destructor = destructor;
    resource->state = SNASK_OM_RESOURCE_LIVE;
    resource->type_name = type_name ? type_name : "resource";
    resource->zone_name = current_zone->name;
    resource->depends_on = depends_on;
    current_zone->resources[current_zone->len++] = resource;
    return resource;
}

SnaskOMResource* s_zone_register(void* c_ptr, SnaskOMDestructor destructor, const char* type_name) {
    return s_zone_register_dep(c_ptr, destructor, type_name, NULL);
}

SnaskOMResource* s_om_resource_handle(SnaskValue* value, const char* expected_type) {
    if (!value || !value->ptr) return NULL;
    int tag = (int)value->tag;
    if (tag != SNASK_RESOURCE && tag != SNASK_BYTES) return NULL;
    SnaskOMResource* resource = (SnaskOMResource*)value->ptr;
    if (resource->state != SNASK_OM_RESOURCE_LIVE) return NULL;
    if (expected_type && resource->type_name && strcmp(resource->type_name, expected_type) != 0) return NULL;
    return resource;
}

void* s_om_resource_ptr(SnaskValue* value, const char* expected_type) {
    SnaskOMResource* resource = s_om_resource_handle(value, expected_type);
    return resource ? resource->c_ptr : NULL;
}

bool s_om_resource_release(SnaskValue* value, const char* expected_type) {
    if (!value || (int)value->tag != SNASK_RESOURCE || !value->ptr) return false;
    SnaskOMResource* resource = (SnaskOMResource*)value->ptr;
    if (resource->state != SNASK_OM_RESOURCE_LIVE) return false;
    if (expected_type && resource->type_name && strcmp(resource->type_name, expected_type) != 0) return false;
    if (resource->destructor && resource->c_ptr) {
        resource->destructor(resource->c_ptr);
    }
    resource->c_ptr = NULL;
    resource->state = SNASK_OM_RESOURCE_CLOSED;
    return true;
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

static void* sqlite_handle_to_ptr(const char* h) {
    if (!h) return NULL;
    void* p = NULL;
    sscanf(h, "%p", &p);
    return p;
}

static void om_sqlite_database_close(void* p) {
    if (p) sqlite3_close((sqlite3*)p);
}

static void om_sqlite_statement_finalize(void* p) {
    if (p) sqlite3_finalize((sqlite3_stmt*)p);
}

static sqlite3* sqlite_database_from_value(SnaskValue* value) {
    sqlite3* db = (sqlite3*)s_om_resource_ptr(value, "sqlite.Database");
    if (db) return db;
    if (!value || (int)value->tag != SNASK_STR || !value->ptr) return NULL;
    return (sqlite3*)sqlite_handle_to_ptr((const char*)value->ptr);
}

typedef struct {
    char* data;
    size_t len;
    size_t cap;
} SqlBuf;

static void sqlb_init(SqlBuf* sb) {
    sb->cap = 256;
    sb->len = 0;
    sb->data = (char*)malloc(sb->cap);
    sb->data[0] = '\0';
}

static void sqlb_append(SqlBuf* sb, const char* s) {
    if (!s) s = "";
    size_t n = strlen(s);
    if (sb->len + n + 1 > sb->cap) {
        while (sb->len + n + 1 > sb->cap) sb->cap *= 2;
        sb->data = (char*)realloc(sb->data, sb->cap);
    }
    memcpy(sb->data + sb->len, s, n);
    sb->len += n;
    sb->data[sb->len] = '\0';
}

static void sqlb_append_ch(SqlBuf* sb, char c) {
    if (sb->len + 2 > sb->cap) {
        sb->cap *= 2;
        sb->data = (char*)realloc(sb->data, sb->cap);
    }
    sb->data[sb->len++] = c;
    sb->data[sb->len] = '\0';
}

static void sqlb_append_json_string(SqlBuf* sb, const char* s) {
    sqlb_append_ch(sb, '"');
    for (const unsigned char* p = (const unsigned char*)(s ? s : ""); *p; p++) {
        unsigned char c = *p;
        if (c == '"' || c == '\\') { sqlb_append_ch(sb, '\\'); sqlb_append_ch(sb, (char)c); }
        else if (c == '\n') { sqlb_append(sb, "\\n"); }
        else if (c == '\r') { sqlb_append(sb, "\\r"); }
        else if (c == '\t') { sqlb_append(sb, "\\t"); }
        else if (c < 0x20) { sqlb_append(sb, " "); }
        else sqlb_append_ch(sb, (char)c);
    }
    sqlb_append_ch(sb, '"');
}

void sqlite_open(SnaskValue* out, SnaskValue* path) {
    if (!path || (int)path->tag != SNASK_STR || !path->ptr) { *out = MAKE_NIL(); return; }
    sqlite3* db = NULL;
    int rc = sqlite3_open((const char*)path->ptr, &db);
    if (rc != SQLITE_OK || !db) {
        if (db) sqlite3_close(db);
        *out = MAKE_NIL();
        return;
    }
    SnaskOMResource* resource = s_zone_register(db, om_sqlite_database_close, "sqlite.Database");
    if (!resource) {
        sqlite3_close(db);
        *out = MAKE_NIL();
        return;
    }
    *out = MAKE_RESOURCE(resource);
}

void sqlite_close(SnaskValue* out, SnaskValue* handle) {
    if (s_om_resource_release(handle, "sqlite.Database")) {
        *out = MAKE_BOOL(true);
        return;
    }
    if (!handle || (int)handle->tag != SNASK_STR || !handle->ptr) { *out = MAKE_NIL(); return; }
    sqlite3* db = (sqlite3*)sqlite_handle_to_ptr((const char*)handle->ptr);
    if (!db) { *out = MAKE_NIL(); return; }
    sqlite3_close(db);
    *out = MAKE_BOOL(true);
}

void sqlite_exec(SnaskValue* out, SnaskValue* handle, SnaskValue* sql) {
    if (!handle || !sql || (int)sql->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    sqlite3* db = sqlite_database_from_value(handle);
    if (!db) { *out = MAKE_NIL(); return; }
    char* err = NULL;
    int rc = sqlite3_exec(db, (const char*)sql->ptr, NULL, NULL, &err);
    if (err) sqlite3_free(err);
    *out = MAKE_BOOL(rc == SQLITE_OK);
}

void sqlite_query(SnaskValue* out, SnaskValue* handle, SnaskValue* sql) {
    if (!handle || !sql || (int)sql->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    sqlite3* db = sqlite_database_from_value(handle);
    if (!db) { *out = MAKE_NIL(); return; }

    sqlite3_stmt* stmt = NULL;
    int rc = sqlite3_prepare_v2(db, (const char*)sql->ptr, -1, &stmt, NULL);
    if (rc != SQLITE_OK || !stmt) { *out = MAKE_NIL(); return; }

    SqlBuf sb;
    sqlb_init(&sb);
    sqlb_append_ch(&sb, '[');
    bool first_row = true;
    int cols = sqlite3_column_count(stmt);

    while ((rc = sqlite3_step(stmt)) == SQLITE_ROW) {
        if (!first_row) sqlb_append_ch(&sb, ',');
        first_row = false;
        sqlb_append_ch(&sb, '{');
        for (int i = 0; i < cols; i++) {
            if (i > 0) sqlb_append_ch(&sb, ',');
            const char* col = sqlite3_column_name(stmt, i);
            sqlb_append_json_string(&sb, col ? col : "");
            sqlb_append_ch(&sb, ':');
            int t = sqlite3_column_type(stmt, i);
            if (t == SQLITE_NULL) {
                sqlb_append(&sb, "null");
            } else if (t == SQLITE_INTEGER) {
                char buf[64];
                snprintf(buf, sizeof(buf), "%lld", (long long)sqlite3_column_int64(stmt, i));
                sqlb_append(&sb, buf);
            } else if (t == SQLITE_FLOAT) {
                char buf[128];
                snprintf(buf, sizeof(buf), "%.15g", sqlite3_column_double(stmt, i));
                sqlb_append(&sb, buf);
            } else {
                const unsigned char* txt = sqlite3_column_text(stmt, i);
                sqlb_append_json_string(&sb, (const char*)txt);
            }
        }
        sqlb_append_ch(&sb, '}');
    }

    sqlite3_finalize(stmt);
    sqlb_append_ch(&sb, ']');

    SnaskValue tmp = MAKE_STR(sb.data);
    json_parse(out, &tmp);
    free(sb.data);
}

void sqlite_prepare(SnaskValue* out, SnaskValue* handle, SnaskValue* sql) {
    if (!handle || !sql || (int)sql->tag != SNASK_STR || !sql->ptr) { *out = MAKE_NIL(); return; }
    SnaskOMResource* db_resource = s_om_resource_handle(handle, "sqlite.Database");
    sqlite3* db = db_resource ? (sqlite3*)db_resource->c_ptr : sqlite_database_from_value(handle);
    if (!db) { *out = MAKE_NIL(); return; }

    sqlite3_stmt* stmt = NULL;
    int rc = sqlite3_prepare_v2(db, (const char*)sql->ptr, -1, &stmt, NULL);
    if (rc != SQLITE_OK || !stmt) {
        if (stmt) sqlite3_finalize(stmt);
        *out = MAKE_NIL();
        return;
    }

    SnaskOMResource* stmt_resource = s_zone_register_dep(
        stmt,
        om_sqlite_statement_finalize,
        "sqlite.Statement",
        db_resource
    );
    if (!stmt_resource) {
        sqlite3_finalize(stmt);
        *out = MAKE_NIL();
        return;
    }
    *out = MAKE_RESOURCE(stmt_resource);
}

void sqlite_finalize(SnaskValue* out, SnaskValue* stmt) {
    *out = MAKE_BOOL(s_om_resource_release(stmt, "sqlite.Statement"));
}

static void om_zlib_bytes_free(void* p) {
    SnaskBytes* bytes = (SnaskBytes*)p;
    if (!bytes) return;
    free(bytes->data);
    free(bytes);
}

void zlib_compress(SnaskValue* out, SnaskValue* input) {
    if (!input || (int)input->tag != SNASK_STR || !input->ptr) {
        *out = MAKE_NIL();
        return;
    }

    const unsigned char* src = (const unsigned char*)input->ptr;
    uLong src_len = (uLong)strlen((const char*)src);
    uLongf dest_len = compressBound(src_len);

    SnaskBytes* bytes = (SnaskBytes*)calloc(1, sizeof(SnaskBytes));
    if (!bytes) {
        *out = MAKE_NIL();
        return;
    }
    bytes->data = (unsigned char*)malloc(dest_len == 0 ? 1 : dest_len);
    if (!bytes->data) {
        free(bytes);
        *out = MAKE_NIL();
        return;
    }

    int rc = compress2((Bytef*)bytes->data, &dest_len, (const Bytef*)src, src_len, Z_BEST_COMPRESSION);
    if (rc != Z_OK) {
        om_zlib_bytes_free(bytes);
        *out = MAKE_NIL();
        return;
    }

    bytes->len = (size_t)dest_len;
    bytes->original_len = (size_t)src_len;

    SnaskOMResource* resource = s_zone_register(bytes, om_zlib_bytes_free, "zlib.Bytes");
    if (!resource) {
        om_zlib_bytes_free(bytes);
        *out = MAKE_NIL();
        return;
    }

    *out = MAKE_BYTES(resource);
}

void zlib_decompress(SnaskValue* out, SnaskValue* input) {
    SnaskBytes* bytes = (SnaskBytes*)s_om_resource_ptr(input, "zlib.Bytes");
    if (!bytes || !bytes->data) {
        *out = MAKE_NIL();
        return;
    }

    uLongf dest_len = (uLongf)bytes->original_len;
    char* text = (char*)snask_gc_malloc((size_t)dest_len + 1);
    if (!text) {
        *out = MAKE_NIL();
        return;
    }

    int rc = uncompress((Bytef*)text, &dest_len, (const Bytef*)bytes->data, (uLong)bytes->len);
    if (rc != Z_OK) {
        *out = MAKE_NIL();
        return;
    }
    text[dest_len] = '\0';
    *out = MAKE_STR(text);
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
