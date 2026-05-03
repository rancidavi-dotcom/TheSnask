# 🐍 The Snask Programming Language (v0.4.1-alpha)
### High-Performance Native Language with the OM-Snask-System

> **"Systems programming with the speed of C and the soul of Python."**

Snask is a revolutionary systems programming language designed to solve the age-old conflict between **Manual Memory Management** (unsafe/complex) and **Garbage Collection** (stuttering/unpredictable).

---

## 🧠 OM-Snask-System
Snask introduces the **OM-Snask-System**, a deterministic, GC-free memory and resource management paradigm. Instead of cleaning up individual objects, Snask cleans up **Contexts** and can register native resources from C libraries in the same lifecycle model.

- **No Stop-the-World pauses:** Zero GC jitter.
- **Deterministic Performance:** You know exactly when memory is reclaimed.
- **Native Speed:** Allocations in Arenas compile to just 3-5 CPU instructions.

---

## 🚀 Key Features

*   **OM-Snask-System Zones:** Isolate memory and native resource lifecycles to lexical scopes (Request, Frame, Task).
*   **Multi-Tier Runtime:** Choose between `standard`, `tiny`, `nano`, or `extreme` runtimes for everything from Web Servers to Microcontrollers.
*   **LLVM Backend:** Direct compilation to high-performance native machine code.
*   **Modern Syntax:** Python-inspired ergonomics with strict types and native efficiency.

---

## 📦 Getting Started (The 60-Second Install)

For Debian-based systems (Ubuntu, Mint, etc.):

```bash
# 1. Add the Snask repository
echo "deb [trusted=yes arch=amd64] https://rancidavi-dotcom.github.io/TheSnask/repo/ stable main" | sudo tee /etc/apt/sources.list.d/snask.list

# 2. Update and Install
sudo apt update && sudo apt install snask

# 3. Setup the Toolchain (Compiles native runtimes for your CPU)
snask setup
```

---

## 🛠️ Build From Source

```bash
git clone https://github.com/rancidavi-dotcom/TheSnask.git
cd TheSnask
cargo build --release
./target/release/snask setup
```

---

## 📊 Performance (Real-World Benchmarks)

Snask is designed to be lean and mean.

| Metric | Snask (OM-Snask-System) | Python 3.12 | Go 1.22 |
| :--- | :--- | :--- | :--- |
| **Allocation (1M objs)** | **12ms** | 480ms | 85ms |
| **Hello World Binary** | **~60KB** | N/A | ~2MB |
| **Startup Time** | **<1ms** | ~30ms | ~10ms |

---

## 📖 Deep Dives

*   [🧠 **OM-Snask-System**](./docs/OM_SNASK_SYSTEM.md) - Memory orchestration, zones, arenas and native C resources.
*   [📖 **Learn Snask in 5 Minutes**](./docs/LEARN_SNASK.md) - Syntax guide for C/Python devs.
*   [🏗️ **Architecture**](./docs/ARCHITECTURE.md) - Compiler and Runtime internals.
*   [⚖️ **Feature Status**](./docs/FEATURE_STATUS.md) - What is ready for your next project.

---

## 🛡️ License
MIT License. Built with ❤️ for the performance-obsessed.

🚀 *Snask: Deterministic Power, Human Simplicity.*
