#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <unistd.h>
#include <time.h>
#include <math.h>
#include <ctype.h>
#include <sys/stat.h>
#include <dirent.h>
#include <errno.h>
#include <sys/utsname.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <fcntl.h>
#include <dlfcn.h>
#include <pthread.h>

// --- GUI (GTK3) ---
// Opcional: compilado quando SNASK_GUI_GTK estiver definido e os headers GTK3 existirem.
#ifdef SNASK_GUI_GTK
#include <gtk/gtk.h>
#endif

// --- SQLite ---
// Opcional: compilado quando SNASK_SQLITE estiver definido e os headers sqlite3 existirem.
#ifdef SNASK_SQLITE
#include <sqlite3.h>
#endif

typedef enum { SNASK_NIL, SNASK_NUM, SNASK_BOOL, SNASK_STR, SNASK_OBJ } SnaskType;

typedef struct {
    double tag;
    double num;
    void* ptr;
} SnaskValue;

// --- GERENCIAMENTO DE MEMÓRIA ---
// GC simples (strings/buffers): registra ponteiros heap e libera no final do processo.
// Objetivo: reduzir a rigidez/complexidade de “precisar dar free” em todo lugar.
static pthread_mutex_t snask_gc_mu = PTHREAD_MUTEX_INITIALIZER;
static void** snask_gc_ptrs = NULL;
static size_t snask_gc_len = 0;
static size_t snask_gc_cap = 0;
static bool snask_gc_inited = false;

static void snask_gc_cleanup(void) {
    pthread_mutex_lock(&snask_gc_mu);
    for (size_t i = 0; i < snask_gc_len; i++) {
        if (snask_gc_ptrs[i]) free(snask_gc_ptrs[i]);
    }
    free(snask_gc_ptrs);
    snask_gc_ptrs = NULL;
    snask_gc_len = 0;
    snask_gc_cap = 0;
    pthread_mutex_unlock(&snask_gc_mu);
}

static void snask_gc_init_if_needed(void) {
    if (snask_gc_inited) return;
    snask_gc_inited = true;
    atexit(snask_gc_cleanup);
}

static void snask_gc_track_ptr(void* p) {
    if (!p) return;
    snask_gc_init_if_needed();
    pthread_mutex_lock(&snask_gc_mu);
    if (snask_gc_len == snask_gc_cap) {
        size_t new_cap = snask_gc_cap ? snask_gc_cap * 2 : 1024;
        void** n = (void**)realloc(snask_gc_ptrs, new_cap * sizeof(void*));
        if (!n) { pthread_mutex_unlock(&snask_gc_mu); return; }
        snask_gc_ptrs = n;
        snask_gc_cap = new_cap;
    }
    snask_gc_ptrs[snask_gc_len++] = p;
    pthread_mutex_unlock(&snask_gc_mu);
}

static void* snask_gc_realloc(void* oldp, size_t n) {
    snask_gc_init_if_needed();
    void* newp = realloc(oldp, n);
    if (!newp) return NULL;
    pthread_mutex_lock(&snask_gc_mu);
    for (size_t i = 0; i < snask_gc_len; i++) {
        if (snask_gc_ptrs[i] == oldp) {
            snask_gc_ptrs[i] = newp;
            pthread_mutex_unlock(&snask_gc_mu);
            return newp;
        }
    }
    pthread_mutex_unlock(&snask_gc_mu);
    snask_gc_track_ptr(newp);
    return newp;
}

static void* snask_gc_malloc(size_t n) {
    snask_gc_init_if_needed();
    void* p = malloc(n);
    snask_gc_track_ptr(p);
    return p;
}

static char* snask_gc_strdup(const char* s) {
    if (!s) return NULL;
    snask_gc_init_if_needed();
    char* p = strdup(s);
    snask_gc_track_ptr(p);
    return p;
}

static char* snask_gc_strndup(const char* s, size_t n) {
    if (!s) return NULL;
    snask_gc_init_if_needed();
    char* p = (char*)malloc(n + 1);
    if (!p) return NULL;
    memcpy(p, s, n);
    p[n] = '\0';
    snask_gc_track_ptr(p);
    return p;
}

typedef struct {
    char** names;
    SnaskValue* values;
    int count;
} SnaskObject;

static int snask_value_strict_eq(SnaskValue* a, SnaskValue* b) {
    if (!a || !b) return 0;
    int ta = (int)a->tag;
    int tb = (int)b->tag;
    if (ta != tb) return 0;
    switch (ta) {
        case SNASK_NIL: return 1;
        case SNASK_NUM: return a->num == b->num;
        case SNASK_BOOL: return (a->num != 0.0) == (b->num != 0.0);
        case SNASK_STR:
            if (!a->ptr || !b->ptr) return a->ptr == b->ptr;
            return strcmp((const char*)a->ptr, (const char*)b->ptr) == 0;
        case SNASK_OBJ:
            // por enquanto: igualdade estrita de objeto = mesma referência
            return a->ptr == b->ptr;
        default:
            return 0;
    }
}

void s_eq_strict(SnaskValue* out, SnaskValue* a, SnaskValue* b) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = snask_value_strict_eq(a, b) ? 1.0 : 0.0;
}

static int snask_value_eq_loose(SnaskValue* a, SnaskValue* b) {
    if (!a || !b) return 0;
    int ta = (int)a->tag;
    int tb = (int)b->tag;

    // nil
    if (ta == SNASK_NIL || tb == SNASK_NIL) return ta == tb;

    // numeric-like: NUM and BOOL can be compared by numeric value
    if ((ta == SNASK_NUM || ta == SNASK_BOOL) && (tb == SNASK_NUM || tb == SNASK_BOOL)) {
        double av = (ta == SNASK_BOOL) ? (a->num != 0.0 ? 1.0 : 0.0) : a->num;
        double bv = (tb == SNASK_BOOL) ? (b->num != 0.0 ? 1.0 : 0.0) : b->num;
        return av == bv;
    }

    // strings: compare by content
    if (ta == SNASK_STR && tb == SNASK_STR) {
        if (!a->ptr || !b->ptr) return a->ptr == b->ptr;
        return strcmp((const char*)a->ptr, (const char*)b->ptr) == 0;
    }

    // objects: for now, equality = same reference (fast, predictable)
    if (ta == SNASK_OBJ && tb == SNASK_OBJ) return a->ptr == b->ptr;

    // different types: not equal
    return 0;
}

void s_eq(SnaskValue* out, SnaskValue* a, SnaskValue* b) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = snask_value_eq_loose(a, b) ? 1.0 : 0.0;
}

void s_ne(SnaskValue* out, SnaskValue* a, SnaskValue* b) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = snask_value_eq_loose(a, b) ? 0.0 : 1.0;
}

// Forward decls (usadas por seções posteriores)
static SnaskValue make_nil(void);
static SnaskValue make_bool(bool b);
static SnaskValue make_str(char* s);
static SnaskValue make_obj(SnaskObject* o);
static SnaskObject* obj_new(int count);
void json_stringify(SnaskValue* out, SnaskValue* v);
void json_parse(SnaskValue* out, SnaskValue* data);

void s_alloc_obj(SnaskValue* out, SnaskValue* size_val, char** names) {
    if ((int)size_val->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; return; }
    int count = (int)size_val->num;
    
    SnaskObject* obj = malloc(sizeof(SnaskObject));
    obj->count = count;
    obj->names = names;
    obj->values = calloc(count, sizeof(SnaskValue));
    
    out->tag = (double)SNASK_OBJ;
    out->ptr = obj;
    out->num = (double)count;
}

// --- AUXILIAR HTTP ---
void http_request(SnaskValue* out, const char* method, SnaskValue* url, SnaskValue* data) {
    if ((int)url->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    
    const char* dbg = getenv("SNASK_HTTP_DEBUG");
    char cmd[4096];
    if (data && (int)data->tag == SNASK_STR) {
        // -f: fail on HTTP errors, -sS: silent but show errors, -L: follow redirects
        // timeouts avoid hanging the runtime on network issues
        snprintf(cmd, sizeof(cmd), "curl -f -sS -L --connect-timeout 10 --max-time 30 -X %s -d '%s' '%s' 2>&1", method, (char*)data->ptr, (char*)url->ptr);
    } else {
        snprintf(cmd, sizeof(cmd), "curl -f -sS -L --connect-timeout 10 --max-time 30 -X %s '%s' 2>&1", method, (char*)url->ptr);
    }

    if (dbg && *dbg) {
        FILE* df = fopen("/tmp/snask_http_debug.log", "a");
        if (df) { fprintf(df, "CMD=%s\n", cmd); fclose(df); }
    }
    
    FILE *fp = popen(cmd, "r");
    if (!fp) { out->tag = (double)SNASK_NIL; return; }
    
    char *response = (char*)snask_gc_malloc(1);
    response[0] = '\0';
    char buffer[1024];
    size_t total_len = 0;
    
    while (fgets(buffer, sizeof(buffer), fp) != NULL) {
        size_t chunk_len = strlen(buffer);
        response = (char*)snask_gc_realloc(response, total_len + chunk_len + 1);
        strcpy(response + total_len, buffer);
        total_len += chunk_len;
    }
    
    int rc = pclose(fp);
    // curl returns non-zero on network errors / HTTP -f failures
    if (rc != 0 && total_len == 0) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    if (dbg && *dbg) {
        FILE* df = fopen("/tmp/snask_http_debug.log", "a");
        if (df) { fprintf(df, "RC=%d LEN=%zu\n", rc, total_len); fclose(df); }
    }
    out->tag = (double)SNASK_STR;
    out->ptr = response;
    out->num = 0;
}

void s_http_get(SnaskValue* out, SnaskValue* url) { http_request(out, "GET", url, NULL); }
void s_http_post(SnaskValue* out, SnaskValue* url, SnaskValue* data) { http_request(out, "POST", url, data); }
void s_http_put(SnaskValue* out, SnaskValue* url, SnaskValue* data) { http_request(out, "PUT", url, data); }
void s_http_delete(SnaskValue* out, SnaskValue* url) { http_request(out, "DELETE", url, NULL); }
void s_http_patch(SnaskValue* out, SnaskValue* url, SnaskValue* data) { http_request(out, "PATCH", url, data); }

void s_print(SnaskValue* v) {
    int tag = (int)v->tag;
    if (tag == SNASK_NUM) printf("%g ", v->num);
    else if (tag == SNASK_STR) printf("%s ", (char*)v->ptr);
    else if (tag == SNASK_BOOL) printf("%s ", v->num ? "true" : "false");
    else if (tag == SNASK_OBJ) printf("<obj at %p> ", v->ptr);
    else printf("nil ");
}

void s_println() { printf("\n"); fflush(stdout); }

void sfs_read(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    FILE *f = fopen((char*)path->ptr, "rb");
    if (!f) { out->tag = (double)SNASK_NIL; return; }
    fseek(f, 0, SEEK_END); long sz = ftell(f); fseek(f, 0, SEEK_SET);
    char *s = malloc(sz + 1); fread(s, sz, 1, f); fclose(f); s[sz] = 0;
    out->tag = (double)SNASK_STR; out->ptr = s; out->num = 0;
}

void sfs_write(SnaskValue* out, SnaskValue* path, SnaskValue* content) {
    out->tag = (double)SNASK_BOOL;
    if ((int)path->tag != SNASK_STR || (int)content->tag != SNASK_STR) { out->num = 0; return; }
    FILE *f = fopen((char*)path->ptr, "w");
    if (!f) { out->num = 0; return; }
    fprintf(f, "%s", (char*)content->ptr);
    fflush(f);
    fclose(f);
    out->num = 1;
    out->ptr = NULL;
}

void sfs_append(SnaskValue* out, SnaskValue* path, SnaskValue* content) {
    out->tag = (double)SNASK_BOOL;
    if ((int)path->tag != SNASK_STR || (int)content->tag != SNASK_STR) { out->num = 0; return; }
    FILE *f = fopen((char*)path->ptr, "a");
    if (!f) { out->num = 0; return; }
    fprintf(f, "%s", (char*)content->ptr);
    fflush(f);
    fclose(f);
    out->num = 1;
    out->ptr = NULL;
}

static bool sfs_copy_file_impl(const char* src, const char* dst) {
    FILE* in = fopen(src, "rb");
    if (!in) return false;
    FILE* out = fopen(dst, "wb");
    if (!out) { fclose(in); return false; }
    char buf[8192];
    size_t n = 0;
    while ((n = fread(buf, 1, sizeof(buf), in)) > 0) {
        if (fwrite(buf, 1, n, out) != n) { fclose(in); fclose(out); return false; }
    }
    fclose(in);
    fflush(out);
    fclose(out);
    return true;
}

void sfs_copy(SnaskValue* out, SnaskValue* src, SnaskValue* dst) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if ((int)src->tag != SNASK_STR || (int)dst->tag != SNASK_STR) return;
    out->num = sfs_copy_file_impl((const char*)src->ptr, (const char*)dst->ptr) ? 1.0 : 0.0;
}

void sfs_move(SnaskValue* out, SnaskValue* src, SnaskValue* dst) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if ((int)src->tag != SNASK_STR || (int)dst->tag != SNASK_STR) return;
    if (rename((const char*)src->ptr, (const char*)dst->ptr) == 0) { out->num = 1.0; return; }
    // fallback: copy + delete (para cross-device)
    if (sfs_copy_file_impl((const char*)src->ptr, (const char*)dst->ptr)) {
        remove((const char*)src->ptr);
        out->num = 1.0;
    }
}

void sfs_mkdir(SnaskValue* out, SnaskValue* path) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if ((int)path->tag != SNASK_STR) return;
    if (mkdir((const char*)path->ptr, 0755) == 0) { out->num = 1.0; return; }
    if (errno == EEXIST) { out->num = 1.0; return; }
}

void sfs_is_file(SnaskValue* out, SnaskValue* path) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if ((int)path->tag != SNASK_STR) return;
    struct stat st;
    if (stat((const char*)path->ptr, &st) != 0) return;
    out->num = S_ISREG(st.st_mode) ? 1.0 : 0.0;
}

void sfs_is_dir(SnaskValue* out, SnaskValue* path) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if ((int)path->tag != SNASK_STR) return;
    struct stat st;
    if (stat((const char*)path->ptr, &st) != 0) return;
    out->num = S_ISDIR(st.st_mode) ? 1.0 : 0.0;
}

// Retorna um "array" como SNASK_OBJ com chaves "0..n-1" e valores string (nomes de entrada).
void sfs_listdir(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    DIR* d = opendir((const char*)path->ptr);
    if (!d) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }

    SnaskObject* arr = (SnaskObject*)malloc(sizeof(SnaskObject));
    arr->count = 0;
    arr->names = NULL;
    arr->values = NULL;
    int cap = 0;

    struct dirent* ent;
    while ((ent = readdir(d)) != NULL) {
        const char* name = ent->d_name;
        if (strcmp(name, ".") == 0 || strcmp(name, "..") == 0) continue;
        if (arr->count >= cap) {
            int new_cap = (cap == 0) ? 16 : cap * 2;
            arr->names = (char**)realloc(arr->names, (size_t)new_cap * sizeof(char*));
            arr->values = (SnaskValue*)realloc(arr->values, (size_t)new_cap * sizeof(SnaskValue));
            for (int i = cap; i < new_cap; i++) { arr->names[i] = NULL; arr->values[i].tag = (double)SNASK_NIL; arr->values[i].ptr = NULL; arr->values[i].num = 0; }
            cap = new_cap;
        }
        char idx_name[32];
        snprintf(idx_name, sizeof(idx_name), "%d", arr->count);
        arr->names[arr->count] = snask_gc_strdup(idx_name);
        arr->values[arr->count] = (SnaskValue){(double)SNASK_STR, 0, snask_gc_strdup(name)};
        arr->count++;
    }
    closedir(d);

    out->tag = (double)SNASK_OBJ;
    out->ptr = arr;
    out->num = (double)arr->count;
}

