# 🏗️ Compiler & Runtime Architecture (v0.4.0)
### The Internal Design of the Snask Platform

This document explains the internal mechanisms of Snask v0.4.0.

---

## 1. Overview: The Compilation Pipeline

Snask uses an ahead-of-time (AOT) compilation strategy targeting LLVM IR.

Status note:
- This document describes the current pipeline plus the intended direction of OM.
- Some OM claims that appeared in older docs are still in progress and must not be read as fully implemented guarantees.
- For feature-by-feature reality, see `docs/FEATURE_STATUS.md`.

```mermaid
graph TD
    A[Snask Source .snask] --> B[Lexer/Parser]
    B --> C[AST]
    C --> D[Semantic Analyzer - Type Checks, Scopes, OM Groundwork]
    D --> OM[OM-Snask-System - C Header Scan & Contract Inference]
    OM --> E[LLVM IR Generator - Native Calls, TLS Injection & SIMD]
    E --> F[Linker - Runtime + C Libraries]
    F --> G[Native Binary]
```

## 2. Orchestrated Memory (OM) v0.4.0
- **Current Reality**: Snask already exposes OM-oriented syntax and runtime strategies such as `stack`, `heap`, `arena`, `zone`, `scope`, `promote`, and `entangle`.
- **Semantic State**: The compiler currently performs scope and type checks, but a full borrow checker and formal `zone_depth` escape analysis are still planned work.
- **Runtime Direction**: The runtime already contains thread-local and specialized allocation pieces, but the language-level OM contract is not yet fully formalized.
- **Near-Term Goal**: Turn OM from a promising implementation direction into a specified, testable, compile-time-enforced model.

## 3. OM-Snask-System / Auto-OM

Snask now has an experimental C interop layer that scans C headers, deduces OM contracts, applies optional `.om.snif` patches, and emits native LLVM calls to external C symbols. This does not make Snask a transpiler: the output remains LLVM/native binary, and the C library is used through ABI-level calls.

The user-facing goal is:

```snask
import_c_om "SDL2/SDL.h" as sdl2

zone "app":
    let window = sdl2.create_window("App", 0, 0, 800, 600, sdl2.WINDOW_HIDDEN)
```

The compiler/runtime goal is:

- infer safe constants, functions and opaque resources from the header;
- hide manual destructors behind OM zone cleanup;
- use `.om.snif` only as a small patch for exceptional APIs;
- block C functions whose pointer ownership cannot be proven safe.

For the full design, current status, examples and roadmap, see `docs/OM_SNASK_SYSTEM.md`.

---
🚀 **Auditable code, predictable performance. That's the Snask promise.**
