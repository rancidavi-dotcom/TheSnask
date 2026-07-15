# Snask Baremetal — Memory Operations

> Reading and writing memory directly, volatile access, pointer manipulation,
> and raw memory intrinsics.

---

## 1. Volatile Memory Access

The compiler **must not optimize away or reorder** these accesses.
Essential for memory-mapped I/O (MMIO) registers.

### 1.1 `@store(address, value)`

Write a value to a memory address. The operation is always volatile.

```snask
@store(0xB8000, 0x0F4F)             // write U16 to VGA buffer
@store(uart_base + 0, 0x48)         // write 'H' to UART data reg
```

**Parameters:**
- `address` — integer (will be cast to `ptr`)
- `value` — the value to write (type determines store width)

**Generated code (x86_64):**
```asm
mov word [0xB8000], 0x0F4F
```

### 1.2 `@volatile(address, Type)`

Read a value from a memory address. The operation is always volatile.

```snask
let status: U8 = @volatile(uart_base + 5, U8)   // read UART line status
let scancode: U8 = @volatile(0x60, U8)           // read PS/2 keyboard
```

**Parameters:**
- `address` — integer or `ptr`
- `Type` — the type to read (U8, U16, U32, U64, I32, etc.)

**Generated code (x86_64):**
```asm
movzx eax, byte [0x3F8 + 5]
```

### 1.3 `@write(address, value, Type)`

Typed volatile write. Like `@store`, but with explicit type hint.

```snask
@write(0xB8000, 0x0F4F, U16)       // explicit U16 write
@write(mmio_base, 0x01, U32)        // 32-bit MMIO register write
```

### Example — UART Driver

```snask
const COM1: U64 = 0x3F8

@raw fun uart_init()
    @write(COM1 + 1, 0x00, U8)      // disable interrupts
    @write(COM1 + 3, 0x80, U8)      // enable DLAB
    @write(COM1 + 0, 0x01, U8)      // baud rate divisor (115200)
    @write(COM1 + 1, 0x00, U8)      // baud rate divisor (high)
    @write(COM1 + 3, 0x03, U8)      // 8 bits, no parity, 1 stop
    @write(COM1 + 2, 0xC7, U8)      // FIFO enable, clear, 14 bytes
    @write(COM1 + 4, 0x0B, U8)      // DTR/RTS enabled

@raw fun uart_tx_ready() : Bool
    return (@volatile(COM1 + 5, U8) & 0x20) != 0

@raw fun uart_putc(c: U8)
    while not uart_tx_ready()
        @asm("pause")
    @write(COM1 + 0, c, U8)

@raw fun uart_puts(s: ptr, len: U64)
    for i in 0..len
        uart_putc(mem_read_u8(s, i))
```

---

## 2. Raw Memory Built-ins

These intrinsics operate on raw pointers with no type checking.
They compile to LLVM `load`/`store`/`memcpy`/`memset` instructions.

### 2.1 Read Functions

```snask
let byte: U8   = mem_read_u8(ptr, offset)
let word: U16  = mem_read_u16(ptr, offset)
let dword: U32 = mem_read_u32(ptr, offset)
```

Reads N bytes starting at `ptr + offset`. The pointer is not dereferenced
as a struct — this is a raw byte load.

### 2.2 Write Functions

```snask
mem_write_u8(ptr, offset, val: U8)
mem_write_u16(ptr, offset, val: U16)
mem_write_u32(ptr, offset, val: U32)
```

Writes N bytes starting at `ptr + offset`.

### 2.3 Bulk Memory

```snask
mem_fill_u8(ptr, val: U8, len: U64)     // memset equivalent
mem_copy(dst: ptr, src: ptr, len: U64)   // memcpy equivalent
```

- `mem_fill_u8` compiles to LLVM `llvm.memset` intrinsic
- `mem_copy` compiles to LLVM `llvm.memcpy` intrinsic

```snask
@raw fun zero_page(page: ptr)
    mem_fill_u8(page, 0, 4096)

@raw fun copy_pages(dst: ptr, src: ptr, count: U64)
    mem_copy(dst, src, count * 4096)
```

### 2.4 Heap Allocation

