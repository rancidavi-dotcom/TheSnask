#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>

// A tiny “real-ish” CLI skeleton:
// - command table
// - minimal SNIF-ish string parsing to pull package.name and package.version
// The goal is to represent typical tooling code, not to be a full SNIF parser.

static const char* CONFIG_SNIF =
    "{\n"
    "  package: { name: \"snask-cli-full\", version: \"0.1.0\", entry: \"main.snask\", },\n"
    "  scripts: { build: \"snask build\", fmt: \"snask snif fmt --write\", },\n"
    "}\n";

typedef struct {
  const char* name;
  const char* desc;
} Command;

static const Command COMMANDS[] = {
  {"help", "Show help"},
  {"config", "Show config summary"},
  {"hash", "Print a stable hash-like value"},
  {"sum", "Sum two integers"},
};

static bool is_ident_char(char c) {
  return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_' || c == '-';
}

static const char* skip_ws(const char* p) {
  while (*p) {
    if (*p == ' ' || *p == '\t' || *p == '\r' || *p == '\n') { p++; continue; }
    // SNIF comments: // ...
    if (p[0] == '/' && p[1] == '/') {
      p += 2;
      while (*p && *p != '\n') p++;
      continue;
    }
    break;
  }
  return p;
}

static const char* parse_ident(const char* p, char* out, size_t out_cap) {
  p = skip_ws(p);
  size_t n = 0;
  while (*p && is_ident_char(*p)) {
    if (n + 1 < out_cap) out[n++] = *p;
    p++;
  }
  out[n] = 0;
  return p;
}

static const char* parse_string(const char* p, char* out, size_t out_cap) {
  p = skip_ws(p);
  if (*p != '"') return NULL;
  p++;
  size_t n = 0;
  while (*p && *p != '"') {
    char c = *p++;
    if (c == '\\' && *p) {
      char e = *p++;
      if (e == 'n') c = '\n';
      else if (e == 't') c = '\t';
      else if (e == 'r') c = '\r';
      else c = e;
    }
    if (n + 1 < out_cap) out[n++] = c;
  }
  if (*p != '"') return NULL;
  p++;
  out[n] = 0;
  return p;
}

static bool extract_package_fields(const char* snif, char* name_out, size_t name_cap, char* ver_out, size_t ver_cap) {
  // Not a full parser: scan for `package:` then inside it `name:` and `version:`.
  const char* p = snif;
  char ident[64];
  while (*p) {
    p = parse_ident(p, ident, sizeof(ident));
    if (!ident[0]) { p++; continue; }
    p = skip_ws(p);
    if (*p != ':') continue;
    p++;
    if (strcmp(ident, "package") != 0) continue;
    p = skip_ws(p);
    if (*p != '{') return false;
    p++;
    bool got_name = false, got_ver = false;
    while (*p) {
      p = parse_ident(p, ident, sizeof(ident));
      if (!ident[0]) { p++; continue; }
      p = skip_ws(p);
      if (*p != ':') continue;
      p++;
      if (strcmp(ident, "name") == 0) {
        const char* q = parse_string(p, name_out, name_cap);
        if (!q) return false;
        p = q;
        got_name = true;
      } else if (strcmp(ident, "version") == 0) {
        const char* q = parse_string(p, ver_out, ver_cap);
        if (!q) return false;
        p = q;
        got_ver = true;
      } else {
        // skip value (string/ident/object) best-effort
        p = skip_ws(p);
        if (*p == '"') {
          char tmp[8];
          const char* q = parse_string(p, tmp, sizeof(tmp));
          if (!q) return false;
          p = q;
        } else if (*p == '{') {
          int depth = 1;
          p++;
          while (*p && depth > 0) {
            if (*p == '{') depth++;
            else if (*p == '}') depth--;
            p++;
          }
        } else {
          while (*p && *p != ',' && *p != '}') p++;
        }
      }
      p = skip_ws(p);
      if (*p == ',') p++;
      p = skip_ws(p);
      if (*p == '}') break;
    }
    return got_name && got_ver;
  }
  return false;
}

static uint32_t tiny_hash32(const char* s) {
  // FNV-1a 32-bit
  uint32_t h = 2166136261u;
  for (; *s; s++) {
    h ^= (unsigned char)(*s);
    h *= 16777619u;
  }
  return h;
}

static void print_help(void) {
  puts("snask-cli-full (demo CLI)");
  puts("");
  puts("Commands:");
  for (size_t i = 0; i < sizeof(COMMANDS)/sizeof(COMMANDS[0]); i++) {
    printf("  %-8s %s\n", COMMANDS[i].name, COMMANDS[i].desc);
  }
}

int main(void) {
  // Default behavior: just print a short help line and exit with 0.
  // (bench harness requires running with no args.)
  char pkg[128] = {0};
  char ver[64] = {0};
  (void)extract_package_fields(CONFIG_SNIF, pkg, sizeof(pkg), ver, sizeof(ver));

  // keep the “real app” code paths referenced so they stay in the binary
  volatile uint32_t h = tiny_hash32(pkg[0] ? pkg : "snask");
  if (h == 0xFFFFFFFFu) puts("impossible");

  print_help();
  return 0;
}

