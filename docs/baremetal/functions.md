# Snask Baremetal — Function Types

> The three function types available in baremetal mode and when to use each.

---

## 1. `@raw fun` — Raw (Unboxed) Functions

The workhorse of baremetal programming. Parameters and return values are
passed directly in CPU registers, with zero boxing overhead.

### Syntax

```snask
@raw fun name(param: Type, ...) : ReturnType
    body
```

### Rules

| Property | Behavior |
|----------|----------|
| Parameters | Passed in registers, stored in allocas at entry |
| Return value | Passed in return register |
| Prologue | Standard (saves/frame pointer if needed) |
| Boxing | None — values are LLVM native types |
| Unsafe | Implicitly `unsafe` (can access any memory) |

### Examples

```snask
// Integer arithmetic — params and return in registers
@raw fun div_round(a: I32, b: I32) : I32
    let q: I32 = a / b
    let r: I32 = a % b
    let half_b: I32 = b / 2
    if r >= half_b
        return q + 1
    return q

// MMIO access — raw address arithmetic
@raw fun read_cmos(reg: U8) : U8
    @outb(0x70, reg)
    @asm("nop")
    @asm("nop")
    return inb(0x71)

// Multiple pointer parameters
@raw fun memcpy_32(dst: ptr, src: ptr, count: U64)
    for i in 0..count
        let val: U32 = mem_read_u32(src, i * 4)
        mem_write_u32(dst, i * 4, val)

// Returns void when no return type is specified
@raw fun poke(addr: U64, val: U8)
    @store(addr, val)
```

### Generated Code

```snask
@raw fun add(a: I32, b: I32) : I32
    return a + b
```

Compiles to (x86_64):

```asm
add:
    push   rbp
    mov    rbp, rsp
    mov    dword [rbp-4], edi     // store a
    mov    dword [rbp-8], esi     // store b
    mov    eax, dword [rbp-4]     // load a
    add    eax, dword [rbp-8]     // add b
    pop    rbp
    ret
```

With optimization, the stack operations are eliminated:

```asm
add:
    lea    eax, [rdi + rsi]
    ret
```

---

## 2. `@naked fun` — Naked Functions

Functions with **no prologue or epilogue** at all. The compiler generates
only the instructions you write. You are responsible for:

- Returning via inline assembly (`ret`, `iretq`, `sysexit`, etc.)
- Saving and restoring registers if needed
- Setting up the stack frame if needed

### Syntax

```snask
@naked fun name(param: Type, ...) : ReturnType
    body
```

The body is optional — a naked function can be just a declaration for
an externally-defined handler.

### Rules

| Property | Behavior |
|----------|----------|
| Parameters | In registers as received by the hardware |
| Prologue | **None** — no push, no frame pointer |
| Return | Must be handled via `@asm("...")` |
| Terminator | Compiler injects `unreachable` after the body |

### Critical: No Auto-Return

The compiler adds `unreachable` after the last statement. This means if
you write a naked function that does not end with a return instruction,
LLVM treats the code after the function as dead.

**Always end with a return instruction in `@asm`:**

```snask
@naked fun isr_handler()
    @asm("iretq")           // correct — ends with return
```

### Use Cases

#### 1. Interrupt Service Routines

```snask
@naked fun isr_divide_error()
    // CPU pushed: SS, RSP, RFLAGS, CS, RIP, error_code(0)
    @asm("push rax")
    @asm("push rcx")
    @asm("push rdx")
    @asm("push rbx")
    @asm("push rbp")
    @asm("push rsi")
    @asm("push rdi")
    @asm("cld")                         // set C calling convention
    @asm("call handle_divide_error")    // call the C handler
    @asm("pop rdi")
    @asm("pop rsi")
    @asm("pop rbp")
    @asm("pop rbx")
    @asm("pop rdx")
    @asm("pop rcx")
    @asm("pop rax")
    @asm("iretq")
```

#### 2. CPU Exception Handlers (with error code)

```snask
@naked fun isr_page_fault()
    // CPU pushed: error_code, RIP, CS, RFLAGS, RSP, SS
    @asm("push rax")
    @asm("mov rax, cr2")               // page fault address
    @asm("push rax")
    @asm("call handle_page_fault")
    @asm("pop rax")
    @asm("pop rax")
    @asm("iretq")
```

#### 3. System Call Entry

