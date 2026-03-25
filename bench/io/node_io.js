const fs = require('fs');

if (process.argv.length < 4) {
  console.error(`usage: node node_io.js <path> <size_mb>`);
  process.exit(2);
}
const path = process.argv[2];
const sizeMB = parseInt(process.argv[3] || "0", 10);
if (!sizeMB || sizeMB <= 0) process.exit(2);

const chunk = Buffer.alloc(1024 * 1024, 'a');

// write
{
  const fd = fs.openSync(path, 'w');
  for (let i = 0; i < sizeMB; i++) {
    fs.writeSync(fd, chunk, 0, chunk.length);
  }
  fs.closeSync(fd);
}

const st = fs.statSync(path);
process.stdout.write(String(st.size) + "\n");
