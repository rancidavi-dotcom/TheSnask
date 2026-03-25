#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <errno.h>
#include <dirent.h>
#include "rt_sfs.h"
#include "rt_gc.h"

void sfs_read(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    FILE *f = fopen((char*)path->ptr, "rb");
    if (!f) { *out = MAKE_NIL(); return; }
    fseek(f, 0, SEEK_END); long sz = ftell(f); fseek(f, 0, SEEK_SET);
    char *s = (char*)malloc(sz + 1); 
    if (!s) { fclose(f); *out = MAKE_NIL(); return; }
    fread(s, sz, 1, f); fclose(f); s[sz] = 0;
    snask_gc_track_ptr(s);
    *out = MAKE_STR(s);
}

void sfs_write(SnaskValue* out, SnaskValue* path, SnaskValue* content) {
    if ((int)path->tag != SNASK_STR || (int)content->tag != SNASK_STR) { *out = MAKE_BOOL(false); return; }
    FILE *f = fopen((char*)path->ptr, "w");
    if (!f) { *out = MAKE_BOOL(false); return; }
    fprintf(f, "%s", (char*)content->ptr);
    fflush(f);
    fclose(f);
    *out = MAKE_BOOL(true);
}

void sfs_append(SnaskValue* out, SnaskValue* path, SnaskValue* content) {
    if ((int)path->tag != SNASK_STR || (int)content->tag != SNASK_STR) { *out = MAKE_BOOL(false); return; }
    FILE *f = fopen((char*)path->ptr, "a");
    if (!f) { *out = MAKE_BOOL(false); return; }
    fprintf(f, "%s", (char*)content->ptr);
    fflush(f);
    fclose(f);
    *out = MAKE_BOOL(true);
}

void sfs_write_mb(SnaskValue* out, SnaskValue* path, SnaskValue* mb_val) {
    if ((int)path->tag != SNASK_STR || (int)mb_val->tag != SNASK_NUM) { *out = MAKE_NUM(0); return; }
    long mb = (long)mb_val->num;
    if (mb <= 0) { *out = MAKE_NUM(0); return; }

    const char* p = (const char*)path->ptr;
    int fd = open(p, O_WRONLY | O_CREAT | O_TRUNC | O_CLOEXEC, 0644);
    if (fd < 0) { *out = MAKE_NUM(0); return; }

    const size_t chunk = 8 * 1024 * 1024;
    char* buf = (char*)malloc(chunk);
    if (!buf) { close(fd); *out = MAKE_NUM(0); return; }
    memset(buf, 'a', chunk);

    unsigned long long total = 0;
    unsigned long long target = (unsigned long long)mb * 1024ULL * 1024ULL;
    while (total < target) {
        size_t want = chunk;
        unsigned long long left = target - total;
        if (left < want) want = (size_t)left;

        size_t off = 0;
        while (off < want) {
            ssize_t n = write(fd, buf + off, want - off);
            if (n < 0) {
                if (errno == EINTR) continue;
                free(buf); close(fd); *out = MAKE_NUM((double)total); return;
            }
            off += (size_t)n;
            total += (unsigned long long)n;
        }
    }
    close(fd); free(buf);
    *out = MAKE_NUM((double)total);
}

void sfs_count_bytes(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR) { *out = MAKE_NUM(0); return; }
    const char* p = (const char*)path->ptr;
    int fd = open(p, O_RDONLY | O_CLOEXEC);
    if (fd < 0) { *out = MAKE_NUM(0); return; }

    const size_t chunk = 8 * 1024 * 1024;
    char* buf = (char*)malloc(chunk);
    if (!buf) { close(fd); *out = MAKE_NUM(0); return; }

    unsigned long long total = 0;
    while (1) {
        ssize_t n = read(fd, buf, chunk);
        if (n < 0) { if (errno == EINTR) continue; break; }
        if (n == 0) break;
        total += (unsigned long long)n;
    }
    close(fd); free(buf);
    *out = MAKE_NUM((double)total);
}

static bool sfs_copy_file_impl(const char* src, const char* dst) {
    FILE* in = fopen(src, "rb"); if (!in) return false;
    FILE* out = fopen(dst, "wb"); if (!out) { fclose(in); return false; }
    char buf[8192]; size_t n = 0;
    while ((n = fread(buf, 1, sizeof(buf), in)) > 0) {
        if (fwrite(buf, 1, n, out) != n) { fclose(in); fclose(out); return false; }
    }
    fclose(in); fflush(out); fclose(out);
    return true;
}

void sfs_copy(SnaskValue* out, SnaskValue* src, SnaskValue* dst) {
    if ((int)src->tag != SNASK_STR || (int)dst->tag != SNASK_STR) { *out = MAKE_BOOL(false); return; }
    *out = MAKE_BOOL(sfs_copy_file_impl((const char*)src->ptr, (const char*)dst->ptr));
}

void sfs_move(SnaskValue* out, SnaskValue* src, SnaskValue* dst) {
    if ((int)src->tag != SNASK_STR || (int)dst->tag != SNASK_STR) { *out = MAKE_BOOL(false); return; }
    if (rename((const char*)src->ptr, (const char*)dst->ptr) == 0) { *out = MAKE_BOOL(true); return; }
    if (sfs_copy_file_impl((const char*)src->ptr, (const char*)dst->ptr)) {
        remove((const char*)src->ptr);
        *out = MAKE_BOOL(true);
    } else {
        *out = MAKE_BOOL(false);
    }
}

