# Snask Baremetal — Atomics & Memory Fencing

> Memory ordering fences and atomic read-modify-write operations for
> multiprocessor synchronization and device interaction.

---

## 1. `fence`

Insert a memory fence/barrier at the current point.

### Syntax

```snask
fence ordering
```

### Orderings

| Ordering | LLVM Mapping | x86-64 Instruction | When To Use |
|----------|-------------|-------------------|-------------|
| `acquire` | `Acquire` | `lfence` (implied) | Load-after-load ordering |
| `release` | `Release` | `sfence` (implied) | Store-after-store ordering |
| `acqrel` | `AcquireRelease` | `mfence` (implied) | Full acquire-release |
| `seq_cst` | `SequentiallyConsistent` | `mfence` | Sequential consistency |

### Examples

```snask
// Producer: write data, then release flag
mem_write_u32(buffer, 0, data)
fence release
@store(flag_addr, 1)

// Consumer: acquire flag, then read data
while @volatile(flag_addr, U32) == 0
    @asm("pause")
fence acquire
let result: U32 = mem_read_u32(buffer, 0)
```

### With MMIO

Ensure that writes to MMIO registers are visible to the device before
subsequent operations:

```snask
@raw fun uart_putc(c: U8)
    // Wait for transmitter to be ready
    while (@volatile(COM1 + 5, U8) & 0x20) == 0
        @asm("pause")

    // Ensure previous reads complete before write
    fence acquire

    // Write data
    @write(COM1 + 0, c, U8)

    // Ensure write reaches device before continuing
    fence release
```

---

## 2. `atomic rmw`

Atomic read-modify-write operations on memory locations.
Essential for lock-free data structures, reference counting,
and device register access on SMP systems.

### Syntax

```snask
atomic rmw(operation, ptr, value, ordering)
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `operation` | identifier | The atomic operation to perform |
| `ptr` | `ptr` or `U64` | Pointer to the memory location |
| `value` | any integer | The operand value |
| `ordering` | identifier | Memory ordering |

### Operations

| Operation | Effect | C11 Equivalent | x86-64 Instruction |
|-----------|--------|---------------|-------------------|
| `add` | `*ptr += value` | `atomic_fetch_add` | `lock xadd` |
| `sub` | `*ptr -= value` | `atomic_fetch_sub` | `lock xadd` with neg |
| `or` | `*ptr \|= value` | `atomic_fetch_or` | `lock or` |
| `and` | `*ptr &= value` | `atomic_fetch_and` | `lock and` |
| `xor` | `*ptr ^= value` | `atomic_fetch_xor` | `lock xor` |
| `xchg` | `*ptr = value` (returns old) | `atomic_exchange` | `lock xchg` |

### Orderings

Same as `fence`: `acquire`, `release`, `acqrel`, `seq_cst`.

### Return Value

`atomic rmw` returns the **previous value** of the memory location.

### Examples

#### Lock-free Reference Counting

```snask
@raw fun ref_inc(ref: ptr) : U32
    let prev: U32 = atomic rmw(add, ref, 1, seq_cst)
    return prev + 1

@raw fun ref_dec(ref: ptr) : U32
    return atomic rmw(sub, ref, 1, seq_cst)
```

#### Spinlock

```snask
@raw fun spin_lock(lock: ptr)
    loop
        let old: U32 = atomic rmw(xchg, lock, 1, acquire)
        if old == 0
            break
        while @volatile(lock, U32) == 1
            @asm("pause")

@raw fun spin_unlock(lock: ptr)
    fence release
    @store(lock, 0)
```

#### Atomic Counter (SMP-safe)

```snask
mut global_counter: U64 = 0

@raw fun next_ticket() : U64
    return atomic rmw(add, &global_counter, 1, seq_cst)
```

#### Device Register Update (with masking)

```snask
@raw fun set_pci_command(bus: U32, dev: U32, func: U32, bits: U16)
    let addr: U64 = pci_config_addr(bus, dev, func, 0x04)  // command reg
    let old: U16 = atomic rmw(or, @inttoptr(addr, ptr), bits, seq_cst)
    // old value available if needed

@raw fun clear_pci_command(bus: U32, dev: U32, func: U32, bits: U16)
    let addr: U64 = pci_config_addr(bus, dev, func, 0x04)
    let old: U16 = atomic rmw(and, @inttoptr(addr, ptr), ~bits, seq_cst)
```

---

## 3. Memory Ordering Reference

| Ordering | CPU Reordering Allowed | Compiler Reordering Allowed | Cost |
|----------|----------------------|---------------------------|------|
| `relaxed` | All | All | Free |
| `acquire` | Loads after (none) | Loads after (none) | Low |
| `release` | Stores before (none) | Stores before (none) | Low |
| `acqrel` | Full barrier | Full barrier | Medium |
| `seq_cst` | Full barrier + global order | Full barrier | High |

In baremetal mode, `seq_cst` is appropriate for most kernel synchronization
primitives. Use `acquire`/`release` for performance-critical paths.

---

## 4. Common Patterns

### 4.1 Compare-and-Swap (via LL/SC or CMPXCHG)

Note: Snask does not have a direct CAS built-in. Use inline assembly:

```snask
@raw fun atomic_cas(ptr: ptr, expected: U64, desired: U64) : Bool
    @asm("lock cmpxchg [rdi], rdx")
    @asm("setz al")
    return false  // actual result from flags via setz
```

### 4.2 Sequenced MMIO Access

```snask
@raw fun mmio_write(addr: ptr, offset: U64, val: U32)
    fence release                 // complete pending writes
    @write(@ptrtoint(addr, U64) + offset, val, U32)
    fence seq_cst                 // flush write buffer to device
```

### 4.3 Per-CPU Variables

```snask
// On x86_64, GS segment is often used for per-CPU data
@raw fn read_percpu(offset: U64) : U64
    @asm("mov rax, gs:[rdi]")
    return 0
```

---

## 5. Platform Differences

| Feature | x86_64 | ARM64 | RISC-V |
|---------|--------|-------|--------|
| `fence acquire` | Implicit `lfence` | `dmb ishld` | `fence r, rw` |
| `fence release` | Implicit `sfence` | `dmb ishst` | `fence rw, w` |
| `fence seq_cst` | `mfence` | `dmb ish` | `fence iorw, iorw` |
| `atomic rmw add` | `lock xadd` | `ldadd` | `amoadd.d` |
| `atomic rmw xchg` | `lock xchg` | `swpal` | `amoswap.d` |

The Snask compiler generates the correct LLVM IR and the backend produces
the appropriate architecture-specific instructions.