// --- OS / PATH helpers ---
void os_cwd(SnaskValue* out) {
    char* buf = (char*)malloc(4096);
    if (!getcwd(buf, 4096)) {
        free(buf);
        out->tag = (double)SNASK_NIL;
        out->ptr = NULL;
        out->num = 0;
        return;
    }
    out->tag = (double)SNASK_STR;
    out->ptr = buf;
    out->num = 0;
}

void os_platform(SnaskValue* out) {
    struct utsname u;
    if (uname(&u) != 0) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    out->tag = (double)SNASK_STR;
    out->ptr = snask_gc_strdup(u.sysname);
    out->num = 0;
}

void os_arch(SnaskValue* out) {
    struct utsname u;
    if (uname(&u) != 0) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    out->tag = (double)SNASK_STR;
    out->ptr = snask_gc_strdup(u.machine);
    out->num = 0;
}

void os_getenv(SnaskValue* out, SnaskValue* key) {
    if ((int)key->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    const char* v = getenv((const char*)key->ptr);
    if (!v) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    out->tag = (double)SNASK_STR;
    out->ptr = snask_gc_strdup(v);
    out->num = 0;
}

void os_setenv(SnaskValue* out, SnaskValue* key, SnaskValue* value) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if ((int)key->tag != SNASK_STR || (int)value->tag != SNASK_STR) return;
    int res = setenv((const char*)key->ptr, (const char*)value->ptr, 1);
    out->num = (res == 0) ? 1.0 : 0.0;
}

void sfs_size(SnaskValue* out, SnaskValue* path) {
    out->tag = (double)SNASK_NUM;
    out->ptr = NULL;
    out->num = 0;
    if ((int)path->tag != SNASK_STR) return;
    struct stat st;
    if (stat((const char*)path->ptr, &st) != 0) return;
    out->num = (double)st.st_size;
}

void sfs_mtime(SnaskValue* out, SnaskValue* path) {
    out->tag = (double)SNASK_NUM;
    out->ptr = NULL;
    out->num = 0;
    if ((int)path->tag != SNASK_STR) return;
    struct stat st;
    if (stat((const char*)path->ptr, &st) != 0) return;
    out->num = (double)st.st_mtime;
}

void sfs_rmdir(SnaskValue* out, SnaskValue* path) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if ((int)path->tag != SNASK_STR) return;
    out->num = (rmdir((const char*)path->ptr) == 0) ? 1.0 : 0.0;
}

static const char* last_slash(const char* s) {
    const char* p = strrchr(s, '/');
    return p;
}

void path_basename(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR || !path->ptr) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    const char* s = (const char*)path->ptr;
    size_t n = strlen(s);
    while (n > 0 && s[n - 1] == '/') n--;
    if (n == 0) { out->tag = (double)SNASK_STR; out->ptr = snask_gc_strdup("/"); out->num = 0; return; }
    char* tmp = snask_gc_strndup(s, n);
    const char* ls = last_slash(tmp);
    const char* base = ls ? (ls + 1) : tmp;
    out->tag = (double)SNASK_STR;
    out->ptr = snask_gc_strdup(base);
    out->num = 0;
    free(tmp);
}

void path_dirname(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR || !path->ptr) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    const char* s = (const char*)path->ptr;
    size_t n = strlen(s);
    while (n > 0 && s[n - 1] == '/') n--;
    if (n == 0) { out->tag = (double)SNASK_STR; out->ptr = snask_gc_strdup("/"); out->num = 0; return; }
    char* tmp = snask_gc_strndup(s, n);
    char* ls = strrchr(tmp, '/');
    if (!ls) { out->tag = (double)SNASK_STR; out->ptr = snask_gc_strdup("."); out->num = 0; free(tmp); return; }
    while (ls > tmp && *ls == '/') ls--;
    size_t dn = (size_t)(ls - tmp + 1);
    if (dn == 0) dn = 1;
    out->tag = (double)SNASK_STR;
    out->ptr = snask_gc_strndup(tmp, dn);
    out->num = 0;
    free(tmp);
}

void path_extname(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR || !path->ptr) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    const char* s = (const char*)path->ptr;
    const char* ls = last_slash(s);
    const char* base = ls ? (ls + 1) : s;
    const char* dot = strrchr(base, '.');
    if (!dot || dot == base) { out->tag = (double)SNASK_STR; out->ptr = snask_gc_strdup(""); out->num = 0; return; }
    out->tag = (double)SNASK_STR;
    out->ptr = snask_gc_strdup(dot + 1);
    out->num = 0;
}

void path_join(SnaskValue* out, SnaskValue* a, SnaskValue* b) {
    if ((int)a->tag != SNASK_STR || (int)b->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    const char* sa = (const char*)a->ptr;
    const char* sb = (const char*)b->ptr;
    if (!sa) sa = "";
    if (!sb) sb = "";
    size_t la = strlen(sa);
    size_t lb = strlen(sb);
    bool a_slash = la > 0 && sa[la - 1] == '/';
    bool b_slash = lb > 0 && sb[0] == '/';
    size_t extra = (a_slash || b_slash || la == 0 || lb == 0) ? 0 : 1;
    char* res = (char*)malloc(la + lb + extra + 1);
    strcpy(res, sa);
    if (extra == 1) strcat(res, "/");
    if (a_slash && b_slash) strcat(res, sb + 1);
    else strcat(res, sb);
    out->tag = (double)SNASK_STR;
    out->ptr = res;
    out->num = 0;
}

// --- Blaze (micro web server) ---
static const char* blaze_find_method(const char* req) {
    if (!req) return NULL;
    if (strncmp(req, "GET ", 4) == 0) return "GET";
    if (strncmp(req, "POST ", 5) == 0) return "POST";
    if (strncmp(req, "PUT ", 4) == 0) return "PUT";
    if (strncmp(req, "PATCH ", 6) == 0) return "PATCH";
    if (strncmp(req, "DELETE ", 7) == 0) return "DELETE";
    return NULL;
}

static bool blaze_parse_path(const char* req, char* out_path, size_t out_cap) {
    const char* method = blaze_find_method(req);
    if (!method) return false;
    const char* p = strchr(req, ' ');
    if (!p) return false;
    p++;
    const char* end = strchr(p, ' ');
    if (!end) return false;
    size_t n = (size_t)(end - p);
    if (n + 1 > out_cap) n = out_cap - 1;
    memcpy(out_path, p, n);
    out_path[n] = '\0';
    // remove querystring
    char* q = strchr(out_path, '?');
    if (q) *q = '\0';
    return true;
}

static bool blaze_parse_target_raw(const char* req, char* out_target, size_t out_cap) {
    if (!req || !out_target || out_cap == 0) return false;
    const char* p = strchr(req, ' ');
    if (!p) return false;
    p++;
    const char* end = strchr(p, ' ');
    if (!end) return false;
    size_t n = (size_t)(end - p);
    if (n + 1 > out_cap) n = out_cap - 1;
    memcpy(out_target, p, n);
    out_target[n] = '\0';
    return true;
}

static SnaskValue blaze_route_lookup(SnaskObject* routes, const char* path, bool* found) {
    *found = false;
    if (!routes || !path) return make_nil();
    for (int i = 0; i < routes->count; i++) {
        if (routes->names[i] && strcmp(routes->names[i], path) == 0) {
            *found = true;
            return routes->values[i];
        }
    }
    return make_nil();
}

static SnaskValue blaze_obj_lookup(SnaskObject* obj, const char* key, bool* found) {
    *found = false;
    if (!obj || !key) return make_nil();
    for (int i = 0; i < obj->count; i++) {
        if (obj->names[i] && strcmp(obj->names[i], key) == 0) {
            *found = true;
            return obj->values[i];
        }
    }
    return make_nil();
}

static void blaze_send_all(int fd, const char* data) {
    if (!data) return;
    size_t len = strlen(data);
    while (len > 0) {
        ssize_t n = send(fd, data, len, 0);
        if (n <= 0) return;
        data += (size_t)n;
        len -= (size_t)n;
    }
}

static void blaze_send_response(int fd, int status, const char* content_type, const char* body) {
    if (!content_type) content_type = "text/plain; charset=utf-8";
    if (!body) body = "";
    char header[512];
    int body_len = (int)strlen(body);
    const char* status_text = (status == 200) ? "OK" : (status == 404) ? "Not Found" : "Error";
    snprintf(
        header,
        sizeof(header),
        "HTTP/1.1 %d %s\r\n"
        "Content-Type: %s\r\n"
        "Content-Length: %d\r\n"
        "Connection: close\r\n"
        "\r\n",
        status,
        status_text,
        content_type,
        body_len
    );
    blaze_send_all(fd, header);
    blaze_send_all(fd, body);
}

static void blaze_send_response_extra(int fd, int status, const char* content_type, const char* extra_header_line, const char* body) {
    if (!content_type) content_type = "text/plain; charset=utf-8";
    if (!body) body = "";
    if (!extra_header_line) extra_header_line = "";
    char header[768];
    int body_len = (int)strlen(body);
    const char* status_text = (status == 200) ? "OK"
        : (status == 302) ? "Found"
        : (status == 400) ? "Bad Request"
        : (status == 404) ? "Not Found"
        : "Error";
    snprintf(
        header,
        sizeof(header),
        "HTTP/1.1 %d %s\r\n"
        "Content-Type: %s\r\n"
        "Content-Length: %d\r\n"
        "%s"
        "Connection: close\r\n"
        "\r\n",
        status,
        status_text,
        content_type,
        body_len,
        extra_header_line
    );
    blaze_send_all(fd, header);
    blaze_send_all(fd, body);
}

static void blaze_send_response_headers(int fd, int status, const char* content_type, const char* header_line, const char* cookie_line, const char* body) {
    char extra[1024];
    extra[0] = '\0';
    if (header_line && header_line[0]) {
        strncat(extra, header_line, sizeof(extra) - strlen(extra) - 1);
        strncat(extra, "\r\n", sizeof(extra) - strlen(extra) - 1);
    }
    if (cookie_line && cookie_line[0]) {
        strncat(extra, "Set-Cookie: ", sizeof(extra) - strlen(extra) - 1);
        strncat(extra, cookie_line, sizeof(extra) - strlen(extra) - 1);
        strncat(extra, "\r\n", sizeof(extra) - strlen(extra) - 1);
    }
    blaze_send_response_extra(fd, status, content_type, extra, body);
}

static SnaskValue make_str_dup(const char* s) {
    if (!s) return make_nil();
    return (SnaskValue){(double)SNASK_STR, 0, snask_gc_strdup(s)};
}

static SnaskValue make_num_val(double n) { return (SnaskValue){(double)SNASK_NUM, n, NULL}; }

static SnaskValue blaze_call_snask_handler(const char* handler_name, const char* method, const char* path, const char* query, const char* body, const char* cookie_header) {
    if (!handler_name) return make_nil();
    char sym[512];
    snprintf(sym, sizeof(sym), "f_%s", handler_name);
    void* fp = dlsym(NULL, sym);
    if (!fp) return make_nil();

    typedef void (*SnaskFn6)(SnaskValue* ra, SnaskValue* a1, SnaskValue* a2, SnaskValue* a3, SnaskValue* a4, SnaskValue* a5);
    SnaskFn6 f = (SnaskFn6)fp;

    SnaskValue ra = make_nil();
    SnaskValue m = make_str_dup(method ? method : "");
    SnaskValue p = make_str_dup(path ? path : "");
    SnaskValue q = make_str_dup(query ? query : "");
    SnaskValue b = make_str_dup(body ? body : "");
    SnaskValue c = make_str_dup(cookie_header ? cookie_header : "");

    f(&ra, &m, &p, &q, &b, &c);
    return ra;
}

static const char* blaze_find_header(const char* req, const char* key) {
    if (!req || !key) return NULL;
    const char* headers = strstr(req, "\r\n");
    if (!headers) return NULL;
    headers += 2;
    size_t klen = strlen(key);
    const char* p = headers;
    while (*p) {
        const char* eol = strstr(p, "\r\n");
        if (!eol) break;
        if (eol == p) break; // end headers
        if (strncasecmp(p, key, klen) == 0 && p[klen] == ':') {
            const char* v = p + klen + 1;
            while (*v == ' ') v++;
            return v;
        }
        p = eol + 2;
    }
    return NULL;
}

static int blaze_parse_content_length(const char* req) {
    const char* v = blaze_find_header(req, "Content-Length");
    if (!v) return 0;
    return atoi(v);
}

static char* blaze_extract_query(const char* req) {
    char target[2048];
    if (!blaze_parse_target_raw(req, target, sizeof(target))) return snask_gc_strdup("");
    const char* q = strchr(target, '?');
    if (!q) return snask_gc_strdup("");
    return snask_gc_strdup(q + 1);
}

static char* blaze_extract_path_only(const char* req) {
    char target[2048];
    if (!blaze_parse_target_raw(req, target, sizeof(target))) return snask_gc_strdup("/");
    char* q = strchr(target, '?');
    if (q) *q = '\0';
    return snask_gc_strdup(target);
}

static char* blaze_extract_cookie_header(const char* req) {
    const char* v = blaze_find_header(req, "Cookie");
    if (!v) return snask_gc_strdup("");
    const char* eol = strstr(v, "\r\n");
    if (!eol) return snask_gc_strdup(v);
    size_t n = (size_t)(eol - v);
    char* tmp = (char*)malloc(n + 1);
    memcpy(tmp, v, n);
    tmp[n] = '\0';
    char* out = snask_gc_strdup(tmp);
    free(tmp);
    return out;
}

// blaze_run(port, routes)
// routes: SNASK_OBJ onde keys são paths ("/", "/ping", "/users") e values são strings ou objetos (viram JSON).
void blaze_run(SnaskValue* out, SnaskValue* port_val, SnaskValue* routes_val) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if ((int)port_val->tag != SNASK_NUM || (int)routes_val->tag != SNASK_OBJ) return;
    int port = (int)port_val->num;
    if (port <= 0 || port > 65535) return;

    int server_fd = socket(AF_INET, SOCK_STREAM, 0);
    if (server_fd < 0) return;
    int opt = 1;
    setsockopt(server_fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));

    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_ANY);
    addr.sin_port = htons((uint16_t)port);
    if (bind(server_fd, (struct sockaddr*)&addr, sizeof(addr)) != 0) { close(server_fd); return; }
    if (listen(server_fd, 64) != 0) { close(server_fd); return; }

    // Servidor iniciado com sucesso (vai bloquear no loop).
    out->num = 1.0;

    SnaskObject* routes = (SnaskObject*)routes_val->ptr;
    for (;;) {
        int client_fd = accept(server_fd, NULL, NULL);
        if (client_fd < 0) continue;

        char req[16384];
        ssize_t n = recv(client_fd, req, sizeof(req) - 1, 0);
        if (n <= 0) { close(client_fd); continue; }
        req[n] = '\0';

        // Se houver body, tenta ler o restante baseado em Content-Length
        int content_len = blaze_parse_content_length(req);
        const char* hdr_end = strstr(req, "\r\n\r\n");
        int have_body = 0;
        if (hdr_end) have_body = (int)(n - (hdr_end + 4 - req));
        while (hdr_end && content_len > have_body && (size_t)n < sizeof(req) - 1) {
            ssize_t m = recv(client_fd, req + n, (sizeof(req) - 1) - (size_t)n, 0);
            if (m <= 0) break;
            n += m;
            req[n] = '\0';
            hdr_end = strstr(req, "\r\n\r\n");
            if (!hdr_end) break;
            have_body = (int)(n - (hdr_end + 4 - req));
        }
        const char* body_ptr = "";
        char* body = NULL;
        if (hdr_end && content_len > 0) {
            const char* bp = hdr_end + 4;
            int avail = (int)(n - (bp - req));
            int take = (avail < content_len) ? avail : content_len;
            body = (char*)malloc((size_t)take + 1);
            memcpy(body, bp, (size_t)take);
            body[take] = '\0';
            body_ptr = body;
        }

        const char* method = blaze_find_method(req);
        char path_key[1024];
        if (!blaze_parse_path(req, path_key, sizeof(path_key))) {
            blaze_send_response(client_fd, 400, "text/plain; charset=utf-8", "Bad Request");
            close(client_fd);
            if (body) free(body);
            continue;
        }

        bool found = false;
        SnaskValue v;
        if (method) {
            char key[1200];
            snprintf(key, sizeof(key), "%s %s", method, path_key);
            v = blaze_route_lookup(routes, key, &found);
        } else {
            v = make_nil();
        }
        if (!found) v = blaze_route_lookup(routes, path_key, &found);
        if (!found) {
            blaze_send_response(client_fd, 404, "text/plain; charset=utf-8", "Not Found");
            close(client_fd);
            if (body) free(body);
            continue;
        }

        // Handler object: { "handler": "fn_name" }
        if ((int)v.tag == SNASK_OBJ) {
            SnaskObject* obj = (SnaskObject*)v.ptr;
            bool has_handler = false;
            SnaskValue hv = blaze_obj_lookup(obj, "handler", &has_handler);
            if (has_handler && (int)hv.tag == SNASK_STR && hv.ptr) {
                char* path_only = blaze_extract_path_only(req);
                char* query = blaze_extract_query(req);
                char* cookie = blaze_extract_cookie_header(req);
                SnaskValue rv = blaze_call_snask_handler((const char*)hv.ptr, method ? method : "GET", path_only, query, body_ptr, cookie);
                v = rv;
            }
        }

        // Suporte a "response objects":
        // { "body": "texto", "status": 200, "content_type": "text/plain" }
        // { "json": <qualquer>, "status": 200 }
        // { "redirect": "https://...", "status": 302 }
        if ((int)v.tag == SNASK_OBJ) {
            SnaskObject* resp = (SnaskObject*)v.ptr;
            bool has_body = false, has_json = false, has_status = false, has_ct = false, has_redirect = false;
            bool has_header = false, has_cookie = false;
            SnaskValue body_v = blaze_obj_lookup(resp, "body", &has_body);
            SnaskValue json_v = blaze_obj_lookup(resp, "json", &has_json);
            SnaskValue status_v = blaze_obj_lookup(resp, "status", &has_status);
            SnaskValue ct_v = blaze_obj_lookup(resp, "content_type", &has_ct);
            SnaskValue redir_v = blaze_obj_lookup(resp, "redirect", &has_redirect);
            SnaskValue header_v = blaze_obj_lookup(resp, "header", &has_header);
            SnaskValue cookie_v = blaze_obj_lookup(resp, "cookie", &has_cookie);

            int status = (has_status && (int)status_v.tag == SNASK_NUM) ? (int)status_v.num : 200;
            const char* ct = (has_ct && (int)ct_v.tag == SNASK_STR) ? (const char*)ct_v.ptr : NULL;
            const char* header_line = (has_header && (int)header_v.tag == SNASK_STR) ? (const char*)header_v.ptr : NULL;
            const char* cookie_line = (has_cookie && (int)cookie_v.tag == SNASK_STR) ? (const char*)cookie_v.ptr : NULL;

            if (has_redirect && (int)redir_v.tag == SNASK_STR) {
                char extra[512];
                snprintf(extra, sizeof(extra), "Location: %s\r\n", (const char*)redir_v.ptr);
                // concatena Location + header/cookie (se existirem)
                char extra2[1024];
                extra2[0] = '\0';
                strncat(extra2, extra, sizeof(extra2) - 1);
                if (header_line && header_line[0]) { strncat(extra2, header_line, sizeof(extra2) - strlen(extra2) - 1); strncat(extra2, "\r\n", sizeof(extra2) - strlen(extra2) - 1); }
                if (cookie_line && cookie_line[0]) { strncat(extra2, "Set-Cookie: ", sizeof(extra2) - strlen(extra2) - 1); strncat(extra2, cookie_line, sizeof(extra2) - strlen(extra2) - 1); strncat(extra2, "\r\n", sizeof(extra2) - strlen(extra2) - 1); }
                blaze_send_response_extra(client_fd, (status == 0 ? 302 : status), ct ? ct : "text/plain; charset=utf-8", extra2, "");
            } else if (has_body && (int)body_v.tag == SNASK_STR) {
                blaze_send_response_headers(client_fd, status, ct ? ct : "text/plain; charset=utf-8", header_line, cookie_line, (const char*)body_v.ptr);
            } else if (has_json) {
                SnaskValue json;
                json_stringify(&json, &json_v);
                blaze_send_response_headers(client_fd, status, ct ? ct : "application/json; charset=utf-8", header_line, cookie_line, (const char*)json.ptr);
                free(json.ptr);
            } else {
                // fallback: stringify do próprio objeto
                SnaskValue json;
                json_stringify(&json, &v);
                blaze_send_response_headers(client_fd, status, ct ? ct : "application/json; charset=utf-8", header_line, cookie_line, (const char*)json.ptr);
                free(json.ptr);
            }
        } else if ((int)v.tag == SNASK_STR) {
            blaze_send_response(client_fd, 200, "text/plain; charset=utf-8", (const char*)v.ptr);
        } else {
            SnaskValue json;
            json_stringify(&json, &v);
            blaze_send_response(client_fd, 200, "application/json; charset=utf-8", (const char*)json.ptr);
            free(json.ptr);
        }
        close(client_fd);
        if (body) free(body);
    }
}

