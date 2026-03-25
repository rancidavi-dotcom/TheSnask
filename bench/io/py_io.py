import os
import sys

if len(sys.argv) < 3:
    print(f"usage: python3 py_io.py <path> <size_mb>", file=sys.stderr)
    raise SystemExit(2)

path = sys.argv[1]
size_mb = int(sys.argv[2])
if size_mb <= 0:
    raise SystemExit(2)

chunk = b"a" * (1024 * 1024)

# write
with open(path, "wb", buffering=1024 * 1024) as f:
    for _ in range(size_mb):
        f.write(chunk)
    f.flush()
    # keep fairness with C/Go/Node (no fsync in this benchmark)

print(os.stat(path).st_size)
