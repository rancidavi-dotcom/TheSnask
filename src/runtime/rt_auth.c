#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <unistd.h>
#include <time.h>
#include <ctype.h>
#include <sys/utsname.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <fcntl.h>
#include <dlfcn.h>
#include <pthread.h>
#include "rt_auth.h"
#include "rt_gc.h"
#include "rt_base.h"

// --- Random helpers ---
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

// os_random_hex is now part of auth_random_hex implementation
void auth_random_hex(SnaskValue* out, SnaskValue* nbytes_val) {
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

    char* s = (char*)snask_gc_malloc((size_t)nbytes * 2 + 1);
    const char* hex = "0123456789abcdef"; // Declared hex constant here
    for (int i = 0; i < nbytes; i++) {
        s[i * 2] = hex[(buf[i] >> 4) & 0xF];
        s[i * 2 + 1] = hex[buf[i] & 0xF];
    }
    s[(size_t)nbytes * 2] = '\0';
    free(buf);
    out->ptr = s;
}

// --- Auth natives ---
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
    SnaskValue nbytes = {(double)SNASK_NUM, 16.0, NULL};
    auth_random_hex(&salt, &nbytes); // Use auth_random_hex which wraps os_random_hex
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
    SnaskValue nbytes = {(double)SNASK_NUM, 16.0, NULL};
    auth_random_hex(out, &nbytes);
}

void auth_csrf_token(SnaskValue* out) {
    SnaskValue nbytes = {(double)SNASK_NUM, 32.0, NULL};
    auth_random_hex(out, &nbytes);
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
