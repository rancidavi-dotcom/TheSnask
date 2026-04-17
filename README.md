# 🐍 The Snask Programming Language (v0.3.6-pre-alpha)
### Experimental Native Language with Orchestrated Memory (OM)

> **⚠️ WARNING:** Snask is in an **early experimental** (pre-alpha) stage. It is a research project exploring low-level memory orchestration patterns. **Not for production use.**

Snask aims to provide high-level syntax with the deterministic control of a low-level language by using **Orchestrated Memory (OM)**.

---

## 🛡️ Trust & Security

In a world of "curl | bash" installers, we choose transparency. We encourage developers to audit the source code and build from source to ensure full control over the binaries being executed.

- **Open Source:** Auditable Rust (compiler) and C (runtime) code.
- **No Hidden Magic:** All memory behaviors are explicit in the LLVM IR generation.
- **Reproducible Results:** Benchmarks can be verified on your own hardware (see `docs/REPRODUCIBILITY.md`).

---

## 🚀 Key Research Areas

*   **Orchestrated Memory (OM):** Exploring deterministic, GC-free memory management via **Zones**, **Arenas**, and **Stack** frames.
*   **Zero-Overhead Abstraction:** Attempting high-level features (dynamic dispatch) without the typical performance penalty of managed Runtimes.
*   **Native Tooling:** A unified build system (SPS) built entirely with performance and size in mind.

---

## 📦 Installation (Debian/Ubuntu)

For a professional experience on Debian-based systems, you can use the official Snask APT repository. This allows for easy updates via `sudo apt upgrade`.

### Quick Setup:
```bash
# 1. Add the Snask repository (currently [trusted=yes] for experimental phase)
echo "deb [trusted=yes arch=amd64] https://rancidavi-dotcom.github.io/TheSnask/repo/ stable main" | sudo tee /etc/apt/sources.list.d/snask.list

# 2. Update and Install
sudo apt update
sudo apt install snask
```

---

## 🛠️ Building From Source

To ensure transparency and security, follow the manual build process:

### Prerequisites:
- **Rust (Cargo):** For compiling the Snask compiler.
- **LLVM 15+:** For code generation.
- **GCC/Clang:** For linking the native runtime.

### Build Steps:
```bash
# 1. Clone the repository
git clone https://github.com/rancidavi-dotcom/TheSnask.git
cd TheSnask

# 2. Compile the Snask compiler
cargo build --release

# 3. Setup the local environment (installs native runtime)
./target/release/snask setup
```

---

## 📊 Proof of Concept (Benchmarks)

Don't trust our claims; run the evidence. 

| Metric | Snask (OM) | Conventional C (GCC) | Python 3.x |
| :--- | :--- | :--- | :--- |
| **Alloc (1M objs)** | **12ms** | 45ms | 450ms+ |
| **Binary Size** | **~200KB** | ~20KB | N/A (Runtime req) |
| **RAM Usage** | **~8MB** | ~1MB | ~30MB+ |

*Detailed instructions on how to reproduce these metrics: [docs/REPRODUCIBILITY.md](./docs/REPRODUCIBILITY.md)*

---

## 📖 Essential Documentation

*   [📖 **Learn Snask**](./docs/LEARN_SNASK.md) - Language syntax and survival tips.
*   [🧭 **Feature Status**](./docs/FEATURE_STATUS.md) - What is currently stable, partial, experimental, or planned.
*   [🧠 **Orchestrated Memory (OM)**](./docs/OM_GUIDE.md) - Deep dive into the core research.
*   [🏗️ **Compiler Architecture**](./docs/ARCHITECTURE.md) - How we generate LLVM IR.
*   [⚖️ **Stability Policy**](./docs/STABILITY.md) - Current pre-alpha status.

---

## 🛡️ License
MIT License. Explore, fork, and challenge the design.

🚀 *Snask: A research on performance, deterministic memory, and simple syntax.*
