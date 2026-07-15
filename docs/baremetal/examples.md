# Snask Baremetal — Examples

> Complete, working examples of kernels, drivers, and emulators.

---

## 1. Minimal x86_64 Kernel

Prints 'O' to VGA text mode and halts.

**kernel.snask:**
```snask
const VGA_ADDR: U64 = 0xB8000
const WHITE_ON_BLACK: U16 = 0x0F00
const CHAR_O: U8 = 0x4F

@naked fun _start()
    @global_asm(".section .multiboot")
    @global_asm(".align 4")
    @global_asm(".long 0x1BADB002")
    @global_asm(".long 0x00000000")
    @global_asm(".long -(0x1BADB002)")
    @global_asm(".text")
    @asm("mov rsp, stack_top")
    @asm("call kmain")
    @asm("cli")
    @asm("hlt")

@raw fun kmain() : I32
    @store(VGA_ADDR, WHITE_ON_BLACK or CHAR_O)
    loop
        @asm("hlt")
    return 0

@global_asm(".section .bss")
@global_asm(".align 16")
@global_asm("stack_bottom:")
@global_asm(".skip 16384")
@global_asm("stack_top:")
```

**linker.ld:**
```ld
ENTRY(_start)

SECTIONS
{
    . = 0x100000;

    .multiboot : { *(.multiboot) }
    .text      : { *(.text*) }
    .rodata    : { *(.rodata*) }
    .data      : { *(.data*) }
    .bss       : { *(.bss*) }
}
```

**Build & run:**
```bash
snask build kernel.snask --profile baremetal
qemu-system-x86_64 -kernel kernel
```

You should see a white `O` on a black background in the top-left corner
of the QEMU display.

---

## 2. VGA Text Mode Driver

```snask
const VGA_ADDR: U64     = 0xB8000
const VGA_COLS: U64     = 80
const VGA_ROWS: U64     = 25
const VGA_CELL_SIZE: U64 = 2

mut vga_row: U8  = 0
mut vga_col: U8  = 0
mut vga_color: U8 = 0x0F   // white on black

@raw fun vga_set_color(fg: U8, bg: U8)
    vga_color = bg << 4 or fg

@raw fun vga_entry(c: U8, color: U8) : U16
    return as_u16(c) or (as_u16(color) << 8)

@raw fun vga_putc(c: U8)
    let row: U64 = as_u64(vga_row)
    let col: U64 = as_u64(vga_col)
    let offset: U64 = (row * VGA_COLS + col) * VGA_CELL_SIZE
    let entry: U16 = vga_entry(c, vga_color)
    @store(VGA_ADDR + offset, entry)

    vga_col = vga_col + 1
    if vga_col >= VGA_COLS
        vga_col = 0
        vga_row = vga_row + 1
        if vga_row >= VGA_ROWS
            vga_row = VGA_ROWS - 1
            vga_scroll()

@raw fun vga_scroll()
    let src: U64 = VGA_ADDR + VGA_COLS * VGA_CELL_SIZE
    let dst: U64 = VGA_ADDR
    let bytes: U64 = (VGA_ROWS - 1) * VGA_COLS * VGA_CELL_SIZE
    mem_copy(@inttoptr(dst, ptr), @inttoptr(src, ptr), bytes)

    // Clear last line
    let last_line: U64 = VGA_ADDR + (VGA_ROWS - 1) * VGA_COLS * VGA_CELL_SIZE
    mem_fill_u8(@inttoptr(last_line, ptr), 0, VGA_COLS * VGA_CELL_SIZE)

@raw fun vga_puts(s: ptr, len: U64)
    for i in 0..len
        vga_putc(mem_read_u8(s, i))

@raw fun vga_clear()
    mem_fill_u8(@inttoptr(VGA_ADDR, ptr), 0, VGA_ROWS * VGA_COLS * VGA_CELL_SIZE)
    vga_row = 0
    vga_col = 0

@raw fun kmain() : I32
    vga_clear()
    vga_set_color(0x0F, 0x00)  // white on black
    let msg = "Hello from Snask!"
    vga_puts(&msg, 17)          // note: string literal in baremetal is tricky
    loop
        @asm("hlt")
    return 0
```

