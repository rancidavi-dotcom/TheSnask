# Snask Baremetal — Built-ins Reference

> Complete reference of all systems-level built-in functions available
> in baremetal mode.

---

## 1. Type Conversion

Convert between integer widths and signedness. The value is truncated
or zero/sign-extended as appropriate.

```snask
as_u8(val)     // → U8   (truncates to 8 bits)
as_u16(val)    // → U16  (truncates to 16 bits)
as_u32(val)    // → U32  (truncates to 32 bits)
as_u64(val)    // → U64  (zero-extends)
as_i8(val)     // → I8   (truncates to 8 bits)
as_i16(val)    // → I16  (truncates to 16 bits)
as_i32(val)    // → I32  (truncates to 32 bits)
as_i64(val)    // → I64  (sign-extends)
as_usize(val)  // → Usize (native pointer width)
as_isize(val)  // → Isize (native pointer width)
as_ptr(val)    // → ptr   (integer to pointer)
```

**Example:**

```snask
@raw fun demo()
    let big: U64 = 0xDEADBEEF_CAFEBABE
    let small: U8 = as_u8(big)       // 0xBE
    let extended: U64 = as_u64(small) // 0xBE (zero-extended)
    let sign_ext: I64 = as_i64(as_i8(small))  // 0xFFFFFFFF_FFFFFFBE
```

---

## 2. Pointer Utilities

```snask
null_ptr()             // → ptr          — returns null pointer
ptr_add(ptr, offset)   // → ptr          — ptr + offset bytes
```

**Example:**

```snask
@raw fun memchr(start: ptr, byte: U8, len: U64) : ptr
    for i in 0..len
        if mem_read_u8(start, i) == byte
            return ptr_add(start, i)
    return null_ptr()
```

---

## 3. Raw Memory Access

```snask
mem_read_u8(ptr, offset)   // → U8    — read byte
mem_read_u16(ptr, offset)  // → U16   — read 16-bit little-endian
mem_read_u32(ptr, offset)  // → U32   — read 32-bit little-endian

mem_write_u8(ptr, offset, val: U8)    // write byte
mem_write_u16(ptr, offset, val: U16)  // write 16-bit
mem_write_u32(ptr, offset, val: U32)  // write 32-bit

mem_fill_u8(ptr, val: U8, len: U64)   // memset: fill memory with byte value
mem_copy(dst: ptr, src: ptr, len: U64) // memcpy: copy memory region
```

**Example:**

```snask
@raw fun clear_bss(start: ptr, end: ptr)
    let len: U64 = @ptrtoint(end, U64) - @ptrtoint(start, U64)
    mem_fill_u8(start, 0, len)

@raw fun smap_copy(dst: ptr, src: ptr, entries: U64)
    mem_copy(dst, src, entries * 24)  // copy SMAP entries
```

---

## 4. Heap Allocation

Requires providing `malloc`/`calloc`/`free` symbols at link time.

```snask
mem_alloc(size: U64)       // → ptr   — allocate (malloc)
mem_alloc_zero(size: U64)  // → ptr   — allocate zeroed (calloc)
mem_free(ptr)                        — deallocate (free)
```

---

## 5. Bit Manipulation

### 5.1 Byte Extraction / Construction

```snask
lo_u8(val)       // → U8   — low byte of U16
hi_u8(val)       // → U8   — high byte of U16
make_u16(lo, hi) // → U16  — construct U16 from two bytes
```

**Example:**

```snask
let word: U16 = 0xABCD
lo_u8(word)        // 0xCD
hi_u8(word)        // 0xAB
make_u16(0xCD, 0xAB)  // 0xABCD
```

### 5.2 Predicates

```snask
is_zero_u8(val)       // → Bool — true if val == 0
is_negative_u8(val)   // → Bool — true if bit 7 set
```

### 5.3 Bit Test / Set / Clear

```snask
bit_test(val, bit)        // → Bool — test bit at position
bit_set(val, bit)         // → val  — set bit
bit_clear(val, bit)       // → val  — clear bit
bit_toggle(val, bit)      // → val  — toggle bit
bit_write(val, bit, enabled)  // → val  — set if enabled, clear otherwise
```

### 5.4 Flag Operations (aliases)

```snask
flag_has(val, bit)        // ≡ bit_test
flag_set(val, bit)        // ≡ bit_set
flag_clear(val, bit)      // ≡ bit_clear
flag_write(val, bit, enabled) // ≡ bit_write
```

**Example:**

```snask
@raw fun pic_mask(irq: U8) : U8
    let imr: U8 = inb(0x21)
    return bit_set(imr, irq)   // set bit to mask IRQ

@raw fun pic_unmask(imr: U8, irq: U8) : U8
    return bit_clear(imr, irq)
```

---

## 6. Integer Arithmetic

### 6.1 Wrapping Arithmetic

