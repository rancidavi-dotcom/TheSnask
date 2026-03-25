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

// Ultra-tiny builds use `-nostartfiles` (no CRT). Some libc paths (atexit) expect `__dso_handle`.
// Provide a weak default so we can link successfully when CRT isn't present.
void* __dso_handle __attribute__((weak)) = 0;

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

void s_print(SnaskValue* v) {
    if (!v) return;
    int t = (int)v->tag;
    switch (t) {
        case SNASK_NIL: printf("nil"); break;
        case SNASK_NUM: printf("%.15g", v->num); break;
        case SNASK_BOOL: printf(v->num != 0.0 ? "true" : "false"); break;
        case SNASK_STR: printf("%s", (char*)v->ptr); break;
        case SNASK_OBJ: {
            SnaskValue out = make_nil();
            json_stringify(&out, v);
            if ((int)out.tag == SNASK_STR) printf("%s", (char*)out.ptr);
            else printf("[object]");
            break;
        }
        default: printf("[unknown]"); break;
    }
}

void s_println(void) {
    printf("\n");
    fflush(stdout);
}
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

void s_get_member(SnaskValue* out, SnaskValue* obj_val, SnaskValue* idx_val) {
    if (!obj_val || (int)obj_val->tag != SNASK_OBJ || !obj_val->ptr) { *out = make_nil(); return; }
    if (!idx_val || (int)idx_val->tag != SNASK_NUM) { *out = make_nil(); return; }
    
    SnaskObject* obj = (SnaskObject*)obj_val->ptr;
    int idx = (int)idx_val->num;
    
    if (idx < 0 || idx >= obj->count) { *out = make_nil(); return; }
    *out = obj->values[idx];
}

void s_set_member(SnaskValue* obj_val, SnaskValue* idx_val, SnaskValue* val) {
    if (!obj_val || (int)obj_val->tag != SNASK_OBJ || !obj_val->ptr) return;
    if (!idx_val || (int)idx_val->tag != SNASK_NUM) return;
    
    SnaskObject* obj = (SnaskObject*)obj_val->ptr;
    int idx = (int)idx_val->num;
    
    if (idx < 0 || idx >= obj->count) return;
    obj->values[idx] = *val;
}

// Forward decls (usadas por seções posteriores)
static SnaskValue make_nil(void) {
    SnaskValue v;
    v.tag = (double)SNASK_NIL;
    v.num = 0;
    v.ptr = NULL;
    return v;
}

static SnaskValue make_bool(bool b) {
    SnaskValue v;
    v.tag = (double)SNASK_BOOL;
    v.num = b ? 1.0 : 0.0;
    v.ptr = NULL;
    return v;
}

static SnaskValue make_str(char* s) {
    SnaskValue v;
    v.tag = (double)SNASK_STR;
    v.num = 0;
    v.ptr = s;
    return v;
}

static SnaskValue make_obj(SnaskObject* o) {
    SnaskValue v;
    v.tag = (double)SNASK_OBJ;
    v.num = 0;
    v.ptr = o;
    return v;
}

static SnaskObject* obj_new(int count) {
    SnaskObject* o = (SnaskObject*)snask_gc_malloc(sizeof(SnaskObject));
    o->count = count;
    o->names = (char**)snask_gc_malloc(sizeof(char*) * count);
    o->values = (SnaskValue*)snask_gc_malloc(sizeof(SnaskValue) * count);
    for (int i = 0; i < count; i++) {
        o->names[i] = NULL;
        o->values[i] = make_nil();
    }
    return o;
}
void json_stringify(SnaskValue* out, SnaskValue* v);
void json_parse(SnaskValue* out, SnaskValue* data);