// Querystring: "a=1&b=2" -> value (string) ou nil
void blaze_qs_get(SnaskValue* out, SnaskValue* qs, SnaskValue* key) {
    if ((int)qs->tag != SNASK_STR || (int)key->tag != SNASK_STR || !qs->ptr || !key->ptr) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    const char* s = (const char*)qs->ptr;
    const char* k = (const char*)key->ptr;
    size_t klen = strlen(k);
    const char* p = s;
    while (*p) {
        const char* amp = strchr(p, '&');
        const char* end = amp ? amp : (p + strlen(p));
        const char* eq = memchr(p, '=', (size_t)(end - p));
        if (eq) {
            size_t nlen = (size_t)(eq - p);
            if (nlen == klen && strncmp(p, k, klen) == 0) {
                const char* v = eq + 1;
                size_t vlen = (size_t)(end - v);
                out->tag = (double)SNASK_STR;
                out->ptr = snask_gc_strndup(v, vlen);
                out->num = 0;
                return;
            }
        } else {
            size_t nlen = (size_t)(end - p);
            if (nlen == klen && strncmp(p, k, klen) == 0) {
                out->tag = (double)SNASK_STR;
                out->ptr = snask_gc_strdup("");
                out->num = 0;
                return;
            }
        }
        if (!amp) break;
        p = amp + 1;
    }
    out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0;
}

// Cookie header: "a=1; b=2" -> value (string) ou nil
void blaze_cookie_get(SnaskValue* out, SnaskValue* cookie_header, SnaskValue* name) {
    if ((int)cookie_header->tag != SNASK_STR || (int)name->tag != SNASK_STR || !cookie_header->ptr || !name->ptr) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    const char* s = (const char*)cookie_header->ptr;
    const char* k = (const char*)name->ptr;
    size_t klen = strlen(k);
    const char* p = s;
    while (*p) {
        while (*p == ' ' || *p == '\t' || *p == ';') p++;
        const char* end = strchr(p, ';');
        if (!end) end = p + strlen(p);
        const char* eq = memchr(p, '=', (size_t)(end - p));
        if (eq) {
            size_t nlen = (size_t)(eq - p);
            if (nlen == klen && strncmp(p, k, klen) == 0) {
                const char* v = eq + 1;
                size_t vlen = (size_t)(end - v);
                out->tag = (double)SNASK_STR;
                out->ptr = snask_gc_strndup(v, vlen);
                out->num = 0;
                return;
            }
        }
        p = end;
        if (*p == ';') p++;
    }
    out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0;
}

// --- Sjson (Snask JSON standard) ---
static SnaskValue sjson_new_empty_object_value(void) {
    SnaskObject* obj = (SnaskObject*)malloc(sizeof(SnaskObject));
    obj->count = 0;
    obj->names = NULL;
    obj->values = NULL;
    return (SnaskValue){(double)SNASK_OBJ, 0, obj};
}

void sjson_new_object(SnaskValue* out) { *out = sjson_new_empty_object_value(); }
void sjson_new_array(SnaskValue* out) { *out = sjson_new_empty_object_value(); }

void sjson_type(SnaskValue* out, SnaskValue* v) {
    out->tag = (double)SNASK_STR;
    out->num = 0;
    out->ptr = NULL;
    int tag = (int)v->tag;
    const char* t = "null";
    if (tag == SNASK_NUM) t = "num";
    else if (tag == SNASK_BOOL) t = "bool";
    else if (tag == SNASK_STR) t = "str";
    else if (tag == SNASK_OBJ) t = "obj";
    out->ptr = snask_gc_strdup(t);
}

void sjson_arr_len(SnaskValue* out, SnaskValue* arr) {
    out->tag = (double)SNASK_NUM;
    out->ptr = NULL;
    out->num = 0;
    if ((int)arr->tag != SNASK_OBJ || !arr->ptr) return;
    SnaskObject* o = (SnaskObject*)arr->ptr;
    out->num = (double)o->count;
}

void sjson_arr_get(SnaskValue* out, SnaskValue* arr, SnaskValue* idx_val) {
    if ((int)arr->tag != SNASK_OBJ || (int)idx_val->tag != SNASK_NUM || !arr->ptr) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    SnaskObject* o = (SnaskObject*)arr->ptr;
    int idx = (int)idx_val->num;
    if (idx < 0 || idx >= o->count) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    *out = o->values[idx];
}

void sjson_arr_set(SnaskValue* out, SnaskValue* arr, SnaskValue* idx_val, SnaskValue* value) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if ((int)arr->tag != SNASK_OBJ || (int)idx_val->tag != SNASK_NUM || !arr->ptr) return;
    SnaskObject* o = (SnaskObject*)arr->ptr;
    int idx = (int)idx_val->num;
    if (idx < 0) return;

    if (idx < o->count) {
        o->values[idx] = *value;
        out->num = 1.0;
        return;
    }

    // expand up to idx
    int new_count = idx + 1;
    o->names = (char**)realloc(o->names, (size_t)new_count * sizeof(char*));
    o->values = (SnaskValue*)realloc(o->values, (size_t)new_count * sizeof(SnaskValue));
    for (int i = o->count; i < new_count; i++) {
        char idx_name[32];
        snprintf(idx_name, sizeof(idx_name), "%d", i);
        o->names[i] = snask_gc_strdup(idx_name);
        o->values[i] = make_nil();
    }
    o->count = new_count;
    o->values[idx] = *value;
    out->num = 1.0;
}

void sjson_arr_push(SnaskValue* out, SnaskValue* arr, SnaskValue* value) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if ((int)arr->tag != SNASK_OBJ || !arr->ptr) return;
    SnaskObject* o = (SnaskObject*)arr->ptr;
    int idx = o->count;
    int new_count = o->count + 1;
    o->names = (char**)realloc(o->names, (size_t)new_count * sizeof(char*));
    o->values = (SnaskValue*)realloc(o->values, (size_t)new_count * sizeof(SnaskValue));
    char idx_name[32];
    snprintf(idx_name, sizeof(idx_name), "%d", idx);
    o->names[idx] = snask_gc_strdup(idx_name);
    o->values[idx] = *value;
    o->count = new_count;
    out->num = 1.0;
}

static bool sjson_is_digits(const char* s) {
    if (!s || !*s) return false;
    for (const unsigned char* p = (const unsigned char*)s; *p; p++) {
        if (*p < '0' || *p > '9') return false;
    }
    return true;
}

// Path get: "a.b.0.c" (obj keys + numeric index)
// Retorna { ok: bool, value: any, error: str }
void sjson_path_get(SnaskValue* out, SnaskValue* root, SnaskValue* path_val) {
    if ((int)path_val->tag != SNASK_STR || !path_val->ptr) { out->tag = (double)SNASK_NIL; return; }
    const char* path = (const char*)path_val->ptr;

    SnaskValue cur = *root;
    const char* p = path;
    char seg[256];

    while (*p) {
        size_t si = 0;
        while (*p && *p != '.') {
            if (si + 1 < sizeof(seg)) seg[si++] = *p;
            p++;
        }
        seg[si] = '\0';
        if (*p == '.') p++;

        if ((int)cur.tag != SNASK_OBJ || !cur.ptr) {
            SnaskObject* r = obj_new(3);
            r->names[0] = snask_gc_strdup("ok"); r->values[0] = make_bool(false);
            r->names[1] = snask_gc_strdup("value"); r->values[1] = make_nil();
            r->names[2] = snask_gc_strdup("error"); r->values[2] = make_str(snask_gc_strdup("path_get: alvo não é objeto/array."));
            *out = make_obj(r);
            return;
        }

        if (seg[0] == '\0') {
            SnaskObject* r = obj_new(3);
            r->names[0] = snask_gc_strdup("ok"); r->values[0] = make_bool(false);
            r->names[1] = snask_gc_strdup("value"); r->values[1] = make_nil();
            r->names[2] = snask_gc_strdup("error"); r->values[2] = make_str(snask_gc_strdup("path_get: segmento vazio."));
            *out = make_obj(r);
            return;
        }

        SnaskObject* o = (SnaskObject*)cur.ptr;
        bool found = false;
        SnaskValue next = make_nil();

        if (sjson_is_digits(seg)) {
            int idx = atoi(seg);
            if (idx >= 0 && idx < o->count) { next = o->values[idx]; found = true; }
        } else {
            for (int i = 0; i < o->count; i++) {
                if (o->names[i] && strcmp(o->names[i], seg) == 0) { next = o->values[i]; found = true; break; }
            }
        }

        if (!found) {
            SnaskObject* r = obj_new(3);
            r->names[0] = snask_gc_strdup("ok"); r->values[0] = make_bool(false);
            r->names[1] = snask_gc_strdup("value"); r->values[1] = make_nil();
            char msg[512];
            snprintf(msg, sizeof(msg), "path_get: segmento '%s' não encontrado.", seg);
            r->names[2] = snask_gc_strdup("error"); r->values[2] = make_str(snask_gc_strdup(msg));
            *out = make_obj(r);
            return;
        }
        cur = next;
    }

    SnaskObject* r = obj_new(3);
    r->names[0] = snask_gc_strdup("ok"); r->values[0] = make_bool(true);
    r->names[1] = snask_gc_strdup("value"); r->values[1] = cur;
    r->names[2] = snask_gc_strdup("error"); r->values[2] = make_str(snask_gc_strdup(""));
    *out = make_obj(r);
}

