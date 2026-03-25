#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <dlfcn.h>
#include "rt_blaze.h"
#include "rt_gc.h"
#include "rt_json.h"

// --- Blaze Internal Helpers ---
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
    const char* p = strchr(req, ' ');
    if (!p) return false;
    p++;
    const char* end = strchr(p, ' ');
    if (!end) return false;
    size_t n = (size_t)(end - p);
    if (n + 1 > out_cap) n = out_cap - 1;
    memcpy(out_path, p, n);
    out_path[n] = '\0';
    char* q = strchr(out_path, '?');
    if (q) *q = '\0';
    return true;
}

static SnaskValue blaze_obj_lookup(SnaskObject* obj, const char* key, bool* found) {
    *found = false;
    if (!obj || !key) return MAKE_NIL();
    for (int i = 0; i < obj->count; i++) {
        if (obj->names[i] && strcmp(obj->names[i], key) == 0) {
            *found = true;
            return obj->values[i];
        }
    }
    return MAKE_NIL();
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

static void blaze_send_response_headers(int fd, int status, const char* content_type, const char* header_line, const char* cookie_line, const char* body) {
    if (!content_type) content_type = "text/plain; charset=utf-8";
    if (!body) body = "";
    char header[2048];
    int body_len = (int)strlen(body);
    const char* status_text = (status == 200) ? "OK" : (status == 302) ? "Found" : (status == 404) ? "Not Found" : "Error";
    
    int n = snprintf(header, sizeof(header),
        "HTTP/1.1 %d %s\r\n"
        "Content-Type: %s\r\n"
        "Content-Length: %d\r\n",
        status, status_text, content_type, body_len);
    
    if (header_line && header_line[0]) n += snprintf(header + n, sizeof(header) - (size_t)n, "%s\r\n", header_line);
    if (cookie_line && cookie_line[0]) n += snprintf(header + n, sizeof(header) - (size_t)n, "Set-Cookie: %s\r\n", cookie_line);
    
    snprintf(header + n, sizeof(header) - (size_t)n, "Connection: close\r\n\r\n");
    blaze_send_all(fd, header);
    blaze_send_all(fd, body);
}

static SnaskValue blaze_call_snask_handler(const char* handler_name, const char* method, const char* path, const char* query, const char* body, const char* cookie) {
    char sym[512]; snprintf(sym, 512, "f_%s", handler_name);
    void* fp = dlsym(NULL, sym);
    if (!fp) return MAKE_NIL();
    typedef void (*SnaskFn6)(SnaskValue*, SnaskValue*, SnaskValue*, SnaskValue*, SnaskValue*, SnaskValue*);
    SnaskFn6 f = (SnaskFn6)fp;
    SnaskValue ra = MAKE_NIL();
    SnaskValue vm = MAKE_STR(snask_gc_strdup(method));
    SnaskValue vp = MAKE_STR(snask_gc_strdup(path));
    SnaskValue vq = MAKE_STR(snask_gc_strdup(query));
    SnaskValue vb = MAKE_STR(snask_gc_strdup(body));
    SnaskValue vc = MAKE_STR(snask_gc_strdup(cookie));
    f(&ra, &vm, &vp, &vq, &vb, &vc);
    return ra;
}

void blaze_run(SnaskValue* out, SnaskValue* port_val, SnaskValue* routes_val) {
    if ((int)port_val->tag != SNASK_NUM || (int)routes_val->tag != SNASK_OBJ) { *out = MAKE_BOOL(false); return; }
    int port = (int)port_val->num;
    int server_fd = socket(AF_INET, SOCK_STREAM, 0);
    if (server_fd < 0) { *out = MAKE_BOOL(false); return; }
    int opt = 1; setsockopt(server_fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));
    struct sockaddr_in addr = { .sin_family = AF_INET, .sin_addr.s_addr = htonl(INADDR_ANY), .sin_port = htons((uint16_t)port) };
    if (bind(server_fd, (struct sockaddr*)&addr, sizeof(addr)) != 0 || listen(server_fd, 64) != 0) { close(server_fd); *out = MAKE_BOOL(false); return; }

    SnaskObject* routes = (SnaskObject*)routes_val->ptr;
    *out = MAKE_BOOL(true);
    for (;;) {
        int client_fd = accept(server_fd, NULL, NULL);
        if (client_fd < 0) continue;
        char req[16384]; ssize_t n = recv(client_fd, req, sizeof(req) - 1, 0);
        if (n <= 0) { close(client_fd); continue; }
        req[n] = 0;

        char path_key[1024];
        if (!blaze_parse_path(req, path_key, sizeof(path_key))) { close(client_fd); continue; }
        const char* method = blaze_find_method(req);
        
        bool found = false;
        SnaskValue v = blaze_obj_lookup(routes, path_key, &found);
        if (!found && method) {
            char mpath[1200]; snprintf(mpath, 1200, "%s %s", method, path_key);
            v = blaze_obj_lookup(routes, mpath, &found);
        }

        if (found && (int)v.tag == SNASK_OBJ) {
            bool has_h = false; SnaskValue hv = blaze_obj_lookup((SnaskObject*)v.ptr, "handler", &has_h);
            if (has_h && (int)hv.tag == SNASK_STR) {
                v = blaze_call_snask_handler((const char*)hv.ptr, method ? method : "GET", path_key, "", "", "");
            }
        }

        if (found) {
            if ((int)v.tag == SNASK_OBJ) {
                SnaskObject* resp = (SnaskObject*)v.ptr;
                bool has_b, has_j, has_s, has_ct, has_h, has_c;
                SnaskValue bv = blaze_obj_lookup(resp, "body", &has_b);
                SnaskValue jv = blaze_obj_lookup(resp, "json", &has_j);
                SnaskValue sv = blaze_obj_lookup(resp, "status", &has_s);
                SnaskValue ctv = blaze_obj_lookup(resp, "content_type", &has_ct);
                SnaskValue hv = blaze_obj_lookup(resp, "header", &has_h);
                SnaskValue cv = blaze_obj_lookup(resp, "cookie", &has_c);

                int status = has_s ? (int)sv.num : 200;
                const char* ct = has_ct ? (const char*)ctv.ptr : (has_j ? "application/json" : "text/plain");
                const char* body_str = "";
                SnaskValue js;
                if (has_b) body_str = (const char*)bv.ptr;
                else if (has_j) { json_stringify(&js, &jv); body_str = (const char*)js.ptr; }
                
                blaze_send_response_headers(client_fd, status, ct, has_h ? (const char*)hv.ptr : NULL, has_c ? (const char*)cv.ptr : NULL, body_str);
            } else {
                blaze_send_response_headers(client_fd, 200, "text/plain", NULL, NULL, (int)v.tag == SNASK_STR ? (const char*)v.ptr : "OK");
            }
        } else {
            blaze_send_response_headers(client_fd, 404, "text/plain", NULL, NULL, "Not Found");
        }
        close(client_fd);
    }
}

void blaze_qs_get(SnaskValue* out, SnaskValue* qs, SnaskValue* key) {
    if ((int)qs->tag != SNASK_STR || (int)key->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    const char* s = (const char*)qs->ptr;
    const char* k = (const char*)key->ptr;
    const char* p = strstr(s, k);
    if (p && (p == s || *(p-1) == '&' || *(p-1) == '?') && *(p + strlen(k)) == '=') {
        const char* v = p + strlen(k) + 1;
        const char* end = strchr(v, '&');
        size_t len = end ? (size_t)(end - v) : strlen(v);
        *out = MAKE_STR(snask_gc_strndup(v, len));
    } else *out = MAKE_NIL();
}

void blaze_cookie_get(SnaskValue* out, SnaskValue* cookie_header, SnaskValue* name) {
    blaze_qs_get(out, cookie_header, name); // Similar logic
}
