# 📐 Memory Model: Orchestrated Memory (v0.4.0)
### Formal Contracts for Native Performance

OM v0.4.0 implements deterministic memory regions via Thread-Local Storage.

## 1. Thread-Local Shadow Arenas
Each execution thread owns an independent 128MB buffer (`__thread void* current_arena_ptr`).

## 2. Temporal Borrow Checker
The compiler performs static analysis:
- Symbols are tagged with `zone_depth`.
- If `Target.depth < Source.depth`, assignment is rejected.

## 3. SIMD Alignment
All Arena blocks are aligned to 64 bytes (`SNASK_SIMD_ALIGN`), enabling AVX/AVX-512 vectorization.

## 4. Promotion (Arena to Heap)
- **Manual**: `promote <var> to heap;`
- **Automatic**: Injected by the `llvm_generator` for escaping function returns.
