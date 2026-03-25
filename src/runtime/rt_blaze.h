#ifndef RT_BLAZE_H
#define RT_BLAZE_H

#include "rt_base.h"

void blaze_run(SnaskValue* out, SnaskValue* port_val, SnaskValue* routes_val);
void blaze_qs_get(SnaskValue* out, SnaskValue* qs, SnaskValue* key);
void blaze_cookie_get(SnaskValue* out, SnaskValue* cookie_header, SnaskValue* name);

#endif // RT_BLAZE_H
