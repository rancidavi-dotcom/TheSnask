#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <errno.h>
#include <dirent.h>
#include <sys/stat.h>
#include <unistd.h>

static void die(const char* msg) { perror(msg); exit(1); }

static void mkpath(const char* p) {
  if (mkdir(p, 0755) != 0 && errno != EEXIST) die("mkdir");
}

int main(int argc, char** argv) {
  if (argc < 3) {
    fprintf(stderr, "usage: %s <dir> <n>\n", argv[0]);
    return 2;
  }
  const char* dir = argv[1];
  long n = atol(argv[2]);
  if (n <= 0) return 2;

  mkpath(dir);

  char path[4096];
  char buf[1024];
  memset(buf, 'a', sizeof(buf));

  // create
  for (long i = 0; i < n; i++) {
    snprintf(path, sizeof(path), "%s/f_%06ld.bin", dir, i);
    FILE* f = fopen(path, "wb");
    if (!f) die("fopen");
    if (fwrite(buf, 1, sizeof(buf), f) != sizeof(buf)) die("fwrite");
    fclose(f);
  }

  // list
  DIR* d = opendir(dir);
  if (!d) die("opendir");
  long count = 0;
  for (;;) {
    struct dirent* e = readdir(d);
    if (!e) break;
    if (e->d_name[0] == '.') continue;
    count++;
  }
  closedir(d);

  // delete
  for (long i = 0; i < n; i++) {
    snprintf(path, sizeof(path), "%s/f_%06ld.bin", dir, i);
    if (unlink(path) != 0) die("unlink");
  }

  printf("%ld\n", count);
  return 0;
}

