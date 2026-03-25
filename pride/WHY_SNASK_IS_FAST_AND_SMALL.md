# Why Snask can be *smaller than conventional C* (and still high-level)

Snask’s goal is not “a language demo”. It’s a **batteries-included platform language** that still produces **real native executables**.

This document explains, in plain terms, how Snask achieves **very small binaries** without changing how you write Snask code.

## 1) Snask ships a controlled toolchain pipeline

Most “C conventional” builds are:

```bash
gcc app.c -o app
```

That’s fast and common, but it’s not size-optimized by default.

Snask’s build pipeline is size-aware:
- LLVM IR generation (compiler knows exactly what’s used)
- aggressive linker garbage collection (dead code removal)
- best-effort stripping
- optional LLD optimizations (ICF)
- “tiny” and “ultra-tiny” profiles for tooling/CLI

## 2) Dead code elimination that actually matters

Snask builds with:
- `-ffunction-sections -fdata-sections` (runtime)
- `-Wl,--gc-sections` (linker removes unused sections)
- `-Wl,--as-needed` (avoid pulling unnecessary dynamic deps)

That means the final binary only keeps what is actually referenced.

## 3) No “export everything”

Conventional builds often carry extra exported symbol baggage when flags like `-rdynamic` are used.

Snask size profiles intentionally avoid export-heavy linking unless necessary.

## 4) Ultra-tiny: custom `_start` (no CRT)

For ultra-tiny builds (Linux x86_64 for now), Snask uses a custom Assembly entrypoint:
- avoids CRT startup overhead
- calls `main()` directly
- exits via syscall

This is a classic “systems trick” that you normally only see in hand-tuned C/ASM projects — Snask makes it **a one-flag build profile**.

## 5) Same language, different runtime slices

Snask does not require you to change your syntax to get smaller binaries.

Instead, Snask selects different runtime artifacts depending on the build profile:
- full runtime (GUI/SQLite/etc.)
- tiny runtime (minimal runtime)
- ultra-tiny start object + tiny runtime (no CRT)

If your program imports heavy subsystems (GUI/SQLite), Snask stays honest and uses the appropriate profile — no hidden breakage.

## 6) Reproducible proof

See `pride/BENCHMARKS.md` for a reproducible benchmark:
- Snask vs “conventional C (gcc defaults)”
- results measured by `stat` bytes + `llvm-size`