void sfs_mkdir(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR) { *out = MAKE_BOOL(false); return; }
    if (mkdir((const char*)path->ptr, 0755) == 0 || errno == EEXIST) { *out = MAKE_BOOL(true); }
    else { *out = MAKE_BOOL(false); }
}

void sfs_rmdir(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR) { *out = MAKE_BOOL(false); return; }
    *out = MAKE_BOOL(rmdir((const char*)path->ptr) == 0);
}

void sfs_is_file(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR) { *out = MAKE_BOOL(false); return; }
    struct stat st;
    if (stat((const char*)path->ptr, &st) != 0) { *out = MAKE_BOOL(false); return; }
    *out = MAKE_BOOL(S_ISREG(st.st_mode));
}

void sfs_is_dir(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR) { *out = MAKE_BOOL(false); return; }
    struct stat st;
    if (stat((const char*)path->ptr, &st) != 0) { *out = MAKE_BOOL(false); return; }
    *out = MAKE_BOOL(S_ISDIR(st.st_mode));
}

void sfs_exists(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR) { *out = MAKE_BOOL(false); return; }
    struct stat st;
    *out = MAKE_BOOL(stat((const char*)path->ptr, &st) == 0);
}

void sfs_delete(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR) { *out = MAKE_BOOL(false); return; }
    *out = MAKE_BOOL(remove((const char*)path->ptr) == 0);
}

void sfs_size(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR) { *out = MAKE_NUM(0); return; }
    struct stat st;
    if (stat((const char*)path->ptr, &st) != 0) { *out = MAKE_NUM(0); return; }
    *out = MAKE_NUM((double)st.st_size);
}

void sfs_mtime(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR) { *out = MAKE_NUM(0); return; }
    struct stat st;
    if (stat((const char*)path->ptr, &st) != 0) { *out = MAKE_NUM(0); return; }
    *out = MAKE_NUM((double)st.st_mtime);
}

void sfs_listdir(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    DIR* d = opendir((const char*)path->ptr);
    if (!d) { *out = MAKE_NIL(); return; }

    SnaskObject* arr = (SnaskObject*)malloc(sizeof(SnaskObject));
    arr->count = 0; arr->names = NULL; arr->values = NULL;
    int cap = 0;

    struct dirent* ent;
    while ((ent = readdir(d)) != NULL) {
        const char* name = ent->d_name;
        if (strcmp(name, ".") == 0 || strcmp(name, "..") == 0) continue;
        if (arr->count >= cap) {
            int new_cap = (cap == 0) ? 16 : cap * 2;
            arr->names = (char**)realloc(arr->names, (size_t)new_cap * sizeof(char*));
            arr->values = (SnaskValue*)realloc(arr->values, (size_t)new_cap * sizeof(SnaskValue));
            for (int i = cap; i < new_cap; i++) { 
                arr->names[i] = NULL; 
                arr->values[i] = MAKE_NIL(); 
            }
            cap = new_cap;
        }
        char idx_name[32];
        snprintf(idx_name, sizeof(idx_name), "%d", arr->count);
        arr->names[arr->count] = snask_gc_strdup(idx_name);
        arr->values[arr->count] = MAKE_STR(snask_gc_strdup(name));
        arr->count++;
    }
    closedir(d);

    *out = MAKE_OBJ(arr);
}

void sfs_bench_create_small_files(SnaskValue* out, SnaskValue* dir, SnaskValue* n_files, SnaskValue* size_bytes) {
    if ((int)dir->tag != SNASK_STR || (int)n_files->tag != SNASK_NUM || (int)size_bytes->tag != SNASK_NUM) { *out = MAKE_NUM(0); return; }
    const char* base = (const char*)dir->ptr;
    int n = (int)n_files->num;
    int sz = (int)size_bytes->num;
    if (!base || n <= 0 || sz <= 0) { *out = MAKE_NUM(0); return; }

    char* buf = (char*)malloc((size_t)sz);
    if (!buf) { *out = MAKE_NUM(0); return; }
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
    *out = MAKE_NUM((double)created);
}

void sfs_bench_count_entries(SnaskValue* out, SnaskValue* dir) {
    if ((int)dir->tag != SNASK_STR || !dir->ptr) { *out = MAKE_NUM(0); return; }
    DIR* d = opendir((const char*)dir->ptr);
    if (!d) { *out = MAKE_NUM(0); return; }
    int count = 0;
    struct dirent* ent;
    while ((ent = readdir(d)) != NULL) {
        if (strcmp(ent->d_name, ".") == 0 || strcmp(ent->d_name, "..") == 0) continue;
        count++;
    }
    closedir(d);
    *out = MAKE_NUM((double)count);
}

void sfs_bench_delete_small_files(SnaskValue* out, SnaskValue* dir, SnaskValue* n_files) {
    if ((int)dir->tag != SNASK_STR || (int)n_files->tag != SNASK_NUM) { *out = MAKE_NUM(0); return; }
    const char* base = (const char*)dir->ptr;
    int n = (int)n_files->num;
    if (!base || n <= 0) { *out = MAKE_NUM(0); return; }

    int deleted = 0;
    for (int i = 0; i < n; i++) {
        char p[4096];
        int plen = snprintf(p, sizeof(p), "%s/f_%d.bin", base, i);
        if (plen <= 0 || plen >= (int)sizeof(p)) continue;
        if (remove(p) == 0) deleted++;
    }
    *out = MAKE_NUM((double)deleted);
}