```snask
let buf: ptr = mem_alloc(size: U64)         // malloc
let zeroed: ptr = mem_alloc_zero(size: U64) // calloc
mem_free(ptr)                                // free
```

**Note:** In baremetal mode, these are **external symbols** (`malloc`, `calloc`, `free`)
that you must provide or link against. If you don't have a heap, do not use them.

### 2.5 Pointer Arithmetic

```snask
let next: ptr = ptr_add(base: ptr, offset: U64)
```

Adds `offset` bytes to the pointer. Equivalent to `(void*)((uintptr_t)base + offset)`.

```snask
let null: ptr = null_ptr()
```

Returns a null pointer (all zeros). Useful as a sentinel.

---

## 3. Pointer Conversion

### 3.1 Integer to Pointer (`@inttoptr`)

```snask
@inttoptr(integer, Type) : Type
```

Converts an integer address to a pointer type.

```snask
let vga: ptr = @inttoptr(0xB8000, ptr)
let mmio: ptr = @inttoptr(0xFED00000, ptr)
let idt: ptr = @inttoptr(idt_phys, ptr)
```

**Parameters:**
- `integer` — the numeric address
- `Type` — target pointer type (usually `ptr`)

**Generated code (x86_64):**
```asm
mov rax, 0xB8000           ; no actual instruction needed — just a register move
```

### 3.2 Pointer to Integer (`@ptrtoint`)

```snask
@ptrtoint(pointer, Type) : Type
```

Converts a pointer back to an integer for arithmetic.

```snask
@raw fun page_align_down(addr: ptr) : U64
    let raw: U64 = @ptrtoint(addr, U64)
    return raw & 0xFFFF_FFFF_FFFF_F000
```

### 3.3 Address-of (`&`)

```snask
let ptr_to_x: ptr = &x
```

Takes the address of a local variable. Useful for passing stack buffers
to `@extern` functions or assembly.

```snask
@raw fun read_port_report(port: U16) : U16
    let buf: U16 = 0
    @extern fun read_port(port: U16, buf: ptr)
    read_port(port, &buf)
    return buf
```

### 3.4 Dereference (`*`)

```snask
let val: I32 = *ptr
```

Reads the value pointed to by `ptr`. The type is inferred from context.

```snask
@raw fun is_null(ptr: ptr) : Bool
    let addr: U64 = @ptrtoint(ptr, U64)
    return addr == 0
```

---

## 4. Volatile Struct Members

When a struct field is declared `volatile`, reads and writes to that field
are always treated as volatile accesses by the compiler.

```snask
struct UART_Regs
    volatile data: U8      // data register (R/W)
    volatile ier: U8       // interrupt enable
    volatile iir: U8       // interrupt identification (R)
    volatile lcr: U8       // line control
    volatile mcr: U8       // modem control
    volatile lsr: U8       // line status (R)
    volatile msr: U8       // modem status (R)

@raw fun uart_read_lsr(base: ptr) : U8
    let regs: ptr = @inttoptr(@ptrtoint(base, U64), ptr)
    // Access via struct offset
    return @volatile(@ptrtoint(regs, U64) + 5, U8)
```

---

## 5. Memory Model

| Address Space | Typical Use | Access Method |
|--------------|-------------|---------------|
| `0x00000` – `0x9FFFF` | Conventional memory | `mem_read_*` / `mem_write_*` |
| `0xA0000` – `0xBFFFF` | VGA/EGA video RAM | `@volatile` / `@store` |
| `0xC0000` – `0xFFFFF` | BIOS, option ROMs | `mem_read_*` |
| `0x100000` – ... | Loaded kernel | Direct variable access |
| `0xFED00000` – ... | MMIO (APIC, HPET, PCIe) | `@volatile` / `@store` |

---

## 6. Best Practices

| Do | Don't |
|----|-------|
| Use `@volatile`/`@store` for all hardware registers | Read MMIO registers without `volatile` |
| Use `mem_read_*`/`mem_write_*` for raw memory buffers | Use struct member access for MMIO (structs may be optimized) |
| Use `const` for hardware base addresses | Hard-code magic numbers everywhere |
| Use `@inttoptr` + `@volatile` for known physical addresses | Cast integers to pointers without checking alignment |
