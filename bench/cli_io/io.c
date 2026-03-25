#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *path = "/tmp/snask_bench_io.txt";
  const char *payload =
      "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

  FILE *f = fopen(path, "wb");
  if (!f) {
    puts("write failed");
    return 1;
  }
  fwrite(payload, 1, strlen(payload), f);
  fclose(f);

  f = fopen(path, "rb");
  if (!f) {
    puts("read failed");
    return 2;
  }
  char buf[256] = {0};
  size_t n = fread(buf, 1, sizeof(buf) - 1, f);
  fclose(f);
  buf[n] = 0;

  if (strcmp(buf, payload) != 0) {
    puts("mismatch");
    return 3;
  }

  puts("ok");
  return 0;
}

