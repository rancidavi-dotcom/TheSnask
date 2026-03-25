import os
import sys

if len(sys.argv) < 3:
    print(f"usage: python3 py_fs_small.py <dir> <n>", file=sys.stderr)
    raise SystemExit(2)

dirp = sys.argv[1]
n = int(sys.argv[2])
if n <= 0:
    raise SystemExit(2)

os.makedirs(dirp, exist_ok=True)
buf = b"a" * 1024

for i in range(n):
    p = os.path.join(dirp, f"f_{i:06d}.bin")
    with open(p, "wb") as f:
        f.write(buf)

count = 0
for name in os.listdir(dirp):
    if name.startswith("."):
        continue
    count += 1

for i in range(n):
    p = os.path.join(dirp, f"f_{i:06d}.bin")
    try:
        os.remove(p)
    except FileNotFoundError:
        pass

print(count)