---

## 3. Serial (UART 16550) Driver

```snask
const COM1: U64 = 0x3F8

// Register offsets
const DATA: U64    = 0
const IER: U64     = 1
const IIR: U64     = 2
const LCR: U64     = 3
const MCR: U64     = 4
const LSR: U64     = 5
const MSR: U64     = 6

const DLAB: U8 = 0x80

@raw fun uart_init()
    @write(COM1 + IER, 0x00, U8)       // disable interrupts
    @write(COM1 + LCR, DLAB, U8)        // enable DLAB
    @write(COM1 + DATA, 0x01, U8)       // baud = 115200 (divisor 1)
    @write(COM1 + IER, 0x00, U8)        // high byte of divisor
    @write(COM1 + LCR, 0x03, U8)        // 8N1
    @write(COM1 + MCR, 0x0B, U8)        // DTR + RTS + aux2
    @write(COM1 + IER, 0x01, U8)        // enable RX interrupt

@raw fun uart_tx_ready() : Bool
    return (@volatile(COM1 + LSR, U8) & 0x20) != 0

@raw fun uart_rx_ready() : Bool
    return (@volatile(COM1 + LSR, U8) & 0x01) != 0

@raw fun uart_putc(c: U8)
    while not uart_tx_ready()
        @asm("pause")
    @write(COM1 + DATA, c, U8)

@raw fun uart_getc() : U8
    while not uart_rx_ready()
        @asm("pause")
    return @volatile(COM1 + DATA, U8)

@raw fun uart_puts(s: ptr, len: U64)
    for i in 0..len
        uart_putc(mem_read_u8(s, i))
    uart_putc('\r')
    uart_putc('\n')

@raw fun uart_put_hex(n: U64)
    let hex: ptr = @inttoptr(0x7000, ptr)  // scratch buffer
    let digits: U64 = 16
    for i in 0..16
        let nibble: U64 = (n >> ((15 - i) * 4)) and 0xF
        let c: U8 = as_u8(if nibble < 10 nibble + 0x30 else nibble + 0x57)
        mem_write_u8(hex, i, c)
    uart_puts(hex, 16)
```

---

## 4. Interrupt Descriptor Table (IDT)

```snask
const IDT_ENTRIES: U64 = 256

struct IDT_Entry
    volatile offset_low: U16
    volatile selector: U16
    volatile ist: U8
    volatile flags: U8
    volatile offset_mid: U16
    volatile offset_high: U32
    volatile zero: U32

struct IDT_Ptr
    volatile limit: U16
    volatile base: U64

mut idt: IDT_Entry[256]   // array not available — use manual layout

@naked fun isr_divide_error()
    @asm("push rax")
    @asm("push rcx")
    @asm("push rdx")
    @asm("push rbx")
    @asm("push rbp")
    @asm("push rsi")
    @asm("push rdi")
    @asm("call handle_divide_error")
    @asm("pop rdi")
    @asm("pop rsi")
    @asm("pop rbp")
    @asm("pop rbx")
    @asm("pop rdx")
    @asm("pop rcx")
    @asm("pop rax")
    @asm("iretq")

@naked fun isr_page_fault()
    @asm("push rax")
    @asm("mov rax, cr2")
    @asm("push rax")
    @asm("push rcx")
    @asm("push rdx")
    @asm("push rbx")
    @asm("push rbp")
    @asm("push rsi")
    @asm("push rdi")
    @asm("call handle_page_fault")
    @asm("pop rdi")
    @asm("pop rsi")
    @asm("pop rbp")
    @asm("pop rbx")
    @asm("pop rdx")
    @asm("pop rcx")
    @asm("pop rax")
    @asm("pop rax")   // clean up cr2
    @asm("iretq")

@raw fun set_idt_entry(index: U64, handler: U64, selector: U16, flags: U8)
    let base: U64 = @ptrtoint(&idt, U64)
    let off: U64 = index * 16   // sizeof(IDT_Entry) = 16
    let entry_addr: U64 = base + off

    @store(entry_addr + 0, as_u16(handler and 0xFFFF))
    @store(entry_addr + 2, selector)
    @store(entry_addr + 4, as_u8(0))
    @store(entry_addr + 5, flags)
    @store(entry_addr + 6, as_u16((handler >> 16) and 0xFFFF))
    @store(entry_addr + 8, as_u32((handler >> 32) and 0xFFFFFFFF))
    @store(entry_addr + 12, as_u32(0))

@raw fun idt_init()
    let code_sel: U16 = 0x08
    let flags: U8 = 0x8E   // present, ring 0, 32-bit interrupt gate

    set_idt_entry(0,  @ptrtoint(isr_divide_error, U64), code_sel, flags)
    set_idt_entry(14, @ptrtoint(isr_page_fault, U64),  code_sel, flags)

    let ptr: IDT_Ptr
    ptr.limit = as_u16(IDT_ENTRIES * 16 - 1)
    ptr.base  = @ptrtoint(&idt, U64)

    @asm("lidt [rdi]")
```

