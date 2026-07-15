# Snask Baremetal — Inline Assembly & Port I/O

> Inline assembly statements, module-level assembly, and x86 port I/O operations.

---

## 1. `@asm("instructions")`

Embeds raw assembly instructions at the current point in the function body.

### Syntax

```snask
@asm("instruction1")
@asm("instruction2; instruction3")
```

Each `@asm` call produces a separate LLVM inline assembly blob. Multiple
instructions can be chained with `;` or `\n` inside one string.

### Rules

| Property | Value |
|----------|-------|
| Side effects | Always `sideeffect` (not optimized away) |
| Constraints | Empty string (no register bindings) |
| Stack alignment | Not adjusted |
| Can throw | No (no exception support) |
| Dialect | AT&T (default) |

### Examples

```snask
// Single instruction
@asm("hlt")

// Multiple instructions
@asm("cli; mov rsp, stack_top")

// Context-switch (paging)
@asm("mov cr3, rax")

// Interrupt return
@asm("iretq")

// CPU identification
@raw fun cpuid(eax_in: U32, ecx_in: U32) : U64
    @asm("cpuid")
    // Result is in RAX (combined eax:edx)
    return 0   // placeholder — cpuid result handled manually
```

### In `@naked fun`

`@asm` is the only way to return from a naked function:

```snask
@naked fun isr_timer()
    @asm("push rax")
    @asm("push rcx")
    @asm("push rdx")
    @asm("call timer_handler")
    @asm("pop rdx")
    @asm("pop rcx")
    @asm("pop rax")
    @asm("iretq")
```

---

## 2. `@global_asm("instructions")`

Embeds assembly at the **module level**, outside any function.
Used for:

- Multiboot headers
- Global descriptor table (GDT) data
- Interrupt vector table (IVT) data
- Constants and data structures
- Assembly functions

### Syntax

```snask
@global_asm("directive_or_instruction")
```

Each call appends to a module-level assembly buffer that gets emitted
as LLVM module-level inline assembly.

### Examples

#### Multiboot Header

```snask
@global_asm(".section .multiboot")
@global_asm(".align 4")
@global_asm(".long 0x1BADB002")     // magic number
@global_asm(".long 0x00000000")     // flags
@global_asm(".long -(0x1BADB002)")  // checksum
```

#### GDT in Assembly

```snask
@global_asm(".align 8")
@global_asm("gdt64:")
@global_asm(".quad 0x0000000000000000")  // null descriptor
@global_asm(".quad 0x00209A0000000000")  // code segment
@global_asm(".quad 0x0000920000000000")  // data segment
@global_asm("gdt64_ptr:")
@global_asm(".word gdt64_ptr - gdt64 - 1")
@global_asm(".quad gdt64")
```

#### IDT Entry Assembly Constant

```snask
@global_asm(".macro make_idt_entry vector, handler, selector, flags")
@global_asm("  .quad handler")
@global_asm("  .word selector")
@global_asm("  .byte 0")
@global_asm("  .byte flags")
@global_asm(".endm")
```

---

## 3. Port I/O (x86)

### 3.1 `@outb(port, value)`

Write a byte to an x86 I/O port.

```snask
@outb(0x70, 0x0A)     // select CMOS register A
@outb(0x71, 0x20)     // read from CMOS
@outb(0x21, 0xFC)     // mask PIC interrupts (IRQ 0-1 only)
@outb(0xA1, 0xFF)     // mask slave PIC (all IRQs)
@outb(0x20, 0x20)     // EOI to master PIC
@outb(0xA0, 0x20)     // EOI to slave PIC
```

**Parameters:**
- `port` — 16-bit I/O port address (`U16`)
- `value` — 8-bit value to write (`U8`)

**Generated code:**
```asm
mov al, byte [value]
mov dx, word [port]
out dx, al
```

### 3.2 `inb(port)`

Read a byte from an x86 I/O port.

```snask
let cmos_status: U8 = inb(0x71)
let keyboard_ack: U8 = inb(0x60)
let isr_number: U8 = inb(0x20)   // read PIC ISR register
```

**Parameter:**
- `port` — 16-bit I/O port address (`U16`)

**Returns:**
- `U8` — the value read from the port

**Generated code:**
```asm
mov dx, word [port]
in al, dx
```

### CMOS/RTC Example

