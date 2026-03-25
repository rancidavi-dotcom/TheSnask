# 📉 Reproducibility Guide (v0.3.6)
### Verifying the Snask Performance Claims on Your Own Hardware

Trust should come from evidence. This guide explains how to reproduce the benchmarks listed in the root [README](../README.md).

---

## 🏗️ Preparation

Before running the benchmarks, ensure your system is properly configured for high-performance builds.

### Requirements:
- **Rust (Stable):** For building the compiler.
- **LLVM 15+:** For code generation and IR.
- **GCC/Clang:** For the final binary link.
- **Python 3:** (Optional) For comparison benchmarks in `bench/`.

### Commands to Setup:
```bash
# 1. Update the toolchain
rustup update stable

# 2. Build the Snask compiler in release mode
cargo build --release

# 3. Setup the local environment (installs native runtime)
./target/release/snask setup
```

---

## 📊 Benchmarking Strategy

Snask benchmarks are located in the `bench/` and `pride/` directories. Each benchmark includes a runner script.

### 1. Allocation Performance (Snask vs C vs Python)
This benchmark measures how fast 1 million simple objects can be allocated using **OM Arenas**, Standard Heap (C), and Managed Heap (Python).

**How to run:**
```bash
./target/release/snask build pride/RAM_BENCHMARKS.snask --release
./pride/RAM_BENCHMARKS 1000000
```

### 2. Binary Size Comparison
Snask optimizes the binary size by using the **Tiny Runtime Profile**.

**How to run:**
```bash
# 1. Build Snask with tiny profile
./target/release/snask build pride/BENCHMARKS.snask --profile tiny

# 2. Compare with GCC
gcc -Os -o c_mini pride/c_comparison.c -s

# 3. View sizes
ls -lh pride/BENCHMARKS c_mini
```

### 3. Startup Speed (Cold Boot)
Comparison of execution time for a "Hello World" app across different languages.

**How to run:**
```bash
time ./target/release/snask run pride/STARTUP_BENCHMARKS.snask
```

---

## 🔍 Understanding the Results

### Are my results lower?
- **Hardware:** Benchmarks on older CPUs or hard drives may show slower times.
- **LLVM Version:** Different LLVM versions have different optimization heuristics.
- **Kernel:** System calls like `mmap` (used by Arenas) have different costs on different Linux kernels.

### Verifying the Code
We encourage you to audit the benchmark source code:
- [pride/BENCHMARKS.md](../pride/BENCHMARKS.md)
- [pride/RAM_BENCHMARKS.md](../pride/RAM_BENCHMARKS.md)
- [pride/IO_BENCHMARKS.md](../pride/IO_BENCHMARKS.md)

---
🚀 **Reality check: If Snask is not fast on your system, please open an issue with your hardware specs.**