---

## 5. PS/2 Keyboard Driver

```snask
const KEYBOARD_DATA: U64 = 0x60
const KEYBOARD_STATUS: U64 = 0x64

const KEY_ENTER: U8 = 0x1C
const KEY_BACKSPACE: U8 = 0x0E
const KEY_LSHIFT_DOWN: U8 = 0x2A
const KEY_LSHIFT_UP: U8 = 0xAA
const KEY_RSHIFT_DOWN: U8 = 0x36
const KEY_RSHIFT_UP: U8 = 0xB6

mut shift_pressed: Bool = false
mut key_buffer: U8[256]   // manual array
mut key_buffer_head: U64 = 0
mut key_buffer_tail: U64 = 0

const SCANCODE_TO_ASCII: U8[] = [
    0x00, 0x1B, '1', '2', '3', '4', '5', '6', '7', '8', '9', '0',
    '-', '=', '\b', '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i',
    'o', 'p', '[', ']', '\n', 0x00, 'a', 's', 'd', 'f', 'g', 'h',
    'j', 'k', 'l', ';', '\'', '`', 0x00, '\\', 'z', 'x', 'c', 'v',
    'b', 'n', 'm', ',', '.', '/', 0x00, '*', 0x00, ' '
]

@raw fun keyboard_handler()
    let status: U8 = @volatile(KEYBOARD_STATUS, U8)
    if (status and 0x01) == 0
        return

    let scancode: U8 = @volatile(KEYBOARD_DATA, U8)

    if scancode == KEY_LSHIFT_DOWN or scancode == KEY_RSHIFT_DOWN
        shift_pressed = true
        return
    if scancode == KEY_LSHIFT_UP or scancode == KEY_RSHIFT_UP
        shift_pressed = false
        return

    // Only handle make codes (bit 7 = 0)
    if (scancode and 0x80) == 0
        let ascii: U8 = SCANCODE_TO_ASCII[scancode]
        if ascii != 0
            // Store in ring buffer
            mem_write_u8(key_buffer, key_buffer_head, ascii)
            key_buffer_head = (key_buffer_head + 1) % 256

@raw fun keyboard_read() : U8
    while key_buffer_head == key_buffer_tail
        @asm("pause")
    let c: U8 = mem_read_u8(key_buffer, key_buffer_tail)
    key_buffer_tail = (key_buffer_tail + 1) % 256
    return c
```

---

## 6. Simple Paging (x86_64)

```snask
const PAGE_PRESENT: U64   = 1 << 0
const PAGE_WRITABLE: U64  = 1 << 1
const PAGE_HUGE: U64      = 1 << 7

// Page table level indices for address 0x0
// PML4[0] → PDP[0] → PD[0] → PT[0] → 4K page