```snask
const CMOS_ADDR: U16 = 0x70
const CMOS_DATA: U16 = 0x71

@raw fun read_cmos(reg: U8) : U8
    @outb(CMOS_ADDR, reg)
    @asm("nop")                    // small delay for CMOS
    @asm("nop")
    return inb(CMOS_DATA)

@raw fun read_rtc_second() : U8
    let bcd: U8 = read_cmos(0x00)
    return (bcd >> 4) * 10 + (bcd & 0x0F)   // BCD to binary
```

### PIC Programming Example

```snask
const PIC1_COMMAND: U16 = 0x20
const PIC1_DATA: U16 = 0x21
const PIC2_COMMAND: U16 = 0xA0
const PIC2_DATA: U16 = 0xA1

const ICW1_ICW4: U8 = 0x11    // ICW4 needed
const ICW1_INIT: U8 = 0x10    // initialization

@raw fun pic_remap(offset1: U8, offset2: U8)
    @outb(PIC1_COMMAND, ICW1_INIT or ICW1_ICW4)
    @asm("nop")
    @outb(PIC2_COMMAND, ICW1_INIT or ICW1_ICW4)
    @asm("nop")
    @outb(PIC1_DATA, offset1)     // master offset
    @asm("nop")
    @outb(PIC2_DATA, offset2)     // slave offset
    @asm("nop")
    @outb(PIC1_DATA, 4)           // tell master there's a slave at IRQ2
    @asm("nop")
    @outb(PIC2_DATA, 2)           // tell slave its cascade identity
    @asm("nop")
    @outb(PIC1_DATA, 0x01)        // 8086 mode
    @asm("nop")
    @outb(PIC2_DATA, 0x01)        // 8086 mode
    @asm("nop")
    @outb(PIC1_DATA, 0xFF)        // mask all
    @outb(PIC2_DATA, 0xFF)        // mask all
```

---

## 4. Common Assembly Patterns

### 4.1 CPU Control

```snask
@asm("cli")               // disable interrupts
@asm("sti")               // enable interrupts
@asm("hlt")               // halt CPU
@asm("pause")              // spin-loop hint (x86)
@asm("nop")               // no-op / delay
```

### 4.2 Control Register Access

```snask
@raw fun read_cr0() : U64
    @asm("mov rax, cr0")

@raw fun write_cr0(val: U64)
    @asm("mov cr0, rax")

@raw fun read_cr2() : U64   // page fault address
    @asm("mov rax, cr2")

@raw fun read_cr3() : U64   // page table base
    @asm("mov rax, cr3")

@raw fun write_cr3(val: U64)
    @asm("mov cr3, rax")
```

### 4.3 Segment Register Access

```snask
@raw fun load_gdt(gdt_ptr: ptr)
    @asm("lgdt [rdi]")
    @asm("push 0x08")
    @asm("lea rax, [rip + .reload]")
    @asm("push rax")
    @asm("retfq")
    @asm(".reload:")
    @asm("mov ax, 0x10")
    @asm("mov ds, ax")
    @asm("mov es, ax")
    @asm("mov fs, ax")
    @asm("mov gs, ax")
    @asm("mov ss, ax")
```

### 4.4 TSS and Task Management

```snask
@raw fun load_tss(tss_ptr: ptr)
    @asm("ltr [rdi]")
```

---

## 5. LLVM Inline Assembly Details

Each `@asm("...")` call generates an LLVM `call void asm sideeffect`:

```llvm
call void asm sideeffect "hlt", ""()
```

The empty constraint string `""` means no input or output operands —
all registers must be managed manually via push/pop or dedicated load/store.

### Manual Register Management

```snask
@raw fun read_msr(msr: U32) : U64
    // ECX = msr, result in RDX:RAX
    @asm("mov ecx, [rbp-4]")       // load msr from stack
    @asm("rdmsr")
    @asm("shl rdx, 32")
    @asm("or rax, rdx")
    return 0                        // placeholder
```

---

## 6. Architecture Support

| Feature | x86_64 | ARM64 | RISC-V |
|---------|--------|-------|--------|
| `@asm("...")` | ✅ | ✅ | ✅ |
| `@global_asm("...")` | ✅ | ✅ | ✅ |
| `@outb(port, val)` | ✅ | ❌ | ❌ |
| `inb(port)` | ✅ | ❌ | ❌ |
| MMIO `@volatile`/`@store` | ✅ | ✅ | ✅ |

Port I/O (`@outb`/`inb`) is x86-specific. ARM64 and RISC-V use memory-mapped I/O
exclusively via `@store`/`@volatile`.