// --- Random ---
void os_random_hex(SnaskValue* out, SnaskValue* nbytes_val) {
    out->tag = (double)SNASK_STR;
    out->ptr = NULL;
    out->num = 0;
    if ((int)nbytes_val->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; return; }
    int nbytes = (int)nbytes_val->num;
    if (nbytes <= 0 || nbytes > 4096) { out->tag = (double)SNASK_NIL; return; }

    unsigned char* buf = (unsigned char*)malloc((size_t)nbytes);
    int fd = open("/dev/urandom", O_RDONLY);
    if (fd < 0) { free(buf); out->tag = (double)SNASK_NIL; return; }
    ssize_t n = read(fd, buf, (size_t)nbytes);
    close(fd);
    if (n != nbytes) { free(buf); out->tag = (double)SNASK_NIL; return; }

    static const char* hex = "0123456789abcdef";
    char* s = (char*)snask_gc_malloc((size_t)nbytes * 2 + 1);
    for (int i = 0; i < nbytes; i++) {
        s[i * 2] = hex[(buf[i] >> 4) & 0xF];
        s[i * 2 + 1] = hex[buf[i] & 0xF];
    }
    s[(size_t)nbytes * 2] = '\0';
    free(buf);
    out->ptr = s;
}

// --- Auth natives (blaze_auth) ---
static uint64_t fnv1a64(const unsigned char* data, size_t len) {
    uint64_t h = 1469598103934665603ULL;
    for (size_t i = 0; i < len; i++) {
        h ^= (uint64_t)data[i];
        h *= 1099511628211ULL;
    }
    return h;
}

static void u64_to_hex(uint64_t v, char out16[17]) {
    static const char* hex = "0123456789abcdef";
    for (int i = 15; i >= 0; i--) {
        out16[i] = hex[v & 0xF];
        v >>= 4;
    }
    out16[16] = '\0';
}

void auth_random_hex(SnaskValue* out, SnaskValue* nbytes_val) { os_random_hex(out, nbytes_val); }

void auth_now(SnaskValue* out) { out->tag = (double)SNASK_NUM; out->num = (double)time(NULL); out->ptr = NULL; }

void auth_const_time_eq(SnaskValue* out, SnaskValue* a, SnaskValue* b) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if ((int)a->tag != SNASK_STR || (int)b->tag != SNASK_STR || !a->ptr || !b->ptr) return;
    const unsigned char* sa = (const unsigned char*)a->ptr;
    const unsigned char* sb = (const unsigned char*)b->ptr;
    size_t la = strlen((const char*)sa);
    size_t lb = strlen((const char*)sb);
    size_t n = (la > lb) ? la : lb;
    unsigned char diff = (unsigned char)(la ^ lb);
    for (size_t i = 0; i < n; i++) {
        unsigned char ca = (i < la) ? sa[i] : 0;
        unsigned char cb = (i < lb) ? sb[i] : 0;
        diff |= (unsigned char)(ca ^ cb);
    }
    out->num = (diff == 0) ? 1.0 : 0.0;
}

// Formato do hash: "v1$<salt_hex>$<hash_hex16>"
void auth_hash_password(SnaskValue* out, SnaskValue* password) {
    if ((int)password->tag != SNASK_STR || !password->ptr) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    SnaskValue salt;
    SnaskValue nbytes = (SnaskValue){(double)SNASK_NUM, 16.0, NULL};
    os_random_hex(&salt, &nbytes);
    if ((int)salt.tag != SNASK_STR || !salt.ptr) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }

    const char* pass = (const char*)password->ptr;
    const char* salt_s = (const char*)salt.ptr;
    size_t ls = strlen(salt_s);
    size_t lp = strlen(pass);
    size_t l = ls + 1 + lp;
    unsigned char* buf = (unsigned char*)malloc(l);
    memcpy(buf, salt_s, ls);
    buf[ls] = ':';
    memcpy(buf + ls + 1, pass, lp);
    uint64_t h = fnv1a64(buf, l);
    free(buf);

    char hex16[17];
    u64_to_hex(h, hex16);

    size_t out_len = 3 + 1 + ls + 1 + 16; // "v1$" + salt + "$" + hash
    char* s = (char*)snask_gc_malloc(out_len + 1);
    snprintf(s, out_len + 1, "v1$%s$%s", salt_s, hex16);

    out->tag = (double)SNASK_STR;
    out->ptr = s;
    out->num = 0;
}

static bool parse_v1_hash(const char* stored, const char** salt_out, size_t* salt_len, const char** hash_out) {
    if (!stored) return false;
    if (strncmp(stored, "v1$", 3) != 0) return false;
    const char* p = stored + 3;
    const char* d = strchr(p, '$');
    if (!d) return false;
    *salt_out = p;
    *salt_len = (size_t)(d - p);
    *hash_out = d + 1;
    if (strlen(*hash_out) != 16) return false;
    return true;
}

void auth_verify_password(SnaskValue* out, SnaskValue* password, SnaskValue* stored_hash) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if ((int)password->tag != SNASK_STR || (int)stored_hash->tag != SNASK_STR || !password->ptr || !stored_hash->ptr) return;
    const char* stored = (const char*)stored_hash->ptr;
    const char* salt = NULL;
    size_t salt_len = 0;
    const char* hash_hex = NULL;
    if (!parse_v1_hash(stored, &salt, &salt_len, &hash_hex)) return;

    const char* pass = (const char*)password->ptr;
    size_t lp = strlen(pass);
    size_t l = salt_len + 1 + lp;
    unsigned char* buf = (unsigned char*)malloc(l);
    memcpy(buf, salt, salt_len);
    buf[salt_len] = ':';
    memcpy(buf + salt_len + 1, pass, lp);
    uint64_t h = fnv1a64(buf, l);
    free(buf);

    char hex16[17];
    u64_to_hex(h, hex16);

    // constant-time compare
    unsigned char diff = 0;
    for (int i = 0; i < 16; i++) diff |= (unsigned char)(hex16[i] ^ hash_hex[i]);
    out->num = (diff == 0) ? 1.0 : 0.0;
}

void auth_session_id(SnaskValue* out) {
    SnaskValue nbytes = (SnaskValue){(double)SNASK_NUM, 16.0, NULL};
    os_random_hex(out, &nbytes);
}

void auth_csrf_token(SnaskValue* out) {
    SnaskValue nbytes = (SnaskValue){(double)SNASK_NUM, 32.0, NULL};
    os_random_hex(out, &nbytes);
}

void auth_cookie_kv(SnaskValue* out, SnaskValue* name, SnaskValue* value) {
    if ((int)name->tag != SNASK_STR || (int)value->tag != SNASK_STR || !name->ptr || !value->ptr) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    const char* n = (const char*)name->ptr;
    const char* v = (const char*)value->ptr;
    size_t ln = strlen(n), lv = strlen(v);
    char* s = (char*)malloc(ln + 1 + lv + 1);
    memcpy(s, n, ln);
    s[ln] = '=';
    memcpy(s + ln + 1, v, lv);
    s[ln + 1 + lv] = '\0';
    out->tag = (double)SNASK_STR;
    out->ptr = s;
    out->num = 0;
}

void auth_cookie_session(SnaskValue* out, SnaskValue* sid) {
    if ((int)sid->tag != SNASK_STR || !sid->ptr) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    const char* v = (const char*)sid->ptr;
    size_t lv = strlen(v);
    const char* suffix = "; Path=/; HttpOnly";
    char* s = (char*)malloc(4 + lv + strlen(suffix) + 1);
    strcpy(s, "sid=");
    strcat(s, v);
    strcat(s, suffix);
    out->tag = (double)SNASK_STR;
    out->ptr = s;
    out->num = 0;
}

void auth_cookie_delete(SnaskValue* out, SnaskValue* name) {
    if ((int)name->tag != SNASK_STR || !name->ptr) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    const char* n = (const char*)name->ptr;
    const char* suffix = "=; Path=/; Max-Age=0";
    char* s = (char*)malloc(strlen(n) + strlen(suffix) + 1);
    strcpy(s, n);
    strcat(s, suffix);
    out->tag = (double)SNASK_STR;
    out->ptr = s;
    out->num = 0;
}

void auth_bearer_header(SnaskValue* out, SnaskValue* token) {
    if ((int)token->tag != SNASK_STR || !token->ptr) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    const char* t = (const char*)token->ptr;
    const char* prefix = "Authorization: Bearer ";
    char* s = (char*)malloc(strlen(prefix) + strlen(t) + 1);
    strcpy(s, prefix);
    strcat(s, t);
    out->tag = (double)SNASK_STR;
    out->ptr = s;
    out->num = 0;
}

void auth_ok(SnaskValue* out) { out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL; }
void auth_fail(SnaskValue* out) { out->tag = (double)SNASK_BOOL; out->num = 0.0; out->ptr = NULL; }
void auth_version(SnaskValue* out) { out->tag = (double)SNASK_STR; out->ptr = snask_gc_strdup("0.2.0"); out->num = 0; }

// --- Type checks ---
void is_nil(SnaskValue* out, SnaskValue* v) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = ((int)v->tag == SNASK_NIL) ? 1.0 : 0.0;
}

void is_str(SnaskValue* out, SnaskValue* v) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = ((int)v->tag == SNASK_STR) ? 1.0 : 0.0;
}

void is_obj(SnaskValue* out, SnaskValue* v) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = ((int)v->tag == SNASK_OBJ) ? 1.0 : 0.0;
}

void sfs_delete(SnaskValue* out, SnaskValue* path) { 
    if ((int)path->tag != SNASK_STR) { out->tag = (double)SNASK_BOOL; out->num = 0; return; }
    int res = remove((char*)path->ptr); 
    out->tag = (double)SNASK_BOOL; out->num = (res == 0); out->ptr = NULL;
}

void sfs_exists(SnaskValue* out, SnaskValue* path) { 
    if ((int)path->tag != SNASK_STR) { out->tag = (double)SNASK_BOOL; out->num = 0; return; }
    int res = access((char*)path->ptr, F_OK); 
    out->tag = (double)SNASK_BOOL; out->num = (res == 0); out->ptr = NULL;
}

void s_abs(SnaskValue* out, SnaskValue* n) { *out = (SnaskValue){(double)SNASK_NUM, fabs(n->num), NULL}; }
void s_max(SnaskValue* out, SnaskValue* a, SnaskValue* b) { *out = (SnaskValue){(double)SNASK_NUM, fmax(a->num, b->num), NULL}; }
void s_min(SnaskValue* out, SnaskValue* a, SnaskValue* b) { *out = (SnaskValue){(double)SNASK_NUM, fmin(a->num, b->num), NULL}; }

void s_len(SnaskValue* out, SnaskValue* s) { 
    if ((int)s->tag != SNASK_STR) { out->tag = (double)SNASK_NUM; out->num = 0; return; }
    out->tag = (double)SNASK_NUM; out->num = (double)strlen((char*)s->ptr); 
}

void s_upper(SnaskValue* out, SnaskValue* s) {
    if ((int)s->tag != SNASK_STR) { *out = *s; return; }
    char* new_s = snask_gc_strdup((char*)s->ptr);
    for(int i = 0; new_s[i]; i++) new_s[i] = toupper(new_s[i]);
    out->tag = (double)SNASK_STR; out->ptr = new_s; out->num = 0;
}

void s_time(SnaskValue* out) { out->tag = (double)SNASK_NUM; out->num = (double)time(NULL); out->ptr = NULL; }
void s_sleep(SnaskValue* out, SnaskValue* ms) { usleep((unsigned int)(ms->num * 1000)); out->tag = (double)SNASK_NIL; }
void s_exit(SnaskValue* out, SnaskValue* code) {
    int status = 0;
    if (code && (int)code->tag == SNASK_NUM) status = (int)code->num;
    out->tag = (double)SNASK_NIL;
    exit(status);
}

// ---------------- Multithreading (pthread) ----------------
typedef struct {
    pthread_t tid;
    char* fn_name;
    char* arg;
    int started;
    int joined;
} SnaskThread;

static char* thread_ptr_to_handle(void* p) {
    char buf[64];
    snprintf(buf, sizeof(buf), "%p", p);
    return snask_gc_strdup(buf);
}

static void* thread_handle_to_ptr(const char* h) {
    if (!h) return NULL;
    void* p = NULL;
    sscanf(h, "%p", &p);
    return p;
}

static void* snask_thread_entry(void* vp) {
    SnaskThread* t = (SnaskThread*)vp;
    if (!t || !t->fn_name) return NULL;
    char sym[512];
    snprintf(sym, sizeof(sym), "f_%s", t->fn_name);
    void* fp = dlsym(NULL, sym);
    if (!fp) return NULL;
    typedef void (*SnaskFn1)(SnaskValue* ra, SnaskValue* a1);
    SnaskFn1 f = (SnaskFn1)fp;
    SnaskValue ra = make_nil();
    SnaskValue a = make_str_dup(t->arg ? t->arg : "");
    f(&ra, &a);
    (void)ra;
    return NULL;
}

// thread_spawn(fn_name, arg_str) -> handle(str) ou nil
void thread_spawn(SnaskValue* out, SnaskValue* fn_name, SnaskValue* arg_str) {
    if ((int)fn_name->tag != SNASK_STR || (int)arg_str->tag != SNASK_STR || !fn_name->ptr || !arg_str->ptr) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    SnaskThread* t = (SnaskThread*)snask_gc_malloc(sizeof(SnaskThread));
    memset(t, 0, sizeof(*t));
    t->fn_name = snask_gc_strdup((const char*)fn_name->ptr);
    t->arg = snask_gc_strdup((const char*)arg_str->ptr);
    if (pthread_create(&t->tid, NULL, snask_thread_entry, t) != 0) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    t->started = 1;
    out->tag = (double)SNASK_STR;
    out->ptr = thread_ptr_to_handle(t);
    out->num = 0;
}

// thread_join(handle) -> bool
void thread_join(SnaskValue* out, SnaskValue* handle) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if ((int)handle->tag != SNASK_STR || !handle->ptr) return;
    SnaskThread* t = (SnaskThread*)thread_handle_to_ptr((const char*)handle->ptr);
    if (!t || !t->started || t->joined) return;
    if (pthread_join(t->tid, NULL) != 0) return;
    t->joined = 1;
    out->num = 1.0;
}

// thread_detach(handle) -> bool
void thread_detach(SnaskValue* out, SnaskValue* handle) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if ((int)handle->tag != SNASK_STR || !handle->ptr) return;
    SnaskThread* t = (SnaskThread*)thread_handle_to_ptr((const char*)handle->ptr);
    if (!t || !t->started) return;
    if (pthread_detach(t->tid) != 0) return;
    out->num = 1.0;
}

