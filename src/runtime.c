#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <unistd.h>
#include <time.h>
#include <math.h>
#include <ctype.h>

typedef enum { SNASK_NIL, SNASK_NUM, SNASK_BOOL, SNASK_STR } SnaskType;

typedef struct {
    double tag;
    double num;
    char* str;
} SnaskValue;

// --- AUXILIAR HTTP ---
void http_request(SnaskValue* out, const char* method, SnaskValue* url, SnaskValue* data) {
    if ((int)url->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    
    char cmd[4096];
    if (data && (int)data->tag == SNASK_STR) {
        // Métodos com corpo (POST, PUT, PATCH)
        snprintf(cmd, sizeof(cmd), "curl -s -L -X %s -d '%s' '%s'", method, data->str, url->str);
    } else {
        // Métodos sem corpo (GET, DELETE)
        snprintf(cmd, sizeof(cmd), "curl -s -L -X %s '%s'", method, url->str);
    }
    
    FILE *fp = popen(cmd, "r");
    if (!fp) { out->tag = (double)SNASK_NIL; return; }
    
    char *response = malloc(1);
    response[0] = '\0';
    char buffer[1024];
    size_t total_len = 0;
    
    while (fgets(buffer, sizeof(buffer), fp) != NULL) {
        size_t chunk_len = strlen(buffer);
        response = realloc(response, total_len + chunk_len + 1);
        strcpy(response + total_len, buffer);
        total_len += chunk_len;
    }
    
    pclose(fp);
    out->tag = (double)SNASK_STR;
    out->str = response;
    out->num = 0;
}

// --- FUNÇÕES EXPORTADAS HTTP ---
void s_http_get(SnaskValue* out, SnaskValue* url) { http_request(out, "GET", url, NULL); }
void s_http_post(SnaskValue* out, SnaskValue* url, SnaskValue* data) { http_request(out, "POST", url, data); }
void s_http_put(SnaskValue* out, SnaskValue* url, SnaskValue* data) { http_request(out, "PUT", url, data); }
void s_http_delete(SnaskValue* out, SnaskValue* url) { http_request(out, "DELETE", url, NULL); }
void s_http_patch(SnaskValue* out, SnaskValue* url, SnaskValue* data) { http_request(out, "PATCH", url, data); }

// --- IMPRESSÃO ---
void s_print(SnaskValue* v) {
    int tag = (int)v->tag;
    if (tag == SNASK_NUM) printf("%g ", v->num);
    else if (tag == SNASK_STR) printf("%s ", v->str);
    else if (tag == SNASK_BOOL) printf("%s ", v->num ? "true" : "false");
    else printf("nil ");
}

void s_println() { printf("\n"); fflush(stdout); }

// --- FILE SYSTEM (SFS) ---
void sfs_read(SnaskValue* out, SnaskValue* path) {
    if ((int)path->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    FILE *f = fopen(path->str, "rb");
    if (!f) { out->tag = (double)SNASK_NIL; return; }
    fseek(f, 0, SEEK_END); long sz = ftell(f); fseek(f, 0, SEEK_SET);
    char *s = malloc(sz + 1); fread(s, sz, 1, f); fclose(f); s[sz] = 0;
    out->tag = (double)SNASK_STR; out->str = s; out->num = 0;
}

void sfs_write(SnaskValue* out, SnaskValue* path, SnaskValue* content) {
    out->tag = (double)SNASK_BOOL;
    if ((int)path->tag != SNASK_STR || (int)content->tag != SNASK_STR) { out->num = 0; return; }
    FILE *f = fopen(path->str, "w");
    if (!f) { out->num = 0; return; }
    fprintf(f, "%s", content->str);
    fflush(f);
    fclose(f);
    out->num = 1;
    out->str = NULL;
}

void sfs_delete(SnaskValue* out, SnaskValue* path) { 
    if ((int)path->tag != SNASK_STR) { out->tag = (double)SNASK_BOOL; out->num = 0; return; }
    int res = remove(path->str); 
    out->tag = (double)SNASK_BOOL; out->num = (res == 0); out->str = NULL;
}

void sfs_exists(SnaskValue* out, SnaskValue* path) { 
    if ((int)path->tag != SNASK_STR) { out->tag = (double)SNASK_BOOL; out->num = 0; return; }
    int res = access(path->str, F_OK); 
    out->tag = (double)SNASK_BOOL; out->num = (res == 0); out->str = NULL;
}

// --- UTILS NATIVAS ---
void s_abs(SnaskValue* out, SnaskValue* n) { *out = (SnaskValue){(double)SNASK_NUM, fabs(n->num), NULL}; }
void s_max(SnaskValue* out, SnaskValue* a, SnaskValue* b) { *out = (SnaskValue){(double)SNASK_NUM, fmax(a->num, b->num), NULL}; }
void s_min(SnaskValue* out, SnaskValue* a, SnaskValue* b) { *out = (SnaskValue){(double)SNASK_NUM, fmin(a->num, b->num), NULL}; }

void s_len(SnaskValue* out, SnaskValue* s) { 
    if ((int)s->tag != SNASK_STR) { out->tag = (double)SNASK_NUM; out->num = 0; return; }
    out->tag = (double)SNASK_NUM; out->num = (double)strlen(s->str); 
}

void s_upper(SnaskValue* out, SnaskValue* s) {
    if ((int)s->tag != SNASK_STR) { *out = *s; return; }
    char* new_s = strdup(s->str);
    for(int i = 0; new_s[i]; i++) new_s[i] = toupper(new_s[i]);
    out->tag = (double)SNASK_STR; out->str = new_s; out->num = 0;
}

void s_time(SnaskValue* out) { out->tag = (double)SNASK_NUM; out->num = (double)time(NULL); out->str = NULL; }
void s_sleep(SnaskValue* out, SnaskValue* ms) { usleep((unsigned int)(ms->num * 1000)); out->tag = (double)SNASK_NIL; }

void s_concat(SnaskValue* out, SnaskValue* s1, SnaskValue* s2) {
    if ((int)s1->tag != SNASK_STR || (int)s2->tag != SNASK_STR) { out->tag = (double)SNASK_NIL; return; }
    size_t len1 = strlen(s1->str); size_t len2 = strlen(s2->str);
    char* new_str = malloc(len1 + len2 + 1);
    strcpy(new_str, s1->str); strcat(new_str, s2->str);
    out->tag = (double)SNASK_STR; out->str = new_str; out->num = 0;
}