void s_alloc_obj(SnaskValue* out, SnaskValue* size_val, char** names) {
    if ((int)size_val->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; return; }
    int count = (int)size_val->num;
    SnaskObject* obj = obj_new(count);
    obj->names = names;
    out->tag = (double)SNASK_OBJ;
    out->ptr = obj;
    out->num = 0;
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

// --- Bench helpers (small files) ---
// These exist to avoid creating huge amounts of temporary Snask strings in tight loops.
// They are exposed only through the `sfs` library (restricted native).
void sfs_bench_create_small_files(SnaskValue* out, SnaskValue* dir, SnaskValue* n_files, SnaskValue* size_bytes) {
    out->tag = (double)SNASK_NUM;
    out->ptr = NULL;
    out->num = 0;
    if ((int)dir->tag != SNASK_STR || (int)n_files->tag != SNASK_NUM || (int)size_bytes->tag != SNASK_NUM) return;

    const char* base = (const char*)dir->ptr;
    int n = (int)n_files->num;
    int sz = (int)size_bytes->num;
    if (!base || n <= 0 || sz <= 0) return;

    char* buf = (char*)malloc((size_t)sz);
    if (!buf) return;
    memset(buf, 'a', (size_t)sz);

    int created = 0;
    for (int i = 0; i < n; i++) {
        char p[4096];
        int plen = snprintf(p, sizeof(p), "%s/f_%d.bin", base, i);
        if (plen <= 0 || plen >= (int)sizeof(p)) continue;
        int fd = open(p, O_WRONLY | O_CREAT | O_TRUNC, 0644);
        if (fd < 0) continue;
        ssize_t w = write(fd, buf, (size_t)sz);
        close(fd);
        if (w == (ssize_t)sz) created++;
    }

    free(buf);
    out->num = (double)created;
}

void sfs_bench_count_entries(SnaskValue* out, SnaskValue* dir) {
    out->tag = (double)SNASK_NUM;
    out->ptr = NULL;
    out->num = 0;
    if ((int)dir->tag != SNASK_STR || !dir->ptr) return;
    DIR* d = opendir((const char*)dir->ptr);
    if (!d) return;
    int count = 0;
    struct dirent* ent;
    while ((ent = readdir(d)) != NULL) {
        const char* name = ent->d_name;
        if (strcmp(name, ".") == 0 || strcmp(name, "..") == 0) continue;
        count++;
    }
    closedir(d);
    out->num = (double)count;
}

void sfs_bench_delete_small_files(SnaskValue* out, SnaskValue* dir, SnaskValue* n_files) {
    out->tag = (double)SNASK_NUM;
    out->ptr = NULL;
    out->num = 0;
    if ((int)dir->tag != SNASK_STR || (int)n_files->tag != SNASK_NUM) return;

    const char* base = (const char*)dir->ptr;
    int n = (int)n_files->num;
    if (!base || n <= 0) return;

    int deleted = 0;
    for (int i = 0; i < n; i++) {
        char p[4096];
        int plen = snprintf(p, sizeof(p), "%s/f_%d.bin", base, i);
        if (plen <= 0 || plen >= (int)sizeof(p)) continue;
        if (remove(p) == 0) deleted++;
    }
    out->num = (double)deleted;
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

void sfs_remove(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR) { out->tag = (double)SNASK_BOOL; out->num = 0; return; }
    int res = remove((char*)path->ptr); 
    out->tag = (double)SNASK_BOOL; out->num = (res == 0); out->ptr = NULL;
}

void sfs_exists(SnaskValue* out, SnaskValue* path) { 
    if ((int)path->tag != SNASK_STR) { out->tag = (double)SNASK_BOOL; out->num = 0; return; }
    int res = access((char*)path->ptr, F_OK); 
    out->tag = (double)SNASK_BOOL; out->num = (res == 0); out->ptr = NULL;
}

#ifdef SNASK_TINY
static inline double snask_tiny_fabs(double x) { return (x < 0.0) ? -x : x; }
static inline double snask_tiny_fmax(double a, double b) { return (a > b) ? a : b; }
static inline double snask_tiny_fmin(double a, double b) { return (a < b) ? a : b; }

static inline double snask_tiny_fmod(double a, double b) {
    // Best-effort fmod replacement for tiny builds (avoid linking libm).
    // Handles common cases; for b==0 returns NaN.
    if (b == 0.0) return 0.0 / 0.0;
    if (a != a) return a; // NaN
    if (b != b) return b; // NaN
    long long q = (long long)(a / b); // trunc toward zero
    return a - ((double)q) * b;
}
#endif

void s_abs(SnaskValue* out, SnaskValue* n) {
#ifdef SNASK_TINY
    *out = (SnaskValue){(double)SNASK_NUM, snask_tiny_fabs(n->num), NULL};
#else
    *out = (SnaskValue){(double)SNASK_NUM, fabs(n->num), NULL};
#endif
}
void s_max(SnaskValue* out, SnaskValue* a, SnaskValue* b) {
#ifdef SNASK_TINY
    *out = (SnaskValue){(double)SNASK_NUM, snask_tiny_fmax(a->num, b->num), NULL};
#else
    *out = (SnaskValue){(double)SNASK_NUM, fmax(a->num, b->num), NULL};
#endif
}
void s_min(SnaskValue* out, SnaskValue* a, SnaskValue* b) {
#ifdef SNASK_TINY
    *out = (SnaskValue){(double)SNASK_NUM, snask_tiny_fmin(a->num, b->num), NULL};
#else
    *out = (SnaskValue){(double)SNASK_NUM, fmin(a->num, b->num), NULL};
#endif
}

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
    SnaskValue a = make_str(snask_gc_strdup(t->arg ? t->arg : ""));
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
    SnaskValue wh = make_str(snask_gc_strdup(widget_handle ? widget_handle : ""));
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
    SnaskValue wh = make_str(snask_gc_strdup(widget_handle ? widget_handle : ""));
    SnaskValue cv = make_str(snask_gc_strdup(ctx ? ctx : ""));
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

void gui_eventbox(SnaskValue* out) {
    GtkWidget* eb = gtk_event_box_new();
    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(eb);
    out->num = 0;
}

void gui_scrolled(SnaskValue* out) {
    GtkWidget* sw = gtk_scrolled_window_new(NULL, NULL);
    gtk_scrolled_window_set_policy(GTK_SCROLLED_WINDOW(sw), GTK_POLICY_AUTOMATIC, GTK_POLICY_AUTOMATIC);
    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(sw);
    out->num = 0;
}

void gui_flowbox(SnaskValue* out) {
    GtkWidget* fb = gtk_flow_box_new();
    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(fb);
    out->num = 0;
}

void gui_flow_add(SnaskValue* out, SnaskValue* flow_h, SnaskValue* child_h) {
    if ((int)flow_h->tag != SNASK_STR || (int)child_h->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* flow = (GtkWidget*)gui_handle_to_ptr((const char*)flow_h->ptr);
    GtkWidget* child = (GtkWidget*)gui_handle_to_ptr((const char*)child_h->ptr);
    if (!flow || !child || !GTK_IS_FLOW_BOX(flow)) { out->tag = (double)SNASK_NIL; return; }
    gtk_flow_box_insert(GTK_FLOW_BOX(flow), child, -1);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void gui_frame(SnaskValue* out) {
    GtkWidget* f = gtk_frame_new(NULL);
    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(f);
    out->num = 0;
}

void gui_set_margin(SnaskValue* out, SnaskValue* widget_h, SnaskValue* margin_v) {
    if ((int)widget_h->tag != SNASK_STR || (int)margin_v->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w) { out->tag = (double)SNASK_NIL; return; }
    int m = (int)margin_v->num;
    gtk_widget_set_margin_start(w, m);
    gtk_widget_set_margin_end(w, m);
    gtk_widget_set_margin_top(w, m);
    gtk_widget_set_margin_bottom(w, m);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void gui_icon(SnaskValue* out, SnaskValue* name, SnaskValue* size_v) {
    if ((int)name->tag != SNASK_STR || (int)size_v->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* img = gtk_image_new_from_icon_name((const char*)name->ptr, GTK_ICON_SIZE_DIALOG);
    if (GTK_IS_IMAGE(img)) gtk_image_set_pixel_size(GTK_IMAGE(img), (int)size_v->num);
    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(img);
    out->num = 0;
}

void gui_css(SnaskValue* out, SnaskValue* css) {
    if ((int)css->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkCssProvider* provider = gtk_css_provider_new();
    gtk_css_provider_load_from_data(provider, (const char*)css->ptr, -1, NULL);
    GdkScreen* screen = gdk_screen_get_default();
    if (screen) {
        gtk_style_context_add_provider_for_screen(
            screen,
            GTK_STYLE_PROVIDER(provider),
            GTK_STYLE_PROVIDER_PRIORITY_USER
        );
    }
    g_object_unref(provider);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void gui_add_class(SnaskValue* out, SnaskValue* widget_h, SnaskValue* cls) {
    if ((int)widget_h->tag != SNASK_STR || (int)cls->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w) { out->tag = (double)SNASK_NIL; return; }
    GtkStyleContext* sc = gtk_widget_get_style_context(w);
    if (sc) gtk_style_context_add_class(sc, (const char*)cls->ptr);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

static gboolean gui_on_tap_cb(GtkWidget* _widget, GdkEventButton* _ev, gpointer user_data) {
    (void)_widget;
    (void)_ev;
    GuiCallbackCtx* ctx = (GuiCallbackCtx*)user_data;
    if (!ctx) return FALSE;
    if (ctx->ctx) (void)gui_call_handler_2(ctx->handler_name, ctx->widget_handle, ctx->ctx);
    else (void)gui_call_handler_1(ctx->handler_name, ctx->widget_handle);
    return FALSE;
}

void gui_on_tap_ctx(SnaskValue* out, SnaskValue* widget_h, SnaskValue* handler_name, SnaskValue* ctx_str) {
    if ((int)widget_h->tag != SNASK_STR || (int)handler_name->tag != SNASK_STR || (int)ctx_str->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w) { out->tag = (double)SNASK_NIL; return; }
    gtk_widget_add_events(w, GDK_BUTTON_PRESS_MASK);
    GuiCallbackCtx* ctx = (GuiCallbackCtx*)calloc(1, sizeof(GuiCallbackCtx));
    ctx->handler_name = strdup((const char*)handler_name->ptr);
    ctx->widget_handle = strdup((const char*)widget_h->ptr);
    ctx->ctx = strdup((const char*)ctx_str->ptr);
    g_signal_connect_data(w, "button-press-event", G_CALLBACK(gui_on_tap_cb), ctx, (GClosureNotify)gui_free_ctx, 0);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
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
    // GtkWindow (GtkBin) can only contain one child.
    if (GTK_IS_BIN(parent)) {
        GtkWidget* old = gtk_bin_get_child(GTK_BIN(parent));
        if (old) {
            gtk_container_remove(GTK_CONTAINER(parent), old);
        }
    }
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
    if (GTK_IS_BOX(box)) {
        gtk_box_pack_start(GTK_BOX(box), child, FALSE, FALSE, 0);
    } else if (GTK_IS_CONTAINER(box)) {
        gtk_container_add(GTK_CONTAINER(box), child);
    } else {
        out->tag = (double)SNASK_NIL;
        return;
    }
    out->tag = (double)SNASK_BOOL;
    out->num = 1.0;
    out->ptr = NULL;
}

void gui_add_expand(SnaskValue* out, SnaskValue* box_h, SnaskValue* child_h) {
    if ((int)box_h->tag != SNASK_STR || (int)child_h->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    GtkWidget* box = (GtkWidget*)gui_handle_to_ptr((const char*)box_h->ptr);
    GtkWidget* child = (GtkWidget*)gui_handle_to_ptr((const char*)child_h->ptr);
    if (!box || !child) { out->tag = (double)SNASK_NIL; return; }
    if (GTK_IS_BOX(box)) {
        gtk_box_pack_start(GTK_BOX(box), child, TRUE, TRUE, 0);
    } else if (GTK_IS_CONTAINER(box)) {
        gtk_container_add(GTK_CONTAINER(box), child);
    } else {
        out->tag = (double)SNASK_NIL;
        return;
    }
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

void gui_textview(SnaskValue* out) {
    GtkWidget* tv = gtk_text_view_new();
    out->tag = (double)SNASK_STR;
    out->ptr = gui_ptr_to_handle(tv);
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
    else if (GTK_IS_TEXT_VIEW(w)) {
        GtkTextBuffer* buf = gtk_text_view_get_buffer(GTK_TEXT_VIEW(w));
        gtk_text_buffer_set_text(buf, (const char*)text->ptr, -1);
    }
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
    if (GTK_IS_TEXT_VIEW(w)) {
        GtkTextBuffer* buf = gtk_text_view_get_buffer(GTK_TEXT_VIEW(w));
        GtkTextIter start, end;
        gtk_text_buffer_get_bounds(buf, &start, &end);
        char* t = gtk_text_buffer_get_text(buf, &start, &end, TRUE);
        out->tag = (double)SNASK_STR;
        out->ptr = snask_gc_strdup(t ? t : "");
        out->num = 0;
        if (t) g_free(t);
        return;
    }
    out->tag = (double)SNASK_STR;
    out->ptr = snask_gc_strdup("");
    out->num = 0;
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

// --- Snask_Skia (experimental) ---
// Default backend is Cairo (human-friendly, always available with GTK3).
// If the runtime is built with SNASK_SKIA, apps can opt into real Skia by setting:
//   USE_SKIA = 1
// (Snask will automatically call `skia_use_real(true)` before main::start.)
//
// Handles are strings:
// - Cairo: "skia_surface:cairo:<id>"
// - Skia:  "skia_surface:skia:<id>"

typedef struct {
    int w;
    int h;
    double r, g, b, a;
    cairo_surface_t* surface;
    cairo_t* cr;
} SnaskSkiaSurface;

// Need Cairo types even when SNASK_SKIA is enabled (default backend is Cairo).
// Cairo is available when GTK3 headers are enabled.
#ifdef SNASK_GUI_GTK
#include <cairo.h>
#endif

static SnaskSkiaSurface** skia_surfaces = NULL;
static size_t skia_surfaces_len = 0;
static size_t skia_surfaces_cap = 0;

static void skia_track_surface(SnaskSkiaSurface* s) {
    if (!s) return;
    if (skia_surfaces_len == skia_surfaces_cap) {
        size_t nc = skia_surfaces_cap ? skia_surfaces_cap * 2 : 64;
        SnaskSkiaSurface** n = (SnaskSkiaSurface**)realloc(skia_surfaces, nc * sizeof(SnaskSkiaSurface*));
        if (!n) return;
        skia_surfaces = n;
        skia_surfaces_cap = nc;
    }
    skia_surfaces[skia_surfaces_len++] = s;
}

static SnaskSkiaSurface* skia_get_surface(const char* handle) {
    if (!handle) return NULL;
    const char* pfx = "skia_surface:cairo:";
    size_t pfx_len = strlen(pfx);
    if (strncmp(handle, pfx, pfx_len) != 0) return NULL;
    long id = strtol(handle + pfx_len, NULL, 10);
    if (id < 0) return NULL;
    size_t idx = (size_t)id;
    if (idx >= skia_surfaces_len) return NULL;
    return skia_surfaces[idx];
}

#ifdef SNASK_SKIA
static int snask_skia_default_backend = 0; // 0=cairo, 1=skia

void skia_use_real(SnaskValue* out, SnaskValue* enabled) {
    if ((int)enabled->tag != SNASK_BOOL) { out->tag = (double)SNASK_NIL; return; }
    snask_skia_default_backend = (enabled->num != 0.0) ? 1 : 0;
    out->tag = (double)SNASK_BOOL;
    out->num = 1.0;
    out->ptr = NULL;
}

void skia_version(SnaskValue* out) {
    out->tag = (double)SNASK_STR;
    out->ptr = snask_gc_strdup(snask_skia_impl_version());
    out->num = 0;
}
#else
void skia_version(SnaskValue* out) {
    out->tag = (double)SNASK_STR;
    out->ptr = snask_gc_strdup("cairo-backend");
    out->num = 0;
}
void skia_use_real(SnaskValue* out, SnaskValue* _b) { (void)_b; out->tag = (double)SNASK_BOOL; out->num = 0.0; out->ptr = NULL; }
#endif

void skia_surface(SnaskValue* out, SnaskValue* wv, SnaskValue* hv) {
    if ((int)wv->tag != SNASK_NUM || (int)hv->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; return; }
    int w = (int)wv->num;
    int h = (int)hv->num;
    if (w <= 0 || h <= 0 || w > 16384 || h > 16384) { out->tag = (double)SNASK_NIL; return; }

#ifdef SNASK_SKIA
    if (snask_skia_default_backend == 1) {
        int id = snask_skia_impl_surface_create(w, h);
        if (id < 0) { out->tag = (double)SNASK_NIL; return; }
        char buf[64];
        snprintf(buf, sizeof(buf), "skia_surface:skia:%d", id);
        out->tag = (double)SNASK_STR;
        out->ptr = snask_gc_strdup(buf);
        out->num = 0;
        return;
    }
#endif

    SnaskSkiaSurface* s = (SnaskSkiaSurface*)calloc(1, sizeof(SnaskSkiaSurface));
    if (!s) { out->tag = (double)SNASK_NIL; return; }
    s->w = w; s->h = h;
    s->r = 1.0; s->g = 1.0; s->b = 1.0; s->a = 1.0;
    s->surface = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, w, h);
    s->cr = cairo_create(s->surface);
    if (!s->surface || !s->cr) { out->tag = (double)SNASK_NIL; return; }

    // Default: clear transparent
    cairo_set_source_rgba(s->cr, 0, 0, 0, 0);
    cairo_set_operator(s->cr, CAIRO_OPERATOR_SOURCE);
    cairo_paint(s->cr);
    cairo_set_operator(s->cr, CAIRO_OPERATOR_OVER);

    skia_track_surface(s);
    char buf[64];
    snprintf(buf, sizeof(buf), "skia_surface:cairo:%zu", skia_surfaces_len - 1);
    out->tag = (double)SNASK_STR;
    out->ptr = snask_gc_strdup(buf);
    out->num = 0;
}

static int skia_parse_handle(const char* handle, bool* is_skia) {
    if (is_skia) *is_skia = false;
    if (!handle) return -1;
    const char* pfx_skia = "skia_surface:skia:";
    const char* pfx_cairo = "skia_surface:cairo:";
    if (strncmp(handle, pfx_skia, strlen(pfx_skia)) == 0) {
        if (is_skia) *is_skia = true;
        return (int)strtol(handle + strlen(pfx_skia), NULL, 10);
    }
    if (strncmp(handle, pfx_cairo, strlen(pfx_cairo)) == 0) {
        if (is_skia) *is_skia = false;
        return (int)strtol(handle + strlen(pfx_cairo), NULL, 10);
    }
    return -1;
}

void skia_surface_width(SnaskValue* out, SnaskValue* surface_h) {
    if ((int)surface_h->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    bool is_skia = false;
    int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) {
        int w = snask_skia_impl_surface_width(id);
        if (w < 0) { out->tag = (double)SNASK_NIL; return; }
        out->tag = (double)SNASK_NUM; out->num = (double)w; out->ptr = NULL;
        return;
    }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    if (!s) { out->tag = (double)SNASK_NIL; return; }
    out->tag = (double)SNASK_NUM; out->num = (double)s->w; out->ptr = NULL;
}

void skia_surface_height(SnaskValue* out, SnaskValue* surface_h) {
    if ((int)surface_h->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    bool is_skia = false;
    int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) {
        int h = snask_skia_impl_surface_height(id);
        if (h < 0) { out->tag = (double)SNASK_NIL; return; }
        out->tag = (double)SNASK_NUM; out->num = (double)h; out->ptr = NULL;
        return;
    }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    if (!s) { out->tag = (double)SNASK_NIL; return; }
    out->tag = (double)SNASK_NUM; out->num = (double)s->h; out->ptr = NULL;
}

void skia_surface_clear(SnaskValue* out, SnaskValue* surface_h, SnaskValue* rv, SnaskValue* gv, SnaskValue* bv, SnaskValue* av) {
    if ((int)surface_h->tag != SNASK_STR || (int)rv->tag != SNASK_NUM || (int)gv->tag != SNASK_NUM || (int)bv->tag != SNASK_NUM || (int)av->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; return; }
    bool is_skia = false;
    int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) {
        bool ok = snask_skia_impl_surface_clear(id, rv->num, gv->num, bv->num, av->num);
        out->tag = (double)SNASK_BOOL; out->num = ok ? 1.0 : 0.0; out->ptr = NULL;
        return;
    }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    if (!s || !s->cr) { out->tag = (double)SNASK_NIL; return; }
    cairo_save(s->cr);
    cairo_set_source_rgba(s->cr, rv->num, gv->num, bv->num, av->num);
    cairo_set_operator(s->cr, CAIRO_OPERATOR_SOURCE);
    cairo_paint(s->cr);
    cairo_restore(s->cr);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void skia_surface_set_color(SnaskValue* out, SnaskValue* surface_h, SnaskValue* rv, SnaskValue* gv, SnaskValue* bv, SnaskValue* av) {
    if ((int)surface_h->tag != SNASK_STR || (int)rv->tag != SNASK_NUM || (int)gv->tag != SNASK_NUM || (int)bv->tag != SNASK_NUM || (int)av->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; return; }
    bool is_skia = false;
    int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) {
        bool ok = snask_skia_impl_surface_set_color(id, rv->num, gv->num, bv->num, av->num);
        out->tag = (double)SNASK_BOOL; out->num = ok ? 1.0 : 0.0; out->ptr = NULL;
        return;
    }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    if (!s) { out->tag = (double)SNASK_NIL; return; }
    s->r = rv->num; s->g = gv->num; s->b = bv->num; s->a = av->num;
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void skia_draw_rect(SnaskValue* out, SnaskValue* surface_h, SnaskValue* xv, SnaskValue* yv, SnaskValue* wv, SnaskValue* hv, SnaskValue* fillv) {
    if ((int)surface_h->tag != SNASK_STR || (int)xv->tag != SNASK_NUM || (int)yv->tag != SNASK_NUM || (int)wv->tag != SNASK_NUM || (int)hv->tag != SNASK_NUM || (int)fillv->tag != SNASK_BOOL) { out->tag = (double)SNASK_NIL; return; }
    bool is_skia = false;
    int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) {
        bool ok = snask_skia_impl_draw_rect(id, xv->num, yv->num, wv->num, hv->num, fillv->num != 0.0);
        out->tag = (double)SNASK_BOOL; out->num = ok ? 1.0 : 0.0; out->ptr = NULL;
        return;
    }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    if (!s || !s->cr) { out->tag = (double)SNASK_NIL; return; }
    cairo_set_source_rgba(s->cr, s->r, s->g, s->b, s->a);
    cairo_rectangle(s->cr, xv->num, yv->num, wv->num, hv->num);
    if (fillv->num != 0.0) cairo_fill(s->cr); else cairo_stroke(s->cr);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void skia_draw_circle(SnaskValue* out, SnaskValue* surface_h, SnaskValue* cxv, SnaskValue* cyv, SnaskValue* rv, SnaskValue* fillv) {
    if ((int)surface_h->tag != SNASK_STR || (int)cxv->tag != SNASK_NUM || (int)cyv->tag != SNASK_NUM || (int)rv->tag != SNASK_NUM || (int)fillv->tag != SNASK_BOOL) { out->tag = (double)SNASK_NIL; return; }
    bool is_skia = false;
    int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) {
        bool ok = snask_skia_impl_draw_circle(id, cxv->num, cyv->num, rv->num, fillv->num != 0.0);
        out->tag = (double)SNASK_BOOL; out->num = ok ? 1.0 : 0.0; out->ptr = NULL;
        return;
    }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    if (!s || !s->cr) { out->tag = (double)SNASK_NIL; return; }
    cairo_set_source_rgba(s->cr, s->r, s->g, s->b, s->a);
    cairo_arc(s->cr, cxv->num, cyv->num, rv->num, 0.0, 2.0 * M_PI);
    if (fillv->num != 0.0) cairo_fill(s->cr); else cairo_stroke(s->cr);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void skia_draw_line(SnaskValue* out, SnaskValue* surface_h, SnaskValue* x1v, SnaskValue* y1v, SnaskValue* x2v, SnaskValue* y2v, SnaskValue* stroke_wv) {
    if ((int)surface_h->tag != SNASK_STR || (int)x1v->tag != SNASK_NUM || (int)y1v->tag != SNASK_NUM || (int)x2v->tag != SNASK_NUM || (int)y2v->tag != SNASK_NUM || (int)stroke_wv->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; return; }
    bool is_skia = false;
    int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) {
        bool ok = snask_skia_impl_draw_line(id, x1v->num, y1v->num, x2v->num, y2v->num, stroke_wv->num);
        out->tag = (double)SNASK_BOOL; out->num = ok ? 1.0 : 0.0; out->ptr = NULL;
        return;
    }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    if (!s || !s->cr) { out->tag = (double)SNASK_NIL; return; }
    cairo_set_source_rgba(s->cr, s->r, s->g, s->b, s->a);
    cairo_set_line_width(s->cr, stroke_wv->num <= 0 ? 1.0 : stroke_wv->num);
    cairo_move_to(s->cr, x1v->num, y1v->num);
    cairo_line_to(s->cr, x2v->num, y2v->num);
    cairo_stroke(s->cr);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void skia_draw_text(SnaskValue* out, SnaskValue* surface_h, SnaskValue* xv, SnaskValue* yv, SnaskValue* textv, SnaskValue* sizev) {
    if ((int)surface_h->tag != SNASK_STR || (int)xv->tag != SNASK_NUM || (int)yv->tag != SNASK_NUM || (int)textv->tag != SNASK_STR || (int)sizev->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; return; }
    bool is_skia = false;
    int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) {
        bool ok = snask_skia_impl_draw_text(id, xv->num, yv->num, (const char*)textv->ptr, sizev->num);
        out->tag = (double)SNASK_BOOL; out->num = ok ? 1.0 : 0.0; out->ptr = NULL;
        return;
    }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    if (!s || !s->cr) { out->tag = (double)SNASK_NIL; return; }
    cairo_set_source_rgba(s->cr, s->r, s->g, s->b, s->a);
    cairo_select_font_face(s->cr, "Sans", CAIRO_FONT_SLANT_NORMAL, CAIRO_FONT_WEIGHT_NORMAL);
    cairo_set_font_size(s->cr, sizev->num <= 0 ? 14.0 : sizev->num);
    cairo_move_to(s->cr, xv->num, yv->num);
    cairo_show_text(s->cr, (const char*)textv->ptr);
    out->tag = (double)SNASK_BOOL; out->num = 1.0; out->ptr = NULL;
}

void skia_save_png(SnaskValue* out, SnaskValue* surface_h, SnaskValue* pathv) {
    if ((int)surface_h->tag != SNASK_STR || (int)pathv->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    bool is_skia = false;
    int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) {
        bool ok = snask_skia_impl_save_png(id, (const char*)pathv->ptr);
        out->tag = (double)SNASK_BOOL; out->num = ok ? 1.0 : 0.0; out->ptr = NULL;
        return;
    }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    if (!s || !s->surface) { out->tag = (double)SNASK_NIL; return; }
    cairo_status_t st = cairo_surface_write_to_png(s->surface, (const char*)pathv->ptr);
    out->tag = (double)SNASK_BOOL;
    out->num = (st == CAIRO_STATUS_SUCCESS) ? 1.0 : 0.0;
    out->ptr = NULL;
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
void gui_eventbox(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_scrolled(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_flowbox(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_flow_add(SnaskValue* out, SnaskValue* _f, SnaskValue* _c) { (void)_f; (void)_c; out->tag = (double)SNASK_NIL; }
void gui_frame(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_set_margin(SnaskValue* out, SnaskValue* _w, SnaskValue* _m) { (void)_w; (void)_m; out->tag = (double)SNASK_NIL; }
void gui_icon(SnaskValue* out, SnaskValue* _n, SnaskValue* _s) { (void)_n; (void)_s; out->tag = (double)SNASK_NIL; }
void gui_css(SnaskValue* out, SnaskValue* _c) { (void)_c; out->tag = (double)SNASK_NIL; }
void gui_add_class(SnaskValue* out, SnaskValue* _w, SnaskValue* _c) { (void)_w; (void)_c; out->tag = (double)SNASK_NIL; }
void gui_listbox(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_list_add_text(SnaskValue* out, SnaskValue* _l, SnaskValue* _t) { (void)_l; (void)_t; out->tag = (double)SNASK_NIL; }
void gui_on_select_ctx(SnaskValue* out, SnaskValue* _l, SnaskValue* _h, SnaskValue* _c) { (void)_l; (void)_h; (void)_c; out->tag = (double)SNASK_NIL; }
void gui_set_child(SnaskValue* out, SnaskValue* _p, SnaskValue* _c) { (void)_p; (void)_c; out->tag = (double)SNASK_NIL; }
void gui_add(SnaskValue* out, SnaskValue* _b, SnaskValue* _c) { (void)_b; (void)_c; out->tag = (double)SNASK_NIL; }
void gui_add_expand(SnaskValue* out, SnaskValue* _b, SnaskValue* _c) { (void)_b; (void)_c; out->tag = (double)SNASK_NIL; }
void gui_label(SnaskValue* out, SnaskValue* _t) { (void)_t; out->tag = (double)SNASK_NIL; }
void gui_entry(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_textview(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
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
void gui_on_tap_ctx(SnaskValue* out, SnaskValue* _w, SnaskValue* _h, SnaskValue* _c) { (void)_w; (void)_h; (void)_c; out->tag = (double)SNASK_NIL; }
void gui_separator_h(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_separator_v(SnaskValue* out) { out->tag = (double)SNASK_NIL; }
void gui_msg_info(SnaskValue* out, SnaskValue* _t, SnaskValue* _m) { (void)_t; (void)_m; out->tag = (double)SNASK_NIL; }
void gui_msg_error(SnaskValue* out, SnaskValue* _t, SnaskValue* _m) { (void)_t; (void)_m; out->tag = (double)SNASK_NIL; }

void skia_version(SnaskValue* out) { out->tag = (double)SNASK_STR; out->ptr = snask_gc_strdup("stub"); out->num = 0; }
void skia_surface(SnaskValue* out, SnaskValue* _w, SnaskValue* _h) { (void)_w; (void)_h; out->tag = (double)SNASK_NIL; }
void skia_surface_width(SnaskValue* out, SnaskValue* _s) { (void)_s; out->tag = (double)SNASK_NIL; }
void skia_surface_height(SnaskValue* out, SnaskValue* _s) { (void)_s; out->tag = (double)SNASK_NIL; }
void skia_surface_clear(SnaskValue* out, SnaskValue* _s, SnaskValue* _r, SnaskValue* _g, SnaskValue* _b, SnaskValue* _a) { (void)_s; (void)_r; (void)_g; (void)_b; (void)_a; out->tag = (double)SNASK_NIL; }
void skia_surface_set_color(SnaskValue* out, SnaskValue* _s, SnaskValue* _r, SnaskValue* _g, SnaskValue* _b, SnaskValue* _a) { (void)_s; (void)_r; (void)_g; (void)_b; (void)_a; out->tag = (double)SNASK_NIL; }
void skia_draw_rect(SnaskValue* out, SnaskValue* _s, SnaskValue* _x, SnaskValue* _y, SnaskValue* _w, SnaskValue* _h, SnaskValue* _f) { (void)_s; (void)_x; (void)_y; (void)_w; (void)_h; (void)_f; out->tag = (double)SNASK_NIL; }
void skia_draw_circle(SnaskValue* out, SnaskValue* _s, SnaskValue* _cx, SnaskValue* _cy, SnaskValue* _r, SnaskValue* _f) { (void)_s; (void)_cx; (void)_cy; (void)_r; (void)_f; out->tag = (double)SNASK_NIL; }
void skia_draw_line(SnaskValue* out, SnaskValue* _s, SnaskValue* _x1, SnaskValue* _y1, SnaskValue* _x2, SnaskValue* _y2, SnaskValue* _sw) { (void)_s; (void)_x1; (void)_y1; (void)_x2; (void)_y2; (void)_sw; out->tag = (double)SNASK_NIL; }
void skia_draw_text(SnaskValue* out, SnaskValue* _s, SnaskValue* _x, SnaskValue* _y, SnaskValue* _t, SnaskValue* _sz) { (void)_s; (void)_x; (void)_y; (void)_t; (void)_sz; out->tag = (double)SNASK_NIL; }
void skia_save_png(SnaskValue* out, SnaskValue* _s, SnaskValue* _p) { (void)_s; (void)_p; out->tag = (double)SNASK_NIL; }

#endif

// ---------------- calc helpers ----------------
void mod(SnaskValue* out, SnaskValue* a, SnaskValue* b) {
    if (!a || !b || (int)a->tag != SNASK_NUM || (int)b->tag != SNASK_NUM) { out->tag = (double)SNASK_NIL; return; }
    if (b->num == 0.0) { out->tag = (double)SNASK_NIL; return; }
    out->tag = (double)SNASK_NUM;
    #ifdef SNASK_TINY
    out->num = snask_tiny_fmod(a->num, b->num);
    #else
    out->num = fmod(a->num, b->num);
    #endif
    out->ptr = NULL;
}

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

#ifndef SNASK_SQLITE
// ---------------- SQLite (MOCKS para Zenith) ----------------
void sqlite_open(SnaskValue* out, SnaskValue* path) {
    if (!path || (int)path->tag != SNASK_STR || !path->ptr) { out->tag = (double)SNASK_NIL; return; }
    out->tag = (double)SNASK_STR;
    out->ptr = snask_gc_strdup("mock-db-handle");
    out->num = 1.0; // Truthy for "if self.handle"
    printf("ℹ️ [ZENITH MOCK] Database opened: %s\n", (char*)path->ptr);
}
void sqlite_close(SnaskValue* out, SnaskValue* h) {
    out->tag = (double)SNASK_BOOL;
    out->num = 1.0;
    out->ptr = NULL;
}
void sqlite_exec(SnaskValue* out, SnaskValue* h, SnaskValue* sql) {
    out->tag = (double)SNASK_BOOL;
    out->num = 1.0;
    out->ptr = NULL;
    if (sql && sql->ptr) printf("ℹ️ [ZENITH MOCK] Executed SQL: %s\n", (char*)sql->ptr);
}
void sqlite_query(SnaskValue* out, SnaskValue* h, SnaskValue* sql) {
    // Retorna um objeto "vazio" (que no Snask funciona como lista vazia)
    out->tag = (double)SNASK_OBJ;
    out->ptr = obj_new(0);
    out->num = 0;
    if (sql && sql->ptr) printf("ℹ️ [ZENITH MOCK] Queried SQL: %s\n", (char*)sql->ptr);
}
#endif

