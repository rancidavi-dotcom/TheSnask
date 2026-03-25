const fs = require('fs');
const path = require('path');

if (process.argv.length < 4) {
  console.error(`usage: node node_fs_small.js <dir> <n>`);
  process.exit(2);
}

const dir = process.argv[2];
const n = parseInt(process.argv[3] || "0", 10);
if (!n || n <= 0) process.exit(2);

fs.mkdirSync(dir, { recursive: true });

const buf = Buffer.alloc(1024, 'a');

for (let i = 0; i < n; i++) {
  const p = path.join(dir, `f_${String(i).padStart(6,'0')}.bin`);
  fs.writeFileSync(p, buf);
}

const entries = fs.readdirSync(dir);
let count = 0;
for (const e of entries) {
  if (e.startsWith('.')) continue;
  count++;
}

for (let i = 0; i < n; i++) {
  const p = path.join(dir, `f_${String(i).padStart(6,'0')}.bin`);
  try { fs.unlinkSync(p); } catch {}
}

process.stdout.write(String(count) + "\n");

