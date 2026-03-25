#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "rt_http.h"
#include "rt_gc.h"

static void http_request(SnaskValue* out, const char* method, SnaskValue* url, SnaskValue* data) {
    if ((int)url->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    
    const char* dbg = getenv("SNASK_HTTP_DEBUG");
    char cmd[4096];
    if (data && (int)data->tag == SNASK_STR) {
        snprintf(cmd, sizeof(cmd), "curl -f -sS -L --connect-timeout 10 --max-time 30 -X %s -d '%s' '%s' 2>&1", method, (char*)data->ptr, (char*)url->ptr);
    } else {
        snprintf(cmd, sizeof(cmd), "curl -f -sS -L --connect-timeout 10 --max-time 30 -X %s '%s' 2>&1", method, (char*)url->ptr);
    }

    if (dbg && *dbg) {
        FILE* df = fopen("/tmp/snask_http_debug.log", "a");
        if (df) { fprintf(df, "CMD=%s\n", cmd); fclose(df); }
    }
    
    FILE *fp = popen(cmd, "r");
    if (!fp) { *out = MAKE_NIL(); return; }
    
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
    if (rc != 0 && total_len == 0) { *out = MAKE_NIL(); return; }
    
    if (dbg && *dbg) {
        FILE* df = fopen("/tmp/snask_http_debug.log", "a");
        if (df) { fprintf(df, "RC=%d LEN=%zu\n", rc, total_len); fclose(df); }
    }
    
    *out = MAKE_STR(response);
}

void s_http_get(SnaskValue* out, SnaskValue* url) { http_request(out, "GET", url, NULL); }
void s_http_post(SnaskValue* out, SnaskValue* url, SnaskValue* data) { http_request(out, "POST", url, data); }
void s_http_put(SnaskValue* out, SnaskValue* url, SnaskValue* data) { http_request(out, "PUT", url, data); }
void s_http_delete(SnaskValue* out, SnaskValue* url) { http_request(out, "DELETE", url, NULL); }
void s_http_patch(SnaskValue* out, SnaskValue* url, SnaskValue* data) { http_request(out, "PATCH", url, data); }