Overflow wraps around (two's complement).

```snask
wrapping_add(a, b)   // a + b (wrapping)
wrapping_sub(a, b)   // a - b (wrapping)
wrapping_mul(a, b)   // a * b (wrapping)
wrapping_inc(a)      // a + 1
wrapping_dec(a)      // a - 1
```

### 6.2 Saturating Arithmetic

Clamps to min/max instead of overflowing.

```snask
saturating_add(a, b)  // a + b (saturating)
```

### 6.3 Carry / Borrow / Overflow Detection

```snask
carry_add_u8(a, b, carry_in)       // → U8 — carry out from u8 addition
borrow_sub_u8(a, b, borrow_in)     // → U8 — borrow out from u8 subtraction
overflow_add_i8(a, b, overflow_in) // → U8 — overflow from i8 addition
overflow_sub_i8(a, b, overflow_in) // → U8 — overflow from i8 subtraction
```

**Example — Multi-precision Arithmetic:**

```snask
@raw fun add_u128(result: ptr, a: ptr, b: ptr)
    let lo_a: U64 = mem_read_u64(a, 0)
    let lo_b: U64 = mem_read_u64(b, 0)
    let (lo_sum, carry) = wrapping_add_carry(lo_a, lo_b)  // pseudo
    mem_write_u64(result, 0, lo_sum)

    let hi_a: U64 = mem_read_u64(a, 8)
    let hi_b: U64 = mem_read_u64(b, 8)
    let hi_sum: U64 = hi_a + hi_b + carry
    mem_write_u64(result, 8, hi_sum)
```

---

## 7. Port I/O (x86)

```snask
@outb(port: U16, val: U8)   // write byte to I/O port
inb(port: U16) → U8          // read byte from I/O port
```

These are **statement-level** (`@outb`) and **expression-level** (`inb`).

---

## 8. Volatile Memory

```snask
@store(address: U64/ptr, value: any)     // volatile write
@volatile(address: U64/ptr, Type) → Type // volatile read
@write(address: U64/ptr, value, Type)    // typed volatile write
```

---

## 9. Pointer Conversion

```snask
@inttoptr(integer, Type) → Type    // integer → pointer
@ptrtoint(pointer, Type) → Type    // pointer → integer
```

---

## 10. Struct Layout (compile-time)

```snask
sizeof(Type)     → U64  — size in bytes
alignof(Type)    → U64  — alignment in bytes
offsetof(type, field) → U64  — member offset in bytes
```

---

## 11. Functions Quick Reference

| Function | Signature | Returns |
|----------|-----------|---------|
| `as_u8` | `(val)` | `U8` |
| `as_u16` | `(val)` | `U16` |
| `as_u32` | `(val)` | `U32` |
| `as_u64` | `(val)` | `U64` |
| `as_i8` | `(val)` | `I8` |
| `as_i16` | `(val)` | `I16` |
| `as_i32` | `(val)` | `I32` |
| `as_i64` | `(val)` | `I64` |
| `as_usize` | `(val)` | `Usize` |
| `as_isize` | `(val)` | `Isize` |
| `as_ptr` | `(val)` | `ptr` |
| `null_ptr` | `()` | `ptr` |
| `ptr_add` | `(ptr, offset: U64)` | `ptr` |
| `mem_read_u8` | `(ptr, offset: U64)` | `U8` |
| `mem_read_u16` | `(ptr, offset: U64)` | `U16` |
| `mem_read_u32` | `(ptr, offset: U64)` | `U32` |
| `mem_write_u8` | `(ptr, offset: U64, val: U8)` | `Void` |
| `mem_write_u16` | `(ptr, offset: U64, val: U16)` | `Void` |
| `mem_write_u32` | `(ptr, offset: U64, val: U32)` | `Void` |
| `mem_fill_u8` | `(ptr, val: U8, len: U64)` | `Void` |
| `mem_copy` | `(dst: ptr, src: ptr, len: U64)` | `Void` |
| `mem_alloc` | `(size: U64)` | `ptr` |
| `mem_alloc_zero` | `(size: U64)` | `ptr` |
| `mem_free` | `(ptr)` | `Void` |
| `lo_u8` | `(val: U16)` | `U8` |
| `hi_u8` | `(val: U16)` | `U8` |
| `make_u16` | `(lo: U8, hi: U8)` | `U16` |
| `is_zero_u8` | `(val: U8)` | `Bool` |
| `is_negative_u8` | `(val: U8)` | `Bool` |
| `bit_test` | `(val, bit)` | `Bool` |
| `bit_set` | `(val, bit)` | Same as val |
| `bit_clear` | `(val, bit)` | Same as val |
| `bit_toggle` | `(val, bit)` | Same as val |
| `bit_write` | `(val, bit, enabled)` | Same as val |
| `flag_has` | `(val, bit)` | `Bool` |
| `flag_set` | `(val, bit)` | Same as val |
| `flag_clear` | `(val, bit)` | Same as val |
| `flag_write` | `(val, bit, enabled)` | Same as val |
| `wrapping_add` | `(a, b)` | Same as inputs |
| `wrapping_sub` | `(a, b)` | Same as inputs |
| `wrapping_mul` | `(a, b)` | Same as inputs |
| `wrapping_inc` | `(a)` | Same as input |
| `wrapping_dec` | `(a)` | Same as input |
| `saturating_add` | `(a, b)` | Same as inputs |
| `carry_add_u8` | `(a: U8, b: U8, c: U8)` | `U8` |
| `borrow_sub_u8` | `(a: U8, b: U8, c: U8)` | `U8` |
| `overflow_add_i8` | `(a: I8, b: I8, c: I8)` | `U8` |
| `overflow_sub_i8` | `(a: I8, b: I8, c: I8)` | `U8` |

---

## 12. Compile-Time Constants Visible to the Compiler

The baremetal profile automatically defines:

| Symbol | Value | Description |
|--------|-------|-------------|
| `__snask_baremetal__` | `1` | Set when baremetal profile is active |

You can use this for conditional compilation via `const`:

```snask
const IS_BAREMETAL: I32 = __snask_baremetal__
```