void s_concat(SnaskValue* out, SnaskValue* s1, SnaskValue* s2) {
    if ((int)s1->tag != SNASK_STR || (int)s2->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    size_t len1 = strlen((char*)s1->ptr); size_t len2 = strlen((char*)s2->ptr);
    char* new_str = (char*)snask_gc_malloc(len1 + len2 + 1);
    strcpy(new_str, (char*)s1->ptr); strcat(new_str, (char*)s2->ptr);
    out->tag = (double)SNASK_STR; out->ptr = new_str; out->num = 0;
}

// substring(str, start, len) -> string
void substring(SnaskValue* out, SnaskValue* s, SnaskValue* start_v, SnaskValue* len_v) {
    if (!s || (int)s->tag != SNASK_STR || !s->ptr || !start_v || (int)start_v->tag != SNASK_NUM || !len_v || (int)len_v->tag != SNASK_NUM) {
        out->tag = (double)SNASK_NIL;
        out->ptr = NULL;
        out->num = 0;
        return;
    }
    const char* src = (const char*)s->ptr;
    int start = (int)start_v->num;
    int len = (int)len_v->num;
    int slen = (int)strlen(src);
    if (start < 0) start = 0;
    if (len < 0) len = 0;
    if (start > slen) start = slen;
    if (start + len > slen) len = slen - start;
    char* dst = (char*)snask_gc_malloc((size_t)len + 1);
    memcpy(dst, src + start, (size_t)len);
    dst[len] = '\0';
    out->tag = (double)SNASK_STR;
    out->ptr = dst;
    out->num = 0;
}

// ---------------- GUI (GTK3) ----------------
#ifdef SNASK_GUI_GTK

static char* gui_ptr_to_handle(void* p) {
    char buf[64];
    snprintf(buf, sizeof(buf), "%p", p);
    return snask_gc_strdup(buf);
}

static void* gui_handle_to_ptr(const char* h) {
    if (!h) return NULL;
    void* p = NULL;
    // accepts "0x..." produced by %p
    sscanf(h, "%p", &p);
    return p;
}

typedef struct {
    char* handler_name;
    char* widget_handle;
    char* ctx;
} GuiCallbackCtx;

static void gui_free_ctx(GuiCallbackCtx* ctx) {
    if (!ctx) return;
    if (ctx->handler_name) free(ctx->handler_name);
    if (ctx->widget_handle) free(ctx->widget_handle);
    if (ctx->ctx) free(ctx->ctx);
    free(ctx);
}

static SnaskValue gui_call_handler_1(const char* handler_name, const char* widget_handle) {
    if (!handler_name) return make_nil();
    char sym[512];
    snprintf(sym, sizeof(sym), "f_%s", handler_name);
    void* fp = dlsym(NULL, sym);
    if (!fp) return make_nil();

    typedef void (*SnaskFn1)(SnaskValue* ra, SnaskValue* a1);
    SnaskFn1 f = (SnaskFn1)fp;

    SnaskValue ra = make_nil();
    SnaskValue wh = make_str_dup(widget_handle ? widget_handle : "");
    f(&ra, &wh);
    return ra;
}

static SnaskValue gui_call_handler_2(const char* handler_name, const char* widget_handle, const char* ctx) {
    if (!handler_name) return make_nil();
    char sym[512];
    snprintf(sym, sizeof(sym), "f_%s", handler_name);
    void* fp = dlsym(NULL, sym);
    if (!fp) return make_nil();

    typedef void (*SnaskFn2)(SnaskValue* ra, SnaskValue* a1, SnaskValue* a2);
    SnaskFn2 f = (SnaskFn2)fp;

    SnaskValue ra = make_nil();
    SnaskValue wh = make_str_dup(widget_handle ? widget_handle : "");
    SnaskValue cv = make_str_dup(ctx ? ctx : "");
    f(&ra, &wh, &cv);
    return ra;
}

static void gui_on_button_clicked(GtkWidget* _widget, gpointer user_data) {
    (void)_widget;
    GuiCallbackCtx* ctx = (GuiCallbackCtx*)user_data;
    if (!ctx) return;
    if (ctx->ctx) (void)gui_call_handler_2(ctx->handler_name, ctx->widget_handle, ctx->ctx);
    else (void)gui_call_handler_1(ctx->handler_name, ctx->widget_handle);
}

void gui_init(SnaskValue* out) {
    int argc = 0;
    char** argv = NULL;
    // gtk_init() termina o processo se não houver display. Preferimos não abortar:
    // retorne false e deixe o app lidar com isso.
    gboolean ok = gtk_init_check(&argc, &argv);
    out->tag = (double)SNASK_BOOL;
    out->num = ok ? 1.0 : 0.0;
    out->ptr = NULL;
}

void gui_quit(SnaskValue* out) {
    gtk_main_quit();
    out->tag = (double)SNASK_NIL;
}

void gui_run(SnaskValue* out) {
    gtk_main();
    out->tag = (double)SNASK_NIL;
}

void gui_window(SnaskValue* out, SnaskValue* title, SnaskValue* w, SnaskValue* h) {
    if ((int)title->tag != SNASK_STR || (int)w->tag != SNASK_NUM || (int)h->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* win = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    gtk_window_set_title(GTK_WINDOW(win), (const char*)title->ptr);
    gtk_window_set_default_size(GTK_WINDOW(win), (int)w->num, (int)h->num);
    g_signal_connect(win, "destroy", G_CALLBACK(gtk_main_quit), NULL);
    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(win);
    out->num = 0;
}

void gui_set_title(SnaskValue* out, SnaskValue* win_h, SnaskValue* title) {
    if ((int)win_h->tag != SNASK_STR || (int)title->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* win = (GtkWidget*)gui_handle_to_ptr((const char*)win_h->ptr);
    if (!win || !GTK_IS_WINDOW(win)) { out->tag = (double)SNASK_NIL; return; }
    gtk_window_set_title(GTK_WINDOW(win), (const char*)title->ptr);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void gui_set_resizable(SnaskValue* out, SnaskValue* win_h, SnaskValue* resizable) {
    if ((int)win_h->tag != SNASK_STR || (int)resizable->tag != SNASK_BOOL) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* win = (GtkWidget*)gui_handle_to_ptr((const char*)win_h->ptr);
    if (!win || !GTK_IS_WINDOW(win)) { out->tag = (double)SNASK_NIL; return; }
    gtk_window_set_resizable(GTK_WINDOW(win), resizable->num != 0.0);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void gui_autosize(SnaskValue* out, SnaskValue* win_h) {
    if ((int)win_h->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* win = (GtkWidget*)gui_handle_to_ptr((const char*)win_h->ptr);
    if (!win || !GTK_IS_WINDOW(win)) { out->tag = (double)SNASK_NIL; return; }
    gtk_window_resize(GTK_WINDOW(win), 1, 1);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void gui_vbox(SnaskValue* out) {
    GtkWidget* box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 8);
    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(box);
    out->num = 0;
}

void gui_hbox(SnaskValue* out) {
    GtkWidget* box = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(box);
    out->num = 0;
}

void gui_scrolled(SnaskValue* out) {
    GtkWidget* sw = gtk_scrolled_window_new(NULL, NULL);
    gtk_scrolled_window_set_policy(GTK_SCROLLED_WINDOW(sw), GTK_POLICY_AUTOMATIC, GTK_POLICY_AUTOMATIC);
    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(sw);
    out->num = 0;
}

void gui_listbox(SnaskValue* out) {
    GtkWidget* lb = gtk_list_box_new();
    gtk_list_box_set_selection_mode(GTK_LIST_BOX(lb), GTK_SELECTION_SINGLE);
    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(lb);
    out->num = 0;
}

void gui_list_add_text(SnaskValue* out, SnaskValue* list_h, SnaskValue* text) {
    if ((int)list_h->tag != SNASK_STR || (int)text->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)list_h->ptr);
    if (!w || !GTK_IS_LIST_BOX(w)) { out->tag = (double)SNASK_NIL; return; }

    GtkWidget* lbl = gtk_label_new((const char*)text->ptr);
    gtk_widget_set_halign(lbl, GTK_ALIGN_START);

    GtkWidget* row = gtk_list_box_row_new();
    gtk_container_add(GTK_CONTAINER(row), lbl);
    gtk_widget_show_all(row);
    gtk_list_box_insert(GTK_LIST_BOX(w), row, -1);
    g_object_set_data_full(G_OBJECT(row), "snask_pkg", strdup((const char*)text->ptr), free);

    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(row);
    out->num = 0;
}

static void gui_on_list_selected(GtkListBox* _box, GtkListBoxRow* row, gpointer user_data) {
    (void)_box;
    GuiCallbackCtx* ctx = (GuiCallbackCtx*)user_data;
    if (!ctx) return;
    if (!row) return;
    const char* pkg = (const char*)g_object_get_data(G_OBJECT(row), "snask_pkg");
    if (!pkg) pkg = "";
    if (ctx->ctx) (void)gui_call_handler_2(ctx->handler_name, pkg, ctx->ctx);
    else (void)gui_call_handler_1(ctx->handler_name, pkg);
}

void gui_on_select_ctx(SnaskValue* out, SnaskValue* list_h, SnaskValue* handler_name, SnaskValue* ctx_str) {
    if ((int)list_h->tag != SNASK_STR || (int)handler_name->tag != SNASK_STR || (int)ctx_str->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)list_h->ptr);
    if (!w || !GTK_IS_LIST_BOX(w)) { out->tag = (double)SNASK_NIL; return; }
    GuiCallbackCtx* ctx = (GuiCallbackCtx*)calloc(1, sizeof(GuiCallbackCtx));
    ctx->handler_name = strdup((const char*)handler_name->ptr);
    ctx->widget_handle = strdup((const char*)list_h->ptr);
    ctx->ctx = strdup((const char*)ctx_str->ptr);
    g_signal_connect_data(w, "row-selected", G_CALLBACK(gui_on_list_selected), ctx, (GClosureNotify)gui_free_ctx, 0);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void gui_set_child(SnaskValue* out, SnaskValue* parent_h, SnaskValue* child_h) {
    if ((int)parent_h->tag != SNASK_STR || (int)child_h->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* parent = (GtkWidget*)gui_handle_to_ptr((const char*)parent_h->ptr);
    GtkWidget* child = (GtkWidget*)gui_handle_to_ptr((const char*)child_h->ptr);
    if (!parent || !child) { out->tag = (double)SNASK_NIL; return; }
    gtk_container_add(GTK_CONTAINER(parent), child);
    out->tag = (double)SNASK_BOOL;
    out->num = 1.0;
    out->ptr = NULL;
}

void gui_add(SnaskValue* out, SnaskValue* box_h, SnaskValue* child_h) {
    if ((int)box_h->tag != SNASK_STR || (int)child_h->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* box = (GtkWidget*)gui_handle_to_ptr((const char*)box_h->ptr);
    GtkWidget* child = (GtkWidget*)gui_handle_to_ptr((const char*)child_h->ptr);
    if (!box || !child) { out->tag = (double)SNASK_NIL; return; }
    gtk_box_pack_start(GTK_BOX(box), child, FALSE, FALSE, 0);
    out->tag = (double)SNASK_BOOL;
    out->num = 1.0;
    out->ptr = NULL;
}

void gui_add_expand(SnaskValue* out, SnaskValue* box_h, SnaskValue* child_h) {
    if ((int)box_h->tag != SNASK_STR || (int)child_h->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* box = (GtkWidget*)gui_handle_to_ptr((const char*)box_h->ptr);
    GtkWidget* child = (GtkWidget*)gui_handle_to_ptr((const char*)child_h->ptr);
    if (!box || !child) { out->tag = (double)SNASK_NIL; return; }
    gtk_box_pack_start(GTK_BOX(box), child, TRUE, TRUE, 0);
    out->tag = (double)SNASK_BOOL;
    out->num = 1.0;
    out->ptr = NULL;
}

void gui_label(SnaskValue* out, SnaskValue* text) {
    if ((int)text->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* w = gtk_label_new((const char*)text->ptr);
    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(w);
    out->num = 0;
}

void gui_entry(SnaskValue* out) {
    GtkWidget* e = gtk_entry_new();
    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(e);
    out->num = 0;
}

void gui_set_placeholder(SnaskValue* out, SnaskValue* entry_h, SnaskValue* text) {
    if ((int)entry_h->tag != SNASK_STR || (int)text->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)entry_h->ptr);
    if (!w || !GTK_IS_ENTRY(w)) { out->tag = (double)SNASK_NIL; return; }
    gtk_entry_set_placeholder_text(GTK_ENTRY(w), (const char*)text->ptr);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void gui_set_editable(SnaskValue* out, SnaskValue* entry_h, SnaskValue* editable) {
    if ((int)entry_h->tag != SNASK_STR || (int)editable->tag != SNASK_BOOL) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)entry_h->ptr);
    if (!w || !GTK_IS_ENTRY(w)) { out->tag = (double)SNASK_NIL; return; }
    gtk_editable_set_editable(GTK_EDITABLE(w), editable->num != 0.0);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void gui_button(SnaskValue* out, SnaskValue* text) {
    if ((int)text->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* b = gtk_button_new_with_label((const char*)text->ptr);
    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(b);
    out->num = 0;
}

void gui_set_enabled(SnaskValue* out, SnaskValue* widget_h, SnaskValue* enabled) {
    if ((int)widget_h->tag != SNASK_STR || (int)enabled->tag != SNASK_BOOL) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w) { out->tag = (double)SNASK_NIL; return; }
    gtk_widget_set_sensitive(w, enabled->num != 0.0);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void gui_set_visible(SnaskValue* out, SnaskValue* widget_h, SnaskValue* visible) {
    if ((int)widget_h->tag != SNASK_STR || (int)visible->tag != SNASK_BOOL) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w) { out->tag = (double)SNASK_NIL; return; }
    gtk_widget_set_visible(w, visible->num != 0.0);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void gui_show_all(SnaskValue* out, SnaskValue* widget_h) {
    if ((int)widget_h->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w) { out->tag = (double)SNASK_NIL; return; }
    gtk_widget_show_all(w);
    out->tag = (double)SNASK_NIL;
}

void gui_set_text(SnaskValue* out, SnaskValue* widget_h, SnaskValue* text) {
    if ((int)widget_h->tag != SNASK_STR || (int)text->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w) { out->tag = (double)SNASK_NIL; return; }
    if (GTK_IS_LABEL(w)) gtk_label_set_text(GTK_LABEL(w), (const char*)text->ptr);
    else if (GTK_IS_BUTTON(w)) gtk_button_set_label(GTK_BUTTON(w), (const char*)text->ptr);
    else if (GTK_IS_ENTRY(w)) gtk_entry_set_text(GTK_ENTRY(w), (const char*)text->ptr);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void gui_get_text(SnaskValue* out, SnaskValue* widget_h) {
    if ((int)widget_h->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w) { out->tag = (double)SNASK_NIL; return; }
    if (GTK_IS_ENTRY(w)) {
        const char* t = gtk_entry_get_text(GTK_ENTRY(w));
        out->tag = (double)SNASK_STR;
        out->ptr = snask_gc_strdup(t ? t : "");
        out->num = 0;
        return;
    }
    out->tag = (double)SNASK_NIL;
}

void gui_on_click(SnaskValue* out, SnaskValue* widget_h, SnaskValue* handler_name) {
    if ((int)widget_h->tag != SNASK_STR || (int)handler_name->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w || !GTK_IS_BUTTON(w)) { out->tag = (double)SNASK_NIL; return; }
    GuiCallbackCtx* ctx = (GuiCallbackCtx*)calloc(1, sizeof(GuiCallbackCtx));
    ctx->handler_name = strdup((const char*)handler_name->ptr);
    ctx->widget_handle = strdup((const char*)widget_h->ptr);
    ctx->ctx = NULL;
    g_signal_connect_data(w, "clicked", G_CALLBACK(gui_on_button_clicked), ctx, (GClosureNotify)gui_free_ctx, 0);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void gui_on_click_ctx(SnaskValue* out, SnaskValue* widget_h, SnaskValue* handler_name, SnaskValue* ctx_str) {
    if ((int)widget_h->tag != SNASK_STR || (int)handler_name->tag != SNASK_STR || (int)ctx_str->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w || !GTK_IS_BUTTON(w)) { out->tag = (double)SNASK_NIL; return; }
    GuiCallbackCtx* ctx = (GuiCallbackCtx*)calloc(1, sizeof(GuiCallbackCtx));
    ctx->handler_name = strdup((const char*)handler_name->ptr);
    ctx->widget_handle = strdup((const char*)widget_h->ptr);
    ctx->ctx = strdup((const char*)ctx_str->ptr);
    g_signal_connect_data(w, "clicked", G_CALLBACK(gui_on_button_clicked), ctx, (GClosureNotify)gui_free_ctx, 0);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void gui_separator_h(SnaskValue* out) {
    GtkWidget* s = gtk_separator_new(GTK_ORIENTATION_HORIZONTAL);
    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(s);
    out->num = 0;
}

void gui_separator_v(SnaskValue* out) {
    GtkWidget* s = gtk_separator_new(GTK_ORIENTATION_VERTICAL);
    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(s);
    out->num = 0;
}

static void gui_msg_dialog(GtkMessageType t, const char* title, const char* msg) {
    GtkWidget* dialog = gtk_message_dialog_new(NULL, GTK_DIALOG_MODAL, t, GTK_BUTTONS_OK, "%s", msg ? msg : "");
    if (title) gtk_window_set_title(GTK_WINDOW(dialog), title);
    gtk_dialog_run(GTK_DIALOG(dialog));
    gtk_widget_destroy(dialog);
}

void gui_msg_info(SnaskValue* out, SnaskValue* title, SnaskValue* msg) {
    if ((int)title->tag != SNASK_STR || (int)msg->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    gui_msg_dialog(GTK_MESSAGE_INFO, (const char*)title->ptr, (const char*)msg->ptr);
    out->tag = (double)SNASK_NIL;
}

void gui_msg_error(SnaskValue* out, SnaskValue* title, SnaskValue* msg) {
    if ((int)title->tag != SNASK_STR || (int)msg->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    gui_msg_dialog(GTK_MESSAGE_ERROR, (const char*)title->ptr, (const char*)msg->ptr);
    out->tag = (double)SNASK_NIL;
}

#else

void gui_init(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_quit(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_run(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_window(SnaskValue* out, SnaskValue* _t, SnaskValue* _w, SnaskValue* _h) { (void)_t; (void)_w; (void)_h; out->tag = (double)SNASK_NIL; }
void gui_set_title(SnaskValue* out, SnaskValue* _w, SnaskValue* _t) { (void)_w; (void)_t; out->tag = (double)SNASK_NIL; }
void gui_set_resizable(SnaskValue* out, SnaskValue* _w, SnaskValue* _b) { (void)_w; (void)_b; out->tag = (double)SNASK_NIL; }
void gui_autosize(SnaskValue* out, SnaskValue* _w) { (void)_w; out->tag = (double)SNASK_NIL; }
void gui_vbox(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_hbox(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_scrolled(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_listbox(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_list_add_text(SnaskValue* out, SnaskValue* _l, SnaskValue* _t) { (void)_l; (void)_t; out->tag = (double)SNASK_NIL; }
void gui_on_select_ctx(SnaskValue* out, SnaskValue* _l, SnaskValue* _h, SnaskValue* _c) { (void)_l; (void)_h; (void)_c; out->tag = (double)SNASK_NIL; }
void gui_set_child(SnaskValue* out, SnaskValue* _p, SnaskValue* _c) { (void)_p; (void)_c; out->tag = (double)SNASK_NIL; }
void gui_add(SnaskValue* out, SnaskValue* _b, SnaskValue* _c) { (void)_b; (void)_c; out->tag = (double)SNASK_NIL; }
void gui_add_expand(SnaskValue* out, SnaskValue* _b, SnaskValue* _c) { (void)_b; (void)_c; out->tag = (double)SNASK_NIL; }
void gui_label(SnaskValue* out, SnaskValue* _t) { (void)_t; out->tag = (double)SNASK_NIL; }
void gui_entry(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_set_placeholder(SnaskValue* out, SnaskValue* _e, SnaskValue* _t) { (void)_e; (void)_t; out->tag = (double)SNASK_NIL; }
void gui_set_editable(SnaskValue* out, SnaskValue* _e, SnaskValue* _b) { (void)_e; (void)_b; out->tag = (double)SNASK_NIL; }
void gui_button(SnaskValue* out, SnaskValue* _t) { (void)_t; out->tag = (double)SNASK_NIL; }
void gui_set_enabled(SnaskValue* out, SnaskValue* _w, SnaskValue* _b) { (void)_w; (void)_b; out->tag = (double)SNASK_NIL; }
void gui_set_visible(SnaskValue* out, SnaskValue* _w, SnaskValue* _b) { (void)_w; (void)_b; out->tag = (double)SNASK_NIL; }
void gui_show_all(SnaskValue* out, SnaskValue* _w) { (void)_w; out->tag = (double)SNASK_NIL; }
void gui_set_text(SnaskValue* out, SnaskValue* _w, SnaskValue* _t) { (void)_w; (void)_t; out->tag = (double)SNASK_NIL; }
void gui_get_text(SnaskValue* out, SnaskValue* _w) { (void)_w; out->tag = (double)SNASK_NIL; }
void gui_on_click(SnaskValue* out, SnaskValue* _w, SnaskValue* _h) { (void)_w; (void)_h; out->tag = (double)SNASK_NIL; }
void gui_on_click_ctx(SnaskValue* out, SnaskValue* _w, SnaskValue* _h, SnaskValue* _c) { (void)_w; (void)_h; (void)_c; out->tag = (double)SNASK_NIL; }
void gui_separator_h(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_separator_v(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_msg_info(SnaskValue* out, SnaskValue* _t, SnaskValue* _m) { (void)_t; (void)_m; out->tag = (double)SNASK_NIL; }
void gui_msg_error(SnaskValue* out, SnaskValue* _t, SnaskValue* _m) { (void)_t; (void)_m; out->tag = (double)SNASK_NIL; }

#endif

// ---------------- calc helpers ----------------
void str_to_num(SnaskValue* out, SnaskValue* s) {
    if (!s || (int)s->tag != SNASK_STR || !s->ptr) { out->tag = (double)SNASK_NIL; return; }
    char* end = NULL;
    double v = strtod((const char*)s->ptr, &end);
    if (end == (char*)s->ptr) { out->tag = (double)SNASK_NIL; return; }
    out->tag = (double)SNASK_NUM;
    out->num = v;
    out->ptr = NULL;
}

void num_to_str(SnaskValue* out, SnaskValue* n) {
    if (!n || (int)n->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; return; }
    char buf[128];
    snprintf(buf, sizeof(buf), "%.15g", n->num);
    out->tag = (double)SNASK_STR;
    out->ptr = snask_gc_strdup(buf);
    out->num = 0;
}

typedef struct {
    const char* s;
    size_t i;
} CalcLexer;

static void calc_skip_ws(CalcLexer* lx) {
    while (lx->s[lx->i] && isspace((unsigned char)lx->s[lx->i])) lx->i++;
}

static int calc_peek(CalcLexer* lx) {
    calc_skip_ws(lx);
    return lx->s[lx->i] ? lx->s[lx->i] : 0;
}

static int calc_get(CalcLexer* lx) {
    calc_skip_ws(lx);
    return lx->s[lx->i] ? lx->s[lx->i++] : 0;
}

static int calc_prec(char op) {
    if (op == '+' || op == '-') return 1;
    if (op == '*' || op == '/') return 2;
    return 0;
}

static bool calc_apply(char op, double a, double b, double* out) {
    switch (op) {
        case '+': *out = a + b; return true;
        case '-': *out = a - b; return true;
        case '*': *out = a * b; return true;
        case '/': if (b == 0.0) return false; *out = a / b; return true;
        default: return false;
    }
}

// Shunting-yard evaluator for + - * / and parentheses.
// Returns 1 on success, 0 on error.
static int calc_eval_c(const char* expr, double* result) {
    double vals[256];
    char ops[256];
    int vtop = -1, otop = -1;
    CalcLexer lx = { expr ? expr : "", 0 };

    bool expect_value = true;
    while (1) {
        int c = calc_peek(&lx);
        if (!c) break;

        if (c == '(') {
            calc_get(&lx);
            ops[++otop] = '(';
            expect_value = true;
            continue;
        }
        if (c == ')') {
            calc_get(&lx);
            while (otop >= 0 && ops[otop] != '(') {
                if (vtop < 1) return 0;
                double b = vals[vtop--];
                double a = vals[vtop--];
                double r;
                if (!calc_apply(ops[otop--], a, b, &r)) return 0;
                vals[++vtop] = r;
            }
            if (otop < 0 || ops[otop] != '(') return 0;
            otop--;
            expect_value = false;
            continue;
        }

        if ((c == '+' || c == '-' || c == '*' || c == '/') && !expect_value) {
            char op = (char)calc_get(&lx);
            while (otop >= 0 && ops[otop] != '(' && calc_prec(ops[otop]) >= calc_prec(op)) {
                if (vtop < 1) return 0;
                double b = vals[vtop--];
                double a = vals[vtop--];
                double r;
                if (!calc_apply(ops[otop--], a, b, &r)) return 0;
                vals[++vtop] = r;
            }
            ops[++otop] = op;
            expect_value = true;
            continue;
        }

        // number (also allow unary + / -)
        if (expect_value && (c == '+' || c == '-')) {
            // unary sign
            char sign = (char)calc_get(&lx);
            int c2 = calc_peek(&lx);
            if (!(isdigit(c2) || c2 == '.')) return 0;
            char* end = NULL;
            double v = strtod(expr + lx.i, &end);
            if (end == expr + lx.i) return 0;
            lx.i = (size_t)(end - expr);
            if (sign == '-') v = -v;
            vals[++vtop] = v;
            expect_value = false;
            continue;
        }

        if (isdigit(c) || c == '.') {
            char* end = NULL;
            double v = strtod(expr + lx.i, &end);
            if (end == expr + lx.i) return 0;
            lx.i = (size_t)(end - expr);
            vals[++vtop] = v;
            expect_value = false;
            continue;
        }

        return 0;
    }

    while (otop >= 0) {
        if (ops[otop] == '(') return 0;
        if (vtop < 1) return 0;
        double b = vals[vtop--];
        double a = vals[vtop--];
        double r;
        if (!calc_apply(ops[otop--], a, b, &r)) return 0;
        vals[++vtop] = r;
    }
    if (vtop != 0) return 0;
    *result = vals[0];
    return 1;
}

void calc_eval(SnaskValue* out, SnaskValue* expr) {
    if (!expr || (int)expr->tag != SNASK_STR || !expr->ptr) { out->tag = (double)SNASK_NIL; return; }
    double r = 0.0;
    if (!calc_eval_c((const char*)expr->ptr, &r)) { out->tag = (double)SNASK_NIL; return; }
    out->tag = (double)SNASK_NUM;
    out->num = r;
    out->ptr = NULL;
}

#ifdef SNASK_SQLITE
// ---------------- SQLite (MVP) ----------------
static char* sqlite_ptr_to_handle(void* p) {
    char buf[64];
    snprintf(buf, sizeof(buf), "%p", p);
    return snask_gc_strdup(buf);
}

static void* sqlite_handle_to_ptr(const char* h) {
    if (!h) return NULL;
    void* p = NULL;
    sscanf(h, "%p", &p);
    return p;
}

// sqlite_open(path) -> handle(str) ou nil
void sqlite_open(SnaskValue* out, SnaskValue* path) {
    if (!path || (int)path->tag != SNASK_STR || !path->ptr) { out->tag = (double)SNASK_NIL; return; }
    sqlite3* db = NULL;
    int rc = sqlite3_open((const char*)path->ptr, &db);
    if (rc != SQLITE_OK || !db) { if (db) sqlite3_close(db); out->tag = (double)SNASK_NIL; return; }
    out->tag = (double)SNASK_STR;
    out->ptr = sqlite_ptr_to_handle(db);
    out->num = 0;
}

void sqlite_close(SnaskValue* out, SnaskValue* handle) {
    if (!handle || (int)handle->tag != SNASK_STR || !handle->ptr) { out->tag = (double)SNASK_NIL; return; }
    sqlite3* db = (sqlite3*)sqlite_handle_to_ptr((const char*)handle->ptr);
    if (!db) { out->tag = (double)SNASK_NIL; return; }
    sqlite3_close(db);
    out->tag = (double)SNASK_BOOL;
    out->num = 1.0;
    out->ptr = NULL;
}

void sqlite_exec(SnaskValue* out, SnaskValue* handle, SnaskValue* sql) {
    if (!handle || !sql || (int)handle->tag != SNASK_STR || (int)sql->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    sqlite3* db = (sqlite3*)sqlite_handle_to_ptr((const char*)handle->ptr);
    if (!db) { out->tag = (double)SNASK_NIL; return; }
    char* err = NULL;
    int rc = sqlite3_exec(db, (const char*)sql->ptr, NULL, NULL, &err);
    if (err) sqlite3_free(err);
    out->tag = (double)SNASK_BOOL;
    out->num = (rc == SQLITE_OK) ? 1.0 : 0.0;
    out->ptr = NULL;
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

// sqlite_query(handle, sql) -> array(obj) ou nil
// Retorna array JSON parseado: [ {col: val, ...}, ... ]
void sqlite_query(SnaskValue* out, SnaskValue* handle, SnaskValue* sql) {
    if (!handle || !sql || (int)handle->tag != SNASK_STR || (int)sql->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    sqlite3* db = (sqlite3*)sqlite_handle_to_ptr((const char*)handle->ptr);
    if (!db) { out->tag = (double)SNASK_NIL; return; }

    sqlite3_stmt* stmt = NULL;
    int rc = sqlite3_prepare_v2(db, (const char*)sql->ptr, -1, &stmt, NULL);
    if (rc != SQLITE_OK || !stmt) { out->tag = (double)SNASK_NIL; return; }

    SqlBuf sb; sqlb_init(&sb);
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

    // parse JSON string into SnaskValue
    SnaskValue tmp;
    tmp.tag = (double)SNASK_STR;
    tmp.ptr = sb.data; // take ownership
    tmp.num = 0;
    json_parse(out, &tmp);
    // json_parse does not take ownership of tmp.ptr; free buffer
    free(sb.data);
}

// ---------------- SQLite Stmt API (para ORM/queries seguras) ----------------
// sqlite_prepare(db_handle, sql) -> stmt_handle(str) ou nil
void sqlite_prepare(SnaskValue* out, SnaskValue* handle, SnaskValue* sql) {
    if (!handle || !sql || (int)handle->tag != SNASK_STR || (int)sql->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    sqlite3* db = (sqlite3*)sqlite_handle_to_ptr((const char*)handle->ptr);
    if (!db) { out->tag = (double)SNASK_NIL; return; }
    sqlite3_stmt* stmt = NULL;
    int rc = sqlite3_prepare_v2(db, (const char*)sql->ptr, -1, &stmt, NULL);
    if (rc != SQLITE_OK || !stmt) { out->tag = (double)SNASK_NIL; return; }
    out->tag = (double)SNASK_STR;
    out->ptr = sqlite_ptr_to_handle(stmt);
    out->num = 0;
}

// sqlite_finalize(stmt_handle) -> bool
void sqlite_finalize(SnaskValue* out, SnaskValue* stmt_h) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if (!stmt_h || (int)stmt_h->tag != SNASK_STR || !stmt_h->ptr) return;
    sqlite3_stmt* st = (sqlite3_stmt*)sqlite_handle_to_ptr((const char*)stmt_h->ptr);
    if (!st) return;
    sqlite3_finalize(st);
    out->num = 1.0;
}

// sqlite_reset(stmt_handle) -> bool
void sqlite_reset(SnaskValue* out, SnaskValue* stmt_h) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if (!stmt_h || (int)stmt_h->tag != SNASK_STR || !stmt_h->ptr) return;
    sqlite3_stmt* st = (sqlite3_stmt*)sqlite_handle_to_ptr((const char*)stmt_h->ptr);
    if (!st) return;
    int rc = sqlite3_reset(st);
    out->num = (rc == SQLITE_OK) ? 1.0 : 0.0;
}

// sqlite_bind_text(stmt_handle, idx, text) -> bool  (idx começa em 1)
void sqlite_bind_text(SnaskValue* out, SnaskValue* stmt_h, SnaskValue* idx_v, SnaskValue* txt) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if (!stmt_h || !idx_v || !txt || (int)stmt_h->tag != SNASK_STR || (int)idx_v->tag != SNASK_NUM || (int)txt->tag != SNASK_STR) return;
    sqlite3_stmt* st = (sqlite3_stmt*)sqlite_handle_to_ptr((const char*)stmt_h->ptr);
    if (!st) return;
    int idx = (int)idx_v->num;
    int rc = sqlite3_bind_text(st, idx, (const char*)txt->ptr, -1, SQLITE_TRANSIENT);
    out->num = (rc == SQLITE_OK) ? 1.0 : 0.0;
}

// sqlite_bind_num(stmt_handle, idx, num) -> bool
void sqlite_bind_num(SnaskValue* out, SnaskValue* stmt_h, SnaskValue* idx_v, SnaskValue* num_v) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if (!stmt_h || !idx_v || !num_v || (int)stmt_h->tag != SNASK_STR || (int)idx_v->tag != SNASK_NUM || (int)num_v->tag != SNASK_NUM) return;
    sqlite3_stmt* st = (sqlite3_stmt*)sqlite_handle_to_ptr((const char*)stmt_h->ptr);
    if (!st) return;
    int idx = (int)idx_v->num;
    int rc = sqlite3_bind_double(st, idx, (double)num_v->num);
    out->num = (rc == SQLITE_OK) ? 1.0 : 0.0;
}

// sqlite_bind_null(stmt_handle, idx) -> bool
void sqlite_bind_null(SnaskValue* out, SnaskValue* stmt_h, SnaskValue* idx_v) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if (!stmt_h || !idx_v || (int)stmt_h->tag != SNASK_STR || (int)idx_v->tag != SNASK_NUM) return;
    sqlite3_stmt* st = (sqlite3_stmt*)sqlite_handle_to_ptr((const char*)stmt_h->ptr);
    if (!st) return;
    int idx = (int)idx_v->num;
    int rc = sqlite3_bind_null(st, idx);
    out->num = (rc == SQLITE_OK) ? 1.0 : 0.0;
}

// sqlite_step(stmt_handle) -> bool (true se retornou uma linha; false se DONE/erro)
void sqlite_step(SnaskValue* out, SnaskValue* stmt_h) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if (!stmt_h || (int)stmt_h->tag != SNASK_STR || !stmt_h->ptr) return;
    sqlite3_stmt* st = (sqlite3_stmt*)sqlite_handle_to_ptr((const char*)stmt_h->ptr);
    if (!st) return;
    int rc = sqlite3_step(st);
    out->num = (rc == SQLITE_ROW) ? 1.0 : 0.0;
}

// sqlite_column(stmt_handle, idx0) -> any (idx0 começa em 0)
void sqlite_column(SnaskValue* out, SnaskValue* stmt_h, SnaskValue* idx_v) {
    if (!stmt_h || !idx_v || (int)stmt_h->tag != SNASK_STR || (int)idx_v->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    sqlite3_stmt* st = (sqlite3_stmt*)sqlite_handle_to_ptr((const char*)stmt_h->ptr);
    if (!st) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    int idx = (int)idx_v->num;
    int t = sqlite3_column_type(st, idx);
    if (t == SQLITE_NULL) {
        out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return;
    }
    if (t == SQLITE_INTEGER) {
        out->tag = (double)SNASK_NUM; out->ptr = NULL; out->num = (double)sqlite3_column_int64(st, idx); return;
    }
    if (t == SQLITE_FLOAT) {
        out->tag = (double)SNASK_NUM; out->ptr = NULL; out->num = (double)sqlite3_column_double(st, idx); return;
    }
    const unsigned char* txt = sqlite3_column_text(st, idx);
    out->tag = (double)SNASK_STR;
    out->ptr = snask_gc_strdup((const char*)(txt ? txt : (const unsigned char*)""));
    out->num = 0;
}

// sqlite_column_count(stmt_handle) -> num
void sqlite_column_count(SnaskValue* out, SnaskValue* stmt_h) {
    if (!stmt_h || (int)stmt_h->tag != SNASK_STR || !stmt_h->ptr) { out->tag = (double)SNASK_NIL; return; }
    sqlite3_stmt* st = (sqlite3_stmt*)sqlite_handle_to_ptr((const char*)stmt_h->ptr);
    if (!st) { out->tag = (double)SNASK_NIL; return; }
    out->tag = (double)SNASK_NUM;
    out->ptr = NULL;
    out->num = (double)sqlite3_column_count(st);
}

// sqlite_column_name(stmt_handle, idx0) -> str
void sqlite_column_name(SnaskValue* out, SnaskValue* stmt_h, SnaskValue* idx_v) {
    if (!stmt_h || !idx_v || (int)stmt_h->tag != SNASK_STR || (int)idx_v->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; return; }
    sqlite3_stmt* st = (sqlite3_stmt*)sqlite_handle_to_ptr((const char*)stmt_h->ptr);
    if (!st) { out->tag = (double)SNASK_NIL; return; }
    int idx = (int)idx_v->num;
    const char* n = sqlite3_column_name(st, idx);
    out->tag = (double)SNASK_STR;
    out->ptr = snask_gc_strdup(n ? n : "");
    out->num = 0;
}
#endif

// --- JSON ---
typedef struct {
    char* data;
    size_t len;
    size_t cap;
} StrBuf;

static void sb_init(StrBuf* sb) {
    sb->cap = 256;
    sb->len = 0;
    sb->data = (char*)malloc(sb->cap);
    sb->data[0] = '\0';
}

static void sb_reserve(StrBuf* sb, size_t extra) {
    size_t needed = sb->len + extra + 1;
    if (needed <= sb->cap) return;
    while (sb->cap < needed) sb->cap *= 2;
    sb->data = (char*)realloc(sb->data, sb->cap);
}

static void sb_append_char(StrBuf* sb, char c) {
    sb_reserve(sb, 1);
    sb->data[sb->len++] = c;
    sb->data[sb->len] = '\0';
}

static void sb_append_cstr(StrBuf* sb, const char* s) {
    size_t n = strlen(s);
    sb_reserve(sb, n);
    memcpy(sb->data + sb->len, s, n);
    sb->len += n;
    sb->data[sb->len] = '\0';
}

static void sb_append_indent(StrBuf* sb, int level, int indent) {
    for (int i = 0; i < level * indent; i++) sb_append_char(sb, ' ');
}

static void sb_append_json_escaped(StrBuf* sb, const char* s) {
    sb_append_char(sb, '"');
    for (const unsigned char* p = (const unsigned char*)s; *p; p++) {
        unsigned char ch = *p;
        switch (ch) {
            case '\"': sb_append_cstr(sb, "\\\""); break;
            case '\\': sb_append_cstr(sb, "\\\\"); break;
            case '\b': sb_append_cstr(sb, "\\b"); break;
            case '\f': sb_append_cstr(sb, "\\f"); break;
            case '\n': sb_append_cstr(sb, "\\n"); break;
            case '\r': sb_append_cstr(sb, "\\r"); break;
            case '\t': sb_append_cstr(sb, "\\t"); break;
            default:
                if (ch < 0x20) {
                    char tmp[7];
                    snprintf(tmp, sizeof(tmp), "\\u%04x", (unsigned int)ch);
                    sb_append_cstr(sb, tmp);
                } else {
                    sb_append_char(sb, (char)ch);
                }
        }
    }
    sb_append_char(sb, '"');
}

static void json_stringify_into(StrBuf* sb, SnaskValue* v, bool pretty, int indent, int level);

static void json_stringify_object_into(StrBuf* sb, SnaskObject* obj, bool pretty, int indent, int level) {
    sb_append_char(sb, '{');
    if (pretty && obj->count > 0) sb_append_char(sb, '\n');
    for (int i = 0; i < obj->count; i++) {
        if (pretty) sb_append_indent(sb, level + 1, indent);
        sb_append_json_escaped(sb, obj->names[i] ? obj->names[i] : "");
        sb_append_char(sb, ':');
        if (pretty) sb_append_char(sb, ' ');
        json_stringify_into(sb, &obj->values[i], pretty, indent, level + 1);
        if (i < obj->count - 1) sb_append_char(sb, ',');
        if (pretty) sb_append_char(sb, '\n');
    }
    if (pretty && obj->count > 0) sb_append_indent(sb, level, indent);
    sb_append_char(sb, '}');
}

static void json_stringify_into(StrBuf* sb, SnaskValue* v, bool pretty, int indent, int level) {
    int tag = (int)v->tag;
    if (tag == SNASK_NUM) {
        char tmp[64];
        snprintf(tmp, sizeof(tmp), "%g", v->num);
        sb_append_cstr(sb, tmp);
    } else if (tag == SNASK_STR) {
        sb_append_json_escaped(sb, (const char*)v->ptr);
    } else if (tag == SNASK_BOOL) {
        sb_append_cstr(sb, v->num ? "true" : "false");
    } else if (tag == SNASK_OBJ) {
        SnaskObject* obj = (SnaskObject*)v->ptr;
        json_stringify_object_into(sb, obj, pretty, indent, level);
    } else {
        sb_append_cstr(sb, "null");
    }
}

void s_json_stringify(SnaskValue* out, SnaskValue* v) {
    StrBuf sb;
    sb_init(&sb);
    json_stringify_into(&sb, v, false, 0, 0);
    out->tag = (double)SNASK_STR;
    out->ptr = sb.data;
    out->num = 0;
}

void json_stringify(SnaskValue* out, SnaskValue* v) {
    s_json_stringify(out, v);
}

void json_stringify_pretty(SnaskValue* out, SnaskValue* v) {
    StrBuf sb;
    sb_init(&sb);
    json_stringify_into(&sb, v, true, 2, 0);
    out->tag = (double)SNASK_STR;
    out->ptr = sb.data;
    out->num = 0;
}

typedef struct {
    const char* s;
    size_t i;
    const char* err;
} JsonParser;

static void jp_skip_ws(JsonParser* p) {
    while (p->s[p->i] && (p->s[p->i] == ' ' || p->s[p->i] == '\n' || p->s[p->i] == '\r' || p->s[p->i] == '\t')) p->i++;
}

static bool jp_consume(JsonParser* p, char ch) {
    jp_skip_ws(p);
    if (p->s[p->i] == ch) { p->i++; return true; }
    return false;
}

static bool jp_match(JsonParser* p, const char* lit) {
    jp_skip_ws(p);
    size_t n = strlen(lit);
    if (strncmp(p->s + p->i, lit, n) == 0) { p->i += n; return true; }
    return false;
}

static char jp_next(JsonParser* p) { return p->s[p->i]; }

static char* jp_parse_string(JsonParser* p) {
    jp_skip_ws(p);
    if (p->s[p->i] != '"') { p->err = "Esperado '\"' no início da string JSON."; return NULL; }
    p->i++; // skip opening quote
    StrBuf sb;
    sb_init(&sb);
    while (p->s[p->i]) {
        char c = p->s[p->i++];
        if (c == '"') {
            return sb.data;
        }
        if (c == '\\') {
            char esc = p->s[p->i++];
            switch (esc) {
                case '"': sb_append_char(&sb, '"'); break;
                case '\\': sb_append_char(&sb, '\\'); break;
                case '/': sb_append_char(&sb, '/'); break;
                case 'b': sb_append_char(&sb, '\b'); break;
                case 'f': sb_append_char(&sb, '\f'); break;
                case 'n': sb_append_char(&sb, '\n'); break;
                case 'r': sb_append_char(&sb, '\r'); break;
                case 't': sb_append_char(&sb, '\t'); break;
                case 'u': {
                    // \uXXXX (suporte básico: somente para ASCII <= 0x7F)
                    unsigned int code = 0;
                    for (int k = 0; k < 4; k++) {
                        char h = p->s[p->i++];
                        if (!isxdigit((unsigned char)h)) { p->err = "Escape \\u inválido em string JSON."; free(sb.data); return NULL; }
                        code = (code << 4) | (unsigned int)(isdigit((unsigned char)h) ? (h - '0') : (tolower((unsigned char)h) - 'a' + 10));
                    }
                    if (code <= 0x7F) sb_append_char(&sb, (char)code);
                    else sb_append_char(&sb, '?');
                    break;
                }
                default:
                    p->err = "Escape inválido em string JSON.";
                    free(sb.data);
                    return NULL;
            }
        } else {
            sb_append_char(&sb, c);
        }
    }
    p->err = "String JSON não terminada.";
    free(sb.data);
    return NULL;
}

static SnaskValue jp_parse_value(JsonParser* p);

static SnaskValue make_nil(void) { return (SnaskValue){(double)SNASK_NIL, 0, NULL}; }
static SnaskValue make_bool(bool b) { return (SnaskValue){(double)SNASK_BOOL, b ? 1.0 : 0.0, NULL}; }
static SnaskValue make_num(double n) { return (SnaskValue){(double)SNASK_NUM, n, NULL}; }
static SnaskValue make_str(char* s) { return (SnaskValue){(double)SNASK_STR, 0, s}; }
static SnaskValue make_obj(SnaskObject* o) { return (SnaskValue){(double)SNASK_OBJ, 0, o}; }

static SnaskObject* obj_new(int count) {
    SnaskObject* obj = (SnaskObject*)malloc(sizeof(SnaskObject));
    obj->count = count;
    obj->names = (char**)calloc((size_t)count, sizeof(char*));
    obj->values = (SnaskValue*)calloc((size_t)count, sizeof(SnaskValue));
    return obj;
}

static void obj_push(SnaskObject** objp, int* cap, int* len, char* name, SnaskValue val) {
    if (*len >= *cap) {
        int new_cap = (*cap == 0) ? 8 : (*cap * 2);
        SnaskObject* obj = *objp;
        obj->names = (char**)realloc(obj->names, (size_t)new_cap * sizeof(char*));
        obj->values = (SnaskValue*)realloc(obj->values, (size_t)new_cap * sizeof(SnaskValue));
        for (int i = *cap; i < new_cap; i++) { obj->names[i] = NULL; obj->values[i] = make_nil(); }
        *cap = new_cap;
    }
    (*objp)->names[*len] = name;
    (*objp)->values[*len] = val;
    (*len)++;
    (*objp)->count = *len;
}

static SnaskValue jp_parse_object(JsonParser* p) {
    if (!jp_consume(p, '{')) { p->err = "Esperado '{'."; return make_nil(); }
    SnaskObject* obj = obj_new(0);
    int cap = 0, len = 0;

    jp_skip_ws(p);
    if (jp_consume(p, '}')) return make_obj(obj);

    while (p->s[p->i]) {
        char* key = jp_parse_string(p);
        if (!key) { free(obj->names); free(obj->values); free(obj); return make_nil(); }
        if (!jp_consume(p, ':')) { p->err = "Esperado ':' após chave do objeto JSON."; free(key); free(obj->names); free(obj->values); free(obj); return make_nil(); }
        SnaskValue val = jp_parse_value(p);
        if (p->err) { free(key); free(obj->names); free(obj->values); free(obj); return make_nil(); }
        obj_push(&obj, &cap, &len, key, val);
        jp_skip_ws(p);
        if (jp_consume(p, '}')) return make_obj(obj);
        if (!jp_consume(p, ',')) { p->err = "Esperado ',' ou '}' em objeto JSON."; free(obj->names); free(obj->values); free(obj); return make_nil(); }
    }
    p->err = "Objeto JSON não terminado.";
    free(obj->names); free(obj->values); free(obj);
    return make_nil();
}

static SnaskValue jp_parse_array(JsonParser* p) {
    if (!jp_consume(p, '[')) { p->err = "Esperado '['."; return make_nil(); }
    SnaskObject* arr = obj_new(0);
    int cap = 0, len = 0;

    jp_skip_ws(p);
    if (jp_consume(p, ']')) return make_obj(arr);

    while (p->s[p->i]) {
        SnaskValue val = jp_parse_value(p);
        if (p->err) { free(arr->names); free(arr->values); free(arr); return make_nil(); }
        char idx_name[32];
        snprintf(idx_name, sizeof(idx_name), "%d", len);
        obj_push(&arr, &cap, &len, snask_gc_strdup(idx_name), val);
        jp_skip_ws(p);
        if (jp_consume(p, ']')) return make_obj(arr);
        if (!jp_consume(p, ',')) { p->err = "Esperado ',' ou ']' em array JSON."; free(arr->names); free(arr->values); free(arr); return make_nil(); }
    }
    p->err = "Array JSON não terminado.";
    free(arr->names); free(arr->values); free(arr);
    return make_nil();
}

static SnaskValue jp_parse_number(JsonParser* p) {
    jp_skip_ws(p);
    char* end = NULL;
    double n = strtod(p->s + p->i, &end);
    if (end == p->s + p->i) { p->err = "Número JSON inválido."; return make_nil(); }
    p->i += (size_t)(end - (p->s + p->i));
    return make_num(n);
}

static SnaskValue jp_parse_value(JsonParser* p) {
    jp_skip_ws(p);
    char c = jp_next(p);
    if (c == '\0') { p->err = "JSON vazio."; return make_nil(); }
    if (c == '"') {
        char* s = jp_parse_string(p);
        if (!s) return make_nil();
        return make_str(s);
    }
    if (c == '{') return jp_parse_object(p);
    if (c == '[') return jp_parse_array(p);
    if (jp_match(p, "null")) return make_nil();
    if (jp_match(p, "true")) return make_bool(true);
    if (jp_match(p, "false")) return make_bool(false);
    if (c == '-' || (c >= '0' && c <= '9')) return jp_parse_number(p);
    p->err = "Token inesperado no JSON.";
    return make_nil();
}

void json_parse(SnaskValue* out, SnaskValue* data) {
    if ((int)data->tag != SNASK_STR || !data->ptr) { out->tag = (double)SNASK_NIL; return; }
    JsonParser p = { .s = (const char*)data->ptr, .i = 0, .err = NULL };
    SnaskValue v = jp_parse_value(&p);
    if (p.err) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    jp_skip_ws(&p);
    if (p.s[p.i] != '\0') { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    *out = v;
}

// Retorna um objeto: { ok: bool, value: any, error: str }
void json_parse_ex(SnaskValue* out, SnaskValue* data) {
    if ((int)data->tag != SNASK_STR || !data->ptr) { out->tag = (double)SNASK_NIL; return; }

    JsonParser p = { .s = (const char*)data->ptr, .i = 0, .err = NULL };
    SnaskValue v = jp_parse_value(&p);
    const char* err = p.err;
    if (!err) {
        jp_skip_ws(&p);
        if (p.s[p.i] != '\0') err = "Conteúdo extra após o JSON.";
    }

    SnaskObject* obj = obj_new(3);
    obj->names[0] = snask_gc_strdup("ok");
    obj->names[1] = snask_gc_strdup("value");
    obj->names[2] = snask_gc_strdup("error");
    obj->values[0] = make_bool(err == NULL);
    obj->values[1] = (err == NULL) ? v : make_nil();
    obj->values[2] = make_str(snask_gc_strdup(err ? err : ""));

    *out = make_obj(obj);
}

void json_get(SnaskValue* out, SnaskValue* obj_val, SnaskValue* key_val) {
    if ((int)obj_val->tag != SNASK_OBJ || (int)key_val->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    SnaskObject* obj = (SnaskObject*)obj_val->ptr;
    const char* key = (const char*)key_val->ptr;
    for (int i = 0; i < obj->count; i++) {
        if (obj->names[i] && strcmp(obj->names[i], key) == 0) { *out = obj->values[i]; return; }
    }
    out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0;
}

void json_has(SnaskValue* out, SnaskValue* obj_val, SnaskValue* key_val) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if ((int)obj_val->tag != SNASK_OBJ || (int)key_val->tag != SNASK_STR) return;
    SnaskObject* obj = (SnaskObject*)obj_val->ptr;
    const char* key = (const char*)key_val->ptr;
    for (int i = 0; i < obj->count; i++) {
        if (obj->names[i] && strcmp(obj->names[i], key) == 0) { out->num = 1.0; return; }
    }
}

void json_len(SnaskValue* out, SnaskValue* obj_val) {
    out->tag = (double)SNASK_NUM;
    out->ptr = NULL;
    if ((int)obj_val->tag != SNASK_OBJ) { out->num = 0; return; }
    SnaskObject* obj = (SnaskObject*)obj_val->ptr;
    out->num = (double)obj->count;
}

void json_index(SnaskValue* out, SnaskValue* obj_val, SnaskValue* idx_val) {
    if ((int)obj_val->tag != SNASK_OBJ || (int)idx_val->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; return; }
    SnaskObject* obj = (SnaskObject*)obj_val->ptr;
    int idx = (int)idx_val->num;
    if (idx < 0 || idx >= obj->count) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    *out = obj->values[idx];
}

void json_set(SnaskValue* out, SnaskValue* obj_val, SnaskValue* key_val, SnaskValue* value) {
    out->tag = (double)SNASK_BOOL;
    out->ptr = NULL;
    out->num = 0;
    if ((int)obj_val->tag != SNASK_OBJ || (int)key_val->tag != SNASK_STR) return;
    SnaskObject* obj = (SnaskObject*)obj_val->ptr;
    const char* key = (const char*)key_val->ptr;

    for (int i = 0; i < obj->count; i++) {
        if (obj->names[i] && strcmp(obj->names[i], key) == 0) {
            obj->values[i] = *value;
            out->num = 1.0;
            return;
        }
    }

    int new_count = obj->count + 1;
    obj->names = (char**)realloc(obj->names, (size_t)new_count * sizeof(char*));
    obj->values = (SnaskValue*)realloc(obj->values, (size_t)new_count * sizeof(SnaskValue));
    obj->names[new_count - 1] = snask_gc_strdup(key);
    obj->values[new_count - 1] = *value;
    obj->count = new_count;
    out->num = 1.0;
}

void json_keys(SnaskValue* out, SnaskValue* obj_val) {
    if ((int)obj_val->tag != SNASK_OBJ) { out->tag = (double)SNASK_NIL; out->ptr = NULL; out->num = 0; return; }
    SnaskObject* obj = (SnaskObject*)obj_val->ptr;
    int n = obj ? obj->count : 0;
    SnaskObject* arr = obj_new(n);
    for (int i = 0; obj && i < n; i++) {
        arr->names[i] = NULL;
        arr->values[i] = make_str(snask_gc_strdup(obj->names[i] ? obj->names[i] : ""));
    }
    *out = make_obj(arr);
}

void s_get_member(SnaskValue* out, SnaskValue* v_obj, SnaskValue* index_val) {
    if ((int)v_obj->tag != SNASK_OBJ) { out->tag = (double)SNASK_NIL; return; }
    SnaskObject* obj = (SnaskObject*)v_obj->ptr;
    int index = (int)index_val->num;
    if (index >= 0 && index < obj->count) *out = obj->values[index];
    else out->tag = (double)SNASK_NIL;
}

void s_set_member(SnaskValue* v_obj, SnaskValue* index_val, SnaskValue* value) {
    if ((int)v_obj->tag != SNASK_OBJ) return;
    SnaskObject* obj = (SnaskObject*)v_obj->ptr;
    int index = (int)index_val->num;
    if (index >= 0 && index < obj->count) obj->values[index] = *value;
}

// ------------------------------------------------------------
// Aliases "__" para nativas (acesso via bibliotecas)
// O compilador reescreve chamadas dentro de módulos importados para "__<nome>".
// ------------------------------------------------------------

#define SNASK_ALIAS0(name) void __##name(SnaskValue* out) { name(out); }
#define SNASK_ALIAS1(name) void __##name(SnaskValue* out, SnaskValue* a) { name(out, a); }
#define SNASK_ALIAS2(name) void __##name(SnaskValue* out, SnaskValue* a, SnaskValue* b) { name(out, a, b); }
#define SNASK_ALIAS3(name) void __##name(SnaskValue* out, SnaskValue* a, SnaskValue* b, SnaskValue* c) { name(out, a, b, c); }

// SFS / Path / OS / HTTP
SNASK_ALIAS1(sfs_read)
SNASK_ALIAS2(sfs_write)
SNASK_ALIAS2(sfs_append)
SNASK_ALIAS1(sfs_delete)
SNASK_ALIAS1(sfs_exists)
SNASK_ALIAS2(sfs_copy)
SNASK_ALIAS2(sfs_move)
SNASK_ALIAS1(sfs_mkdir)
SNASK_ALIAS1(sfs_is_file)
SNASK_ALIAS1(sfs_is_dir)
SNASK_ALIAS1(sfs_listdir)
SNASK_ALIAS1(sfs_size)
SNASK_ALIAS1(sfs_mtime)
SNASK_ALIAS1(sfs_rmdir)

SNASK_ALIAS1(path_basename)
SNASK_ALIAS1(path_dirname)
SNASK_ALIAS1(path_extname)
SNASK_ALIAS2(path_join)

SNASK_ALIAS0(os_cwd)
SNASK_ALIAS0(os_platform)
SNASK_ALIAS0(os_arch)
SNASK_ALIAS1(os_getenv)
SNASK_ALIAS2(os_setenv)
SNASK_ALIAS1(os_random_hex)

SNASK_ALIAS1(s_http_get)
SNASK_ALIAS2(s_http_post)
SNASK_ALIAS2(s_http_put)
SNASK_ALIAS1(s_http_delete)
SNASK_ALIAS2(s_http_patch)

// Blaze / Auth
SNASK_ALIAS2(blaze_run)
SNASK_ALIAS2(blaze_qs_get)
SNASK_ALIAS2(blaze_cookie_get)

SNASK_ALIAS1(auth_random_hex)
SNASK_ALIAS0(auth_now)
SNASK_ALIAS2(auth_const_time_eq)
SNASK_ALIAS1(auth_hash_password)
SNASK_ALIAS2(auth_verify_password)
SNASK_ALIAS0(auth_session_id)
SNASK_ALIAS0(auth_csrf_token)
SNASK_ALIAS2(auth_cookie_kv)
SNASK_ALIAS1(auth_cookie_session)
SNASK_ALIAS1(auth_cookie_delete)
SNASK_ALIAS1(auth_bearer_header)
SNASK_ALIAS0(auth_ok)
SNASK_ALIAS0(auth_fail)
SNASK_ALIAS0(auth_version)

// GUI
SNASK_ALIAS0(gui_init)
SNASK_ALIAS0(gui_run)
SNASK_ALIAS0(gui_quit)
SNASK_ALIAS3(gui_window)
SNASK_ALIAS2(gui_set_title)
SNASK_ALIAS2(gui_set_resizable)
SNASK_ALIAS1(gui_autosize)
SNASK_ALIAS0(gui_vbox)
SNASK_ALIAS0(gui_hbox)
SNASK_ALIAS0(gui_scrolled)
SNASK_ALIAS0(gui_listbox)
SNASK_ALIAS2(gui_list_add_text)
SNASK_ALIAS3(gui_on_select_ctx)
SNASK_ALIAS2(gui_set_child)
SNASK_ALIAS2(gui_add)
SNASK_ALIAS2(gui_add_expand)
SNASK_ALIAS1(gui_label)
SNASK_ALIAS0(gui_entry)
SNASK_ALIAS2(gui_set_placeholder)
SNASK_ALIAS2(gui_set_editable)
SNASK_ALIAS1(gui_button)
SNASK_ALIAS2(gui_set_enabled)
SNASK_ALIAS2(gui_set_visible)
SNASK_ALIAS1(gui_show_all)
SNASK_ALIAS2(gui_set_text)
SNASK_ALIAS1(gui_get_text)
SNASK_ALIAS2(gui_on_click)
SNASK_ALIAS3(gui_on_click_ctx)
SNASK_ALIAS0(gui_separator_h)
SNASK_ALIAS0(gui_separator_v)
SNASK_ALIAS2(gui_msg_info)
SNASK_ALIAS2(gui_msg_error)

// SQLite
SNASK_ALIAS1(sqlite_open)
SNASK_ALIAS1(sqlite_close)
SNASK_ALIAS2(sqlite_exec)
SNASK_ALIAS2(sqlite_query)
SNASK_ALIAS2(sqlite_prepare)
SNASK_ALIAS1(sqlite_finalize)
SNASK_ALIAS1(sqlite_reset)
SNASK_ALIAS3(sqlite_bind_text)
SNASK_ALIAS3(sqlite_bind_num)
SNASK_ALIAS2(sqlite_bind_null)
SNASK_ALIAS1(sqlite_step)
SNASK_ALIAS2(sqlite_column)
SNASK_ALIAS1(sqlite_column_count)
SNASK_ALIAS2(sqlite_column_name)

// Threads
SNASK_ALIAS2(thread_spawn)
SNASK_ALIAS1(thread_join)
SNASK_ALIAS1(thread_detach)

// JSON / SJSON
SNASK_ALIAS1(json_stringify)
SNASK_ALIAS1(json_stringify_pretty)
SNASK_ALIAS1(json_parse)
SNASK_ALIAS2(json_get)
SNASK_ALIAS2(json_has)
SNASK_ALIAS1(json_len)
SNASK_ALIAS2(json_index)
SNASK_ALIAS3(json_set)
SNASK_ALIAS1(json_keys)
SNASK_ALIAS1(json_parse_ex)

SNASK_ALIAS0(sjson_new_object)
SNASK_ALIAS0(sjson_new_array)
SNASK_ALIAS1(sjson_type)
SNASK_ALIAS1(sjson_arr_len)
SNASK_ALIAS2(sjson_arr_get)
SNASK_ALIAS3(sjson_arr_set)
SNASK_ALIAS2(sjson_arr_push)
SNASK_ALIAS2(sjson_path_get)

#undef SNASK_ALIAS0
#undef SNASK_ALIAS1
#undef SNASK_ALIAS2
#undef SNASK_ALIAS3
