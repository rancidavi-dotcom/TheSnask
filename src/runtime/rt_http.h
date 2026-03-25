#ifndef RT_HTTP_H
#define RT_HTTP_H

#include "rt_base.h"

void s_http_get(SnaskValue* out, SnaskValue* url);
void s_http_post(SnaskValue* out, SnaskValue* url, SnaskValue* data);
void s_http_put(SnaskValue* out, SnaskValue* url, SnaskValue* data);
void s_http_delete(SnaskValue* out, SnaskValue* url);
void s_http_patch(SnaskValue* out, SnaskValue* url, SnaskValue* data);

#endif // RT_HTTP_H
