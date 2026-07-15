# Snask Baremetal — Structs & Data Layout

> Defining structured data types, querying layout properties, and
> memory-mapped register maps.

---

## 1. Struct Declaration

### Syntax

```snask
struct Name
    member1: Type1
    volatile member2: Type2
    member3: Type3
```

- Members are sequential in memory
- No padding between members by default
- `volatile` keyword marks a member for non-optimized access

### Simple Example

```snask
struct Point
    x: I32
    y: I32

@raw fun origin() : I32
    let p: Point
    p.x = 0
    p.y = 0
    return p.x + p.y
```

### Access

```snask
@raw fun demo()
    let p: Point
    p.x = 10
    p.y = 20
    @store(0xB8000, p.x)
```

---

## 2. Volatile Members

Members declared with `volatile` are accessed using LLVM volatile
load/store instructions — the compiler will never optimize away
or reorder these accesses.

```snask
struct UART_MMIO
    volatile data: U8       // R/W: data register
    volatile ier: U8        // R/W: interrupt enable
    volatile iir: U8        // R: interrupt identification (read-only!)
    volatile lcr: U8        // R/W: line control
    volatile mcr: U8        // R/W: modem control
    volatile lsr: U8        // R: line status
    volatile msr: U8        // R: modem status
```

All MMIO register maps should use `volatile` on every member.

---

## 3. Layout Queries

These built-in functions query struct layout at compile time. They
compile to **constants** — zero runtime overhead.

### 3.1 `sizeof(Type)`

Returns the size of a type in bytes.

```snask
let sz: U64 = sizeof(Point)        // 8 (two I32s)
let sz2: U64 = sizeof(UART_MMIO)   // 7 (seven U8s)
let sz3: U64 = sizeof(U64)         // 8
let sz4: U64 = sizeof(ptr)         // 8 (on x86_64)
```

### 3.2 `alignof(Type)`

Returns the alignment requirement of a type.

```snask
let al: U64 = alignof(Point)        // 4 (max member alignment)
let al2: U64 = alignof(I32)         // 4
let al3: U64 = alignof(U64)         // 8
let al4: U64 = alignof(ptr)         // 8
```

### 3.3 `offsetof(StructType, member)`

Returns the byte offset of a member from the start of the struct.

```snask
let off: U64 = offsetof(UART_MMIO, lsr)   // 5
let off2: U64 = offsetof(Point, y)         // 4
```

### Example — Manual MMIO Access via Offsets

```snask
struct COM_Regs
    volatile data: U8       // +0
    volatile ier: U8        // +1
    volatile iir: U8        // +2
    volatile lcr: U8        // +3
    volatile mcr: U8        // +4
    volatile lsr: U8        // +5
    volatile msr: U8        // +6

const COM1_BASE: U64 = 0x3F8

@raw fun uart_read(reg: U64) : U8
    return @volatile(COM1_BASE + reg, U8)

@raw fun uart_write(reg: U64, val: U8)
    @store(COM1_BASE + reg, val)
```

---

## 4. Member Access in Raw Functions

In raw mode, struct members are accessed by their offset within the struct.
The compiler translates `p.x` to a GEP (GetElementPtr) instruction with
the correct offset.

```snask
struct PageTableEntry
    volatile value: U64

@raw fun set_page_entry(table: ptr, index: U64, value: U64)
    let entry: ptr = ptr_add(table, index * 8)
    @store(@ptrtoint(entry, U64), value)
```

---

## 5. Limitations

| Feature | Status | Notes |
|---------|--------|-------|
| Default member values | ❌ | Not supported |
| Nested structs | ✅ | Can contain other structs |
| Array members | ❌ | Not in parsed grammar |
| Union/anonymous | ❌ | Use `@global_asm` for overlays |
| Bit fields | ❌ | Use bit operations manually |
| Packed/alignment | ❌ | Members are tightly packed |
| `repr(C)` | ❌ | Not explicitly, but `@extern fun` uses C ABI |

---

## 6. Struct Layout in Memory

```snask
struct Registers
    volatile rax: U64      // offset 0
    volatile rbx: U64      // offset 8
    volatile rcx: U64      // offset 16
    volatile rdx: U64      // offset 24
```

Memory layout:
```
Offset:  0   8   16  24
        ┌───┬───┬───┬───┐
        │RAX│RBX│RCX│RDX│
        └───┴───┴───┴───┘
```

---

## 7. Practical: IDT Entry

```snask
struct IDT_Entry
    volatile offset_low: U16
    volatile selector: U16
    volatile ist: U8
    volatile flags: U8
    volatile offset_mid: U16
    volatile offset_high: U32
    volatile zero: U32

@raw fun set_idt_entry(table: ptr, index: U64, handler: U64, selector: U16, flags: U8)
    let base: U64 = @ptrtoint(table, U64)
    let entry_base: U64 = base + index * sizeof(IDT_Entry)
    let entry: ptr = @inttoptr(entry_base, ptr)

    @store(entry_base + offsetof(IDT_Entry, offset_low), handler and 0xFFFF)
    @store(entry_base + offsetof(IDT_Entry, selector), selector)
    @store(entry_base + offsetof(IDT_Entry, ist), 0)
    @store(entry_base + offsetof(IDT_Entry, flags), flags)
    @store(entry_base + offsetof(IDT_Entry, offset_mid), (handler >> 16) and 0xFFFF)
    @store(entry_base + offsetof(IDT_Entry, offset_high), (handler >> 32) and 0xFFFFFFFF)
    @store(entry_base + offsetof(IDT_Entry, zero), 0)
```
