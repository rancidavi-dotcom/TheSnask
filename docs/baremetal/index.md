# Snask Baremetal — Documentation

> Write operating systems, hypervisors, bootloaders, and emulators in a high-level language
> with full control over memory, interrupts, and hardware.

---

## Table of Contents

- [Language Fundamentals](language.md) — types, variables, control flow, operators, comments
- [Function Types](functions.md) — `@raw fun`, `@naked fun`, `@extern fun`
- [Memory Operations](memory.md) — `@store`, `@volatile`, `@write`, `mem_*` builtins, pointer ops
- [Inline Assembly & Port I/O](asm.md) — `@asm`, `@global_asm`, `@outb`, `inb`
- [Atomics & Fencing](atomics.md) — `fence`, `atomic rmw`
- [Structs & Layout](structs.md) — `struct`, `sizeof`, `alignof`, `offsetof`
- [Built-ins Reference](builtins.md) — complete API of all systems-level intrinsics
- [Examples](examples.md) — full working kernels, drivers, and emulators

---

## 1. What is Snask Baremetal?

Snask is a high-level systems language that compiles directly to machine code via LLVM.
The **baremetal profile** strips away the standard library, runtime, and garbage collector,
giving you full, unchecked access to the CPU and memory — exactly what you need for:

- **Operating system kernels** (x86_64, ARM, RISC-V)
- **Bootloaders** (Multiboot, UEFI)
- **Hypervisors and monitors**
- **Emulators** (Chip-8, Game Boy, NES)
- **Firmware and embedded systems**
- **Real-time systems**

### Philosophy

Baremetal Snask puts you as close to the metal as C or Rust, but with a cleaner syntax,
built-in type-safe low-level operations, and first-class support for inline assembly,
memory-mapped I/O, and interrupt handling.

| Concern | Snask Baremetal | C | Rust |
|---------|----------------|---|---|
| Zero-overhead FFI | `@extern fun` | `extern` | `extern "C"` |
| Inline assembly | `@asm("...")` | `asm("...")` | `asm!("...")` |
| Naked ISR handlers | `@naked fun` | `__attribute__((naked))` | `#[naked]` |
| Volatile access | `@volatile` / `@store` | `volatile*` | `core::ptr::read_volatile` |
| MMIO at addresses | `@inttoptr` + `@volatile` | `(volatile T*)addr` | `addr as *mut T` |
| Port I/O | `@outb` / `inb` | `outb()` / `inb()` | `x86::io::outb` |
| Linker control | `linker.ld` | `linker.ld` | `linker.ld` |
| No runtime | default | default | `#![no_std]` |
| No name mangling | `@extern` / `@raw` | default | `extern "C"` |

---

## 2. Getting Started

### 2.1 Install Snask

```bash
git clone https://github.com/your-org/snask
cd snask
cargo build --release
cp target/release/snask /usr/local/bin/
```

### 2.2 Your First Kernel

Create `kernel.snask`:

```snask
@raw fun kmain() : I32
    @store(0xB8000, 0x0F4F)
    loop
        @asm("hlt")
    return 0
```

| Instruction | Meaning |
|-------------|---------|
| `@raw fun kmain() : I32` | Raw function — unboxed params, direct return |
| `@store(0xB8000, 0x0F4F)` | Write `0x0F4F` (white 'O' on black) to VGA text buffer |
| `@asm("hlt")` | Halt the CPU until next interrupt |

### 2.3 Linker Script

Create `linker.ld`:

```ld
ENTRY(kmain)

SECTIONS
{
    . = 0x100000;

    .text : { *(.text*) }
    .rodata : { *(.rodata*) }
    .data : { *(.data*) }
    .bss : { *(.bss*) }
}
```

### 2.4 Build & Run

```bash
snask build kernel.snask --profile baremetal
qemu-system-x86_64 -kernel kernel
```

The `--profile baremetal` flag enables:
- `-ffreestanding -nostdlib -mno-red-zone`
- No runtime linking
- Global variables → linker sections
- Blocked: stdio, JSON, HTTP, filesystem, GUI modules

---

## 3. Project Structure for a Real OS

```
myos/
├── kernel.snask          // Entry point, main loop
├── vga.snask             // VGA text mode driver
├── idt.snask             // Interrupt descriptor table
├── gdt.snask             // Global descriptor table
├── paging.snask          // Page tables, memory mapping
├── uart.snask            // Serial port driver
├── timer.snask           // PIT/HPET timer driver
├── keyboard.snask        // PS/2 keyboard driver
├── linker.ld             // Linker script
└── build.sh              // Build script
```

---

## 4. Core Concepts

### 4.1 No Runtime, No Garbage Collector

In baremetal mode, there is no heap allocator, no GC, no type tags, no boxing.
What you write is what the CPU executes. `@raw fun` parameters are CPU registers,
`@raw fun` return values are CPU registers.

### 4.2 Functions Are Your API

| Function Type | Prologue | Params | Returns | Use Case |
|---------------|----------|--------|---------|----------|
| `@raw fun` | Yes | Registers | Register | Normal kernel functions |
| `@naked fun` | None | Registers | via `@asm` | Interrupt handlers |
| `@extern fun` | C ABI | C ABI | C ABI | Linking with C objects |

### 4.3 Memory Access

Every memory access in baremetal is unchecked. There is no MMU setup done by Snask.
You read and write directly to physical or virtual addresses using:

| Operation | Syntax |
|-----------|--------|
| Volatile read | `@volatile(address, Type)` |
| Volatile write | `@store(address, value)` |
| Typed volatile write | `@write(address, value, Type)` |
| Raw memory read | `mem_read_u8/16/32(ptr, offset)` |
| Raw memory write | `mem_write_u8/16/32(ptr, offset, val)` |
| Fill memory | `mem_fill_u8(ptr, val, len)` |
| Copy memory | `mem_copy(dst, src, len)` |

---

## 5. Translation to C / Assembly

Every Snask baremetal construct maps directly to C or assembly:

```snask
@raw fun write_vga(addr: U64, val: U16)
    @store(addr, val)
```

**Equivalent C:**

```c
void write_vga(uint64_t addr, uint16_t val) {
    *(volatile uint16_t*)addr = val;
}
```

```snask
@naked fun isr_handler()
    @asm("iretq")
```

**Equivalent C (GCC):**

```c
__attribute__((interrupt)) void isr_handler() {
    // compiler generates iretq
}
```

Or with naked:
```c
__attribute__((naked)) void isr_handler() {
    asm("iretq");
}
```

---

## 6. Build Profile Reference

| Flag | Effect |
|------|--------|
| `--profile baremetal` | No libc, no runtime, freestanding |
| `-ffreestanding` | No hosted libc assumptions |
| `-nostdlib` | No C standard library |
| `-mno-red-zone` | x86_64: no red zone (required for interrupts) |

The Snask compiler automatically generates a `__snask_init` function that
initializes all global variables before control reaches your entry point.

---

## 7. Next Steps

- Read the [Language Fundamentals](language.md) guide
- Explore the [Built-ins Reference](builtins.md) for the complete API
- Follow the [Examples](examples.md) to build real drivers and kernels
- Learn how to write [Interrupt Handlers](functions.md#naked-fun) with `@naked fun`
- Understand [Memory-Mapped I/O](memory.md) patterns