mut pml4: U64[512]   // at some known physical address

@raw fun pml4_init()
    // Identity map first 2MB with huge pages
    let base: U64 = @ptrtoint(&pml4, U64)
    mem_fill_u8(@inttoptr(base, ptr), 0, 4096)    // zero PML4

    // Set up recursive mapping (PML4 last entry points to itself)
    mem_write_u64(@inttoptr(base + 511*8), base or 0x03)

    // PML4[0] → PDPT
    let pdpt_addr: U64 = base + 4096
    mem_write_u64(@inttoptr(base, ptr), pdpt_addr or (PAGE_PRESENT or PAGE_WRITABLE))

    // Zero PDPT
    mem_fill_u8(@inttoptr(pdpt_addr, ptr), 0, 4096)

    // PDPT[0] → 1GB page (if supported)
    mem_write_u64(@inttoptr(pdpt_addr, ptr), 0x0 or (PAGE_PRESENT or PAGE_WRITABLE or PAGE_HUGE))

@raw fun enable_paging()
    let pml4_phys: U64 = @ptrtoint(&pml4, U64)
    @asm("mov cr3, rax")     // RAX = pml4_phys
    @asm("mov rcx, cr0")
    @asm("or rcx, 0x80000001")
    @asm("mov cr0, rcx")     // enable paging + protection
```

---

## 7. Chip-8 Emulator (Core Loop)

A minimal Chip-8 CPU interpreter core showing how emulator inner loops
map to Snask baremetal.

```snask
const MEM_SIZE: U64  = 4096
const STACK_SIZE: U64 = 16

struct Chip8
    memory: U8[4096]
    V: U8[16]
    I: U16
    pc: U16
    stack: U16[16]
    sp: U8
    delay_timer: U8
    sound_timer: U8
    keypad: U8[16]
    display: U8[64 * 32]

@raw fun chip8_init(c8: ptr)
    mem_fill_u8(c8, 0, sizeof(Chip8))

@raw fun chip8_load_rom(c8: ptr, rom: ptr, size: U64)
    let mem_base: U64 = @ptrtoint(c8, U64)
    mem_copy(@inttoptr(mem_base, ptr), rom, size)

@raw fun chip8_emulate_cycle(c8: ptr)
    let pc: U64 = as_u64(@volatile(@ptrtoint(c8, U64) + offsetof(Chip8, pc), U16))
    let mem: U64 = @ptrtoint(c8, U64) + offsetof(Chip8, memory)

    let opcode: U16 = mem_read_u16(@inttoptr(mem, ptr), pc)
    let next_pc: U16 = as_u16(pc) + 2

    let nibble1: U8 = as_u8((opcode >> 12) and 0x000F)
    let nibble2: U8 = as_u8((opcode >> 8)  and 0x000F)
    let nibble3: U8 = as_u8((opcode >> 4)  and 0x000F)
    let nibble4: U8 = as_u8(opcode and 0x000F)

    let x: U8 = nibble2
    let y: U8 = nibble3
    let nnn: U16 = opcode and 0x0FFF

    if opcode == 0x00E0
        // CLS — clear display
        let disp: U64 = @ptrtoint(c8, U64) + offsetof(Chip8, display)
        mem_fill_u8(@inttoptr(disp, ptr), 0, 64 * 32)
    elif nibble1 == 0x01
        // JP nnn
        next_pc = nnn
    elif nibble1 == 0x06
        // LD Vx, byte
        let v_addr: U64 = @ptrtoint(c8, U64) + offsetof(Chip8, V) + as_u64(x)
        @store(v_addr, as_u8(nibble4))
    elif nibble1 == 0x07
        // ADD Vx, byte
        let v_addr: U64 = @ptrtoint(c8, U64) + offsetof(Chip8, V) + as_u64(x)
        let val: U8 = @volatile(v_addr, U8)
        @store(v_addr, val + as_u8(nibble4))
    elif nibble1 == 0x0A
        // LD I, nnn
        let i_addr: U64 = @ptrtoint(c8, U64) + offsetof(Chip8, I)
        @store(i_addr, nnn)
    elif nibble1 == 0x0D
        // DRW Vx, Vy, nibble — draw sprite
        let vx: U64 = @ptrtoint(c8, U64) + offsetof(Chip8, V) + as_u64(x)
        let vy: U64 = @ptrtoint(c8, U64) + offsetof(Chip8, V) + as_u64(y)
        let x_pos: U8  = @volatile(vx, U8)
        let y_pos: U8  = @volatile(vy, U8)
        let height: U8 = nibble4
        chip8_draw_sprite(c8, x_pos, y_pos, height)

    // Update PC
    let pc_addr: U64 = @ptrtoint(c8, U64) + offsetof(Chip8, pc)
    @store(pc_addr, next_pc)