```snask
@naked fun syscall_entry()
    @asm("swapgs")
    @asm("mov gs:[0], rsp")            // save user RSP
    @asm("mov rsp, gs:[8]")            // load kernel RSP
    @asm("push rbx")
    @asm("push rcx")                    // saved RIP from syscall
    @asm("push rdx")
    @asm("push rsi")
    @asm("push rdi")
    @asm("push r8")
    @asm("push r9")
    @asm("push r10")
    @asm("push r11")                    // saved RFLAGS from syscall
    @asm("call syscall_dispatch")
    @asm("pop r11")
    @asm("pop r10")
    @asm("pop r9")
    @asm("pop r8")
    @asm("pop rdi")
    @asm("pop rsi")
    @asm("pop rdx")
    @asm("pop rcx")
    @asm("pop rbx")
    @asm("mov rsp, gs:[0]")            // restore user RSP
    @asm("swapgs")
    @asm("sysretq")
```

#### 4. Boot Entry Point (Multiboot)

```snask
@naked fun _start()
    @global_asm(".section .multiboot")
    @global_asm(".align 4")
    @global_asm(".long 0x1BADB002")     // magic
    @global_asm(".long 0x00000000")     // flags
    @global_asm(".long -(0x1BADB002)")  // checksum
    @global_asm(".text")
    @asm("cli")
    @asm("mov rsp, stack_top")
    @asm("call kmain")
    @asm("hlt")
```

### Declaration without Body

```snask
@naked fun isr_spurious()
```

This declares the symbol so it can be referenced in IDT setup without
generating any code. The handler can be defined externally or in assembly.

---

## 3. `@extern fun` — External Functions

Declare functions defined in C or assembly objects linked into the final binary.

### Syntax

```snask
@extern fun name(param: Type, ...) : ReturnType
```

No body — the function is resolved at link time.

### Rules

| Property | Behavior |
|----------|----------|
| Calling convention | C ABI (System V x86_64, AAPCS ARM, etc.) |
| Name mangling | None — symbol name matches exactly |
| Body | Not allowed |
| Returns | By value (no boxing) |

### Examples

#### Linking with C

```c
// cpart.c
void outb(uint16_t port, uint8_t val) {
    asm("outb %0, %1" : : "a"(val), "Nd"(port));
}
```

```snask
// kernel.snask
@extern fun outb(port: U16, val: U8)

@raw fun kmain()
    outb(0x21, 0xFC)   // mask PIC interrupts
    loop
        @asm("hlt")
```

Build:

```bash
gcc -c cpart.c -o cpart.o
snask build kernel.snask --profile baremetal cpart.o
```

#### Assembly Functions

```asm
// utils.asm
global load_gdt
load_gdt:
    lgdt [rdi]
    push 0x08
    lea rax, [rel .reload]
    push rax
    retfq
.reload:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    ret
```

```snask
@extern fun load_gdt(gdt_ptr: ptr)

@raw fun kmain()
    let gdt: GDT_ptr = prepare_gdt()
    load_gdt(&gdt)
```

## 4. Function Comparison

| Feature | `@raw` | `@naked` | `@extern` |
|---------|--------|----------|-----------|
| Body required | Yes | Optional | No |
| Prologue | Yes | No | (C ABI) |
| Epilogue | Yes | No | (C ABI) |
| Params in registers | Yes | Yes | Yes (C ABI) |
| Return in register | Yes | via @asm | Yes (C ABI) |
| Name mangling | Snask (`f_` prefix) | Snask (`f_` prefix) | None |
| Linkage | Internal | Internal | External |
| Can inline asm | Yes | Yes | No |
| Can call other fns | Yes | Yes (with push/pop) | N/A |
| Alloca for params | Yes (optimized away usually) | No | N/A |

## 5. Calling Convention Details

### `@raw fun` (x86_64)

- **Integer args:** RDI, RSI, RDX, RCX, R8, R9 (System V)
- **Return:** RAX
- **Stack cleanup:** Caller

### `@naked fun` (x86_64)

- **Interrupt:** CPU pushes RIP, CS, RFLAGS, RSP, SS (and error code for some)
- **Return:** `iretq` or `ret` depending on context
- **No automatic save/restore** — you must push/pop everything you touch

### `@extern fun`

- Follows the platform C ABI (System V x86_64, AAPCS32/64 on ARM, etc.)
- For x86_64: first 6 integer args in RDI, RSI, RDX, RCX, R8, R9
