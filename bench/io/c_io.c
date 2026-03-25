#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <sys/stat.h>

static void die(const char* msg) { perror(msg); exit(1); }

int main(int argc, char** argv) {
  if (argc < 3) {
    fprintf(stderr, "usage: %s <path> <size_mb>\n", argv[0]);
    return 2;
  }
  const char* path = argv[1];
  long size_mb = atol(argv[2]);
  if (size_mb <= 0) return 2;

  const size_t chunk = 1024 * 1024;
  char* buf = (char*)malloc(chunk);
  if (!buf) die("malloc");
  memset(buf, 'a', chunk);

  FILE* f = fopen(path, "wb");
  if (!f) die("fopen write");
  for (long i = 0; i < size_mb; i++) {
    if (fwrite(buf, 1, chunk, f) != chunk) die("fwrite");
  }
  fflush(f);
  fclose(f);

  struct stat st;
  if (stat(path, &st) != 0) die("stat");
  printf("%llu\n", (unsigned long long)st.st_size);
  free(buf);
  return 0;
}