@raw fun chip8_draw_sprite(c8: ptr, x: U8, y: U8, height: U8)
    let I: U64 = as_u64(@volatile(@ptrtoint(c8, U64) + offsetof(Chip8, I), U16))
    let mem: U64 = @ptrtoint(c8, U64) + offsetof(Chip8, memory)
    let disp: U64 = @ptrtoint(c8, U64) + offsetof(Chip8, display)

    // Set VF = 0 (collision flag)
    let vf_addr: U64 = @ptrtoint(c8, U64) + offsetof(Chip8, V) + 0xF
    @store(vf_addr, 0)

    for row in 0..as_u64(height)
        let sprite_byte: U8 = mem_read_u8(@inttoptr(mem, ptr), I + row)
        for col in 0..8
            if (sprite_byte and (0x80 >> col)) != 0
                let px: U64 = (as_u64(y) + row) * 64 + (as_u64(x) + col)
                let pixel: U8 = @volatile(disp + px, U8)
                if pixel != 0
                    @store(vf_addr, 1)   // collision
                @store(disp + px, pixel xor 1)
```

---

## 8. Build System Integration

### Build Script (build.sh)

```bash
//!/bin/bash
set -euo pipefail

PROFILE="baremetal"
KERNEL="kernel.snask"
OUTPUT="kernel"
LINKER="linker.ld"

// Build
snask build $KERNEL --profile $PROFILE

// Optionally strip debug symbols
// x86_64-elf-strip $OUTPUT

// Run in QEMU
qemu-system-x86_64 \
    -kernel $OUTPUT \
    -serial stdio \
    -no-reboot \
    -d cpu_reset \
    -m 256M
```

### With GRUB Multiboot

```bash
// Create ISO
cp kernel isofiles/boot/kernel
cp grub.cfg isofiles/boot/grub/
grub-mkrescue -o kernel.iso isofiles

// Run
qemu-system-x86_64 -cdrom kernel.iso
```

grub.cfg:
```cfg
set timeout=0
set default=0

menuentry "Snask OS" {
    multiboot /boot/kernel
    boot
}
```

### Debugging with QEMU + GDB

```bash
// Terminal 1:
qemu-system-x86_64 -kernel kernel -s -S

// Terminal 2:
gdb -ex "target remote :1234" \
    -ex "symbol-file kernel" \
    -ex "break kmain" \
    -ex "continue"
```

---

## 9. Complete File Structure for a Real OS

```
myos/
├── kernel.snask          // Entry point (_start), kmain
├── vga.snask             // VGA text mode driver
├── serial.snask          // UART 16550 driver
├── idt.snask             // IDT setup & handlers
├── gdt.snask             // GDT setup
├── paging.snask          // Page table management
├── keyboard.snask        // PS/2 keyboard driver
├── timer.snask           // PIT/HPET driver
├── heap.snask            // Simple heap allocator
├── linker.ld             // Linker script
├── build.sh              // Build & run script
└── README.md             // Project documentation
```

Each file compiles independently and can be linked together:

```bash
snask build kernel.snask --profile baremetal vga.snask serial.snask idt.snask ...
```
