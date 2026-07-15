# Snask Baremetal — Language Fundamentals

> Types, variables, control flow, operators, and syntax for baremetal programming.

---

## 1. Types

### 1.1 Integer Types

Snask provides fixed-width integers that map directly to LLVM/CPU types:

| Type | Width | Signed | LLVM Type | C Equivalent |
|------|-------|--------|-----------|-------------|
| `I8`  | 8 bit  | Yes | `i8`  | `int8_t` |
| `I16` | 16 bit | Yes | `i16` | `int16_t` |
| `I32` | 32 bit | Yes | `i32` | `int32_t` |
| `I64` | 64 bit | Yes | `i64` | `int64_t` |
| `U8`  | 8 bit  | No  | `i8`  | `uint8_t` |
| `U16` | 16 bit | No  | `i16` | `uint16_t` |
| `U32` | 32 bit | No  | `i32` | `uint32_t` |
| `U64` | 64 bit | No  | `i64` | `uint64_t` |
| `Isize` | native | Yes | `i64`/`i32` | `intptr_t` |
| `Usize` | native | No  | `i64`/`i32` | `uintptr_t` |

**Literal defaults:** integer literals without explicit type default to `I64` in raw mode.

```snask
@raw fun demo()
    let a = 42          // I64
    let b: U8 = 255     // explicit U8
    let c: I32 = -1     // explicit I32
    let d: Usize = 0x1000  // pointer-sized
```

### 1.2 Boolean Type

| Type | Values |
|------|--------|
| `Bool` | `true`, `false` |

Booleans are 1-bit integers in LLVM.

```snask
@raw fun check() : Bool
    return true
```

### 1.3 Float Types

| Type | Width | LLVM Type | C Equivalent |
|------|-------|-----------|-------------|
| `F32` | 32 bit | `float` | `float` |
| `F64` | 64 bit | `double` | `double` |

**Note:** Floating-point in kernel code requires saving/restoring FPU/SSE state
in interrupt handlers. For kernel code, prefer integer operations.

### 1.4 Pointer Type

| Type | Width | Description |
|------|-------|-------------|
| `ptr` | native (8 bytes on x86_64) | Opaque pointer, equivalent to `void*` |

Pointers do not carry pointee type information — they are raw addresses.

```snask
@raw fun demo()
    let p: ptr = @inttoptr(0xB8000, ptr)   // integer → pointer
    let addr: U64 = @ptrtoint(p, U64)       // pointer → integer
```

### 1.5 Void Type

| Type | Description |
|------|-------------|
| `Void` | No value, used for procedures that don't return |

```snask
@raw fun hlt_loop() : Void
    loop
        @asm("hlt")
```

If a `@raw fun` has no return type annotation, it defaults to `Void`:

```snask
@raw fun no_return()
    @store(0xB8000, 0x0F4F)
```

### 1.6 Struct Types

See [Structs & Layout](structs.md).

```snask
struct Registers
    volatile rax: U64
    volatile rbx: U64
    volatile rcx: U64
    volatile rdx: U64
```

### 1.7 Volatile Type Wrapper

The `volatile` keyword on struct members tells the compiler to always emit
memory accesses and never optimize them away — essential for MMIO registers.

```snask
struct UART
    volatile data: U8     // Data register (R/W)
    volatile status: U8   // Status register (read-only)
```

---

## 2. Variables

### 2.1 Immutable (`let`)

```snask
let x: I32 = 10
let name = "irrelevant in baremetal"   // strings need runtime
```

In baremetal, `let` is useful only for scalar types. Strings require heap allocation.

### 2.2 Mutable (`mut`)

```snask
mut counter: U64 = 0
counter = counter + 1
```

### 2.3 Constants (`const`)

```snask
const VGA_ADDR: U64 = 0xB8000
const WHITE_ON_BLACK: U16 = 0x0F00
```

`const` values are **inlined at compile time** — they do not occupy memory.

### 2.4 Globals

In baremetal mode, all top-level `let`, `mut`, and `const` declarations become
**linker-level globals**. They are initialized by the auto-generated
`__snask_init` function before your entry point runs.

```snask
mut tick_count: U64 = 0       // BSS global
mut vga_cursor: U16 = 0       // BSS global
const KERNEL_BASE: U64 = 0x100000  // compile-time constant

@raw fun kmain() : I32
    tick_count = 1
    return 0
```

Generated LLVM-IR for globals:

