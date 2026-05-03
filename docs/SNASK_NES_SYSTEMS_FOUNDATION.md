# Snask NES Systems Foundation

This document describes the low-level Snask builtins intended to support a real NES emulator core.

No new syntax is introduced here. Everything is normal Snask function-call syntax, so existing language grammar, OM, zones, profiles and diagnostics remain intact.

## Raw Memory

Raw memory is manual memory. These functions must be used inside `@unsafe`.

```snask
@unsafe:
    let mem: ptr = mem_alloc_zero(65536)
    mem_write_u8(mem, 0xFFFC, 0x00)
    mem_write_u8(mem, 0xFFFD, 0x80)
    let pc: u16 = mem_read_u16(mem, 0xFFFC)
    mem_free(mem)
```

Builtins:

- `mem_alloc(size) -> ptr`
- `mem_alloc_zero(size) -> ptr`
- `mem_free(ptr) -> void`
- `ptr_add(ptr, offset) -> ptr`
- `mem_read_u8(ptr, offset) -> u8`
- `mem_read_u16(ptr, offset) -> u16`
- `mem_read_u32(ptr, offset) -> u32`
- `mem_write_u8(ptr, offset, value) -> void`
- `mem_write_u16(ptr, offset, value) -> void`
- `mem_write_u32(ptr, offset, value) -> void`
- `mem_fill_u8(ptr, value, len) -> void`
- `mem_copy(dst, src, len) -> void`

Multi-byte memory helpers use little-endian order, which matches the 6502/NES bus model.

## Integer Conversion

These helpers preserve explicit fixed-width intent:

- `as_u8(x) -> u8`
- `as_u16(x) -> u16`
- `as_u32(x) -> u32`
- `as_u64(x) -> u64`
- `as_i8(x) -> i8`
- `as_i16(x) -> i16`
- `as_i32(x) -> i32`
- `as_i64(x) -> i64`
- `as_usize(x) -> usize`
- `as_isize(x) -> isize`

## 6502 Word Helpers

- `lo_u8(word) -> u8`
- `hi_u8(word) -> u8`
- `make_u16(lo, hi) -> u16`

These are useful for reset vectors, interrupt vectors and zero-page indirect addressing.

## Bit And Flag Helpers

- `bit_test(value, bit) -> bool`
- `bit_set(value, bit) -> same integer type`
- `bit_clear(value, bit) -> same integer type`
- `bit_toggle(value, bit) -> same integer type`
- `bit_write(value, bit, enabled) -> same integer type`
- `flag_has(flags, bit) -> bool`
- `flag_set(flags, bit) -> same integer type`
- `flag_clear(flags, bit) -> same integer type`
- `flag_write(flags, bit, enabled) -> same integer type`
- `is_zero_u8(value) -> bool`
- `is_negative_u8(value) -> bool`

These directly support the 6502 status register:

```text
N V - B D I Z C
7 6 5 4 3 2 1 0
```

## Overflow And Carry

- `wrapping_add(a, b)`
- `wrapping_sub(a, b)`
- `wrapping_mul(a, b)`
- `wrapping_inc(x)`
- `wrapping_dec(x)`
- `saturating_add(a, b)`
- `carry_add_u8(a, b, carry_in) -> bool`
- `borrow_sub_u8(a, b, borrow_in) -> bool`
- `overflow_add_i8(a, b, carry_in) -> bool`
- `overflow_sub_i8(a, b, borrow_in) -> bool`

These are the foundation for correct ADC/SBC and branch flag behavior. They do not implement the CPU for you; they expose deterministic primitive operations needed to implement it faithfully in Snask.

## NES Direction

With these primitives, a Snask NES CPU core can model:

- a full 64KB bus;
- little-endian vector reads;
- fixed-width register values;
- status flags;
- carry/borrow/overflow behavior;
- deterministic instruction helpers;
- raw memory operations isolated in `@unsafe`.

The next major foundations are fixed arrays/slices or a higher-level safe bus abstraction over these raw memory primitives, then a complete 6502 opcode table implemented in pure Snask.
