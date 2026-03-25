# 🛡️ Memory Safety & Threat Model (v0.3.6)
### Defining the Boundaries of Safety in Snask

Snask does not promise total compile-time "Borrow-Check" soundness like Rust. Instead, it provides **Runtime-Enforced Memory Safety** (REMS) through the Orchestrated Memory model.

---

## 1. The Soundness Claim

**Code outside `@unsafe` blocks is Memory-Safe as long as it adheres to the OM Model Invariants.** Safety is not guaranteed by type-system-based proofs, but by the physical boundaries of the **Arenas** and **Zones** managed by the runtime engine.

| Runtime Invariant | Protection | Status |
| :--- | :--- | :--- |
| **Zone Bound** | Prevents use-after-free by resetting in bulk. | ✅ Guaranteed |
| **Arena Bound** | Guard pages and bounds checks prevent overflows. | ✅ Guaranteed |
| **Graph Integrity** | Promotion preserves logical identity/cycles. | ✅ Guaranteed |
| **Stack Privacy** | Compiler warns on escaping stack objects. | ⚠️ Warning only |

---

## 2. The @unsafe "Override"

In Snask, the `@unsafe` keyword is a **Manual Override**. When using it:
1.  **Semantic guarantees are completely suspended.**
2.  **Raw memory access is permitted at the C level.**
3.  **The developer assumes all responsibility for memory integrity.**

---

## 3. Threat Model

- **Primary Goal:** Prevent massive memory leaks and undiagnosed segfaults in high-performance desktop/server apps.
- **Secondary Goal:** Catch temporal reference misuse (use-after-zone) in Debug Mode via **Memory Poisoning**.
- **Non-Goal:** Protecting against malicious, hand-crafted `@unsafe` code or out-of-order execution side-channel attacks.

---

## 4. Current Research (Pre-Alpha)

- **Hardened Runtime (Zombie Zones):** Currently testing a feature that fills reset memory with `0xCC` or `0xDEADBEEF` patterns to provide instant segfaults on stale read/write.
- **Identity Handle Engine:** Exploring a system of indirection to make pointers independent of their physical memory address during Tier Promotion.

---
🚀 **Snask is about controlled power. Know your zones, know your risks.**