```llvm
@g_tick_count = global i64 0
@g_vga_cursor = global i16 0

define void @__snask_init() {
  store i64 0, ptr @g_tick_count
  store i16 0, ptr @g_vga_cursor
  ret void
}
```

---

## 3. Control Flow

### 3.1 Conditional (`if` / `elif` / `else`)

```snask
@raw fun check_irq(n: I32) : Bool
    if n < 32
        return true     // CPU exception
    elif n < 48
        return false    // PIC IRQ
    else
        return true     // software interrupt
```

Parentheses are not required. The condition must evaluate to `Bool`.

### 3.2 Infinite Loop (`loop`)

```snask
@raw fun halt_forever()
    loop
        @asm("hlt")
```

`loop` creates an infinite loop — equivalent to `while true`.

### 3.3 Conditional Loop (`while`)

```snask
@raw fun spin_lock(lock: ptr)
    while @volatile(lock, U8) != 0
        @asm("pause")
```

### 3.4 Counted Loop (`for`)

```snask
@raw fun clear_screen()
    let vga: ptr = @inttoptr(0xB8000, ptr)
    for i in 0..(25 * 80)
        mem_write_u16(vga, i * 2, 0x0F20)
```

### 3.5 Return (`return`)

```snask
@raw fun add(a: I32, b: I32) : I32
    return a + b
```

For `@raw fun`, the return value is passed directly in a CPU register.
For `@naked fun`, the body must handle its own return via inline assembly.

---

## 4. Operators

### 4.1 Arithmetic

| Operator | Operation | Example |
|----------|-----------|---------|
| `+` | Add | `a + b` |
| `-` | Subtract | `a - b` |
| `*` | Multiply | `a * b` |
| `/` | Divide | `a / b` |
| `%` | Modulo | `a % b` |

### 4.2 Bitwise

| Operator | Operation | Example |
|----------|-----------|---------|
| `&` | Bitwise AND | `flags & 0xFF` |
| `\|` | Bitwise OR | `flags \| 0x80` |
| `^` | Bitwise XOR | `a ^ b` |
| `<<` | Shift left | `1 << 5` |
| `>>` | Shift right | `val >> 8` |
| `~` | Bitwise NOT | `~mask` |

### 4.3 Comparison

| Operator | Operation | Example |
|----------|-----------|---------|
| `==` | Equal | `a == b` |
| `!=` | Not equal | `a != 0` |
| `<` | Less than | `n < 32` |
| `>` | Greater than | `x > 0` |
| `<=` | Less or equal | `val <= 255` |
| `>=` | Greater or equal | `size >= 4096` |

### 4.4 Logical

| Operator | Operation | Example |
|----------|-----------|---------|
| `and` | Logical AND | `a > 0 and b > 0` |
| `or` | Logical OR | `err == 0 or retry` |
| `not` | Logical NOT | `not ready` |

### 4.5 Assignment

| Operator | Description | Example |
|----------|-------------|---------|
| `=` | Simple assignment | `counter = 0` |

Snask does **not** have compound assignment operators (`+=`, `-=`, etc.)
in the parsed grammar. Use the long form:

```snask
counter = counter + 1
```

---

## 5. Scoping & Comments

### 5.1 Block Scoping

Variables declared with `let`, `mut`, or `const` are scoped to the enclosing block:

```snask
@raw fun demo()
    let a: I32 = 1
    if true
        let b: I32 = 2
        // b is visible here
    // b is NOT visible here
```

### 5.2 Comments

```snask
// This is a single-line comment
let x: I32 = 1  // inline comment

// Multi-line comments use the same syntax
// Every line starts with #
```

---

## 6. Integer Literals

| Format | Prefix | Example |
|--------|--------|---------|
| Decimal | — | `42` |
| Hexadecimal | `0x` | `0xB8000` |
| Binary | `0b` | `0b10101010` |
| Octal | `0o` | `0o755` |
| Character | `'` | `'A'` (ASCII value) |

---

## 7. Best Practices

### DO
- Use `@raw fun` for all kernel functions (avoids boxing overhead)
- Use explicit type annotations for function parameters and returns
- Use `const` for fixed hardware addresses and masks
- Use `mut` sparingly — prefer immutable `let` where possible
- Document the expected calling convention for `@extern fun`

### DON'T
- Use `print()` or any stdio function (not available in baremetal)
- Rely on heap allocation (strings, lists, dicts need runtime)
- Forget to save/restore registers in `@naked fun` interrupt handlers
- Use floating-point in interrupt handlers without saving FPU state
