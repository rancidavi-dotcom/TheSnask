#ifndef RT_AUTH_H
#define RT_AUTH_H

#include "rt_base.h"

// Password hashing (v1 format: v1$<salt>$<hash16>)
void auth_hash_password(SnaskValue* out, SnaskValue* password);
void auth_verify_password(SnaskValue* out, SnaskValue* password, SnaskValue* stored_hash);

// Session / Token management
void auth_session_id(SnaskValue* out);
void auth_csrf_token(SnaskValue* out);

// Cookie helpers
void auth_cookie_kv(SnaskValue* out, SnaskValue* name, SnaskValue* value);
void auth_cookie_session(SnaskValue* out, SnaskValue* sid);
void auth_cookie_delete(SnaskValue* out, SnaskValue* name);

// HTTP Header helpers
void auth_bearer_header(SnaskValue* out, SnaskValue* token);

// Status helpers
void auth_ok(SnaskValue* out);
void auth_fail(SnaskValue* out);

// Versioning
void auth_version(SnaskValue* out);

#endif // RT_AUTH_H
