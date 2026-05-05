# Fundacao Systems/NES em Snask

Este documento lista as primitivas de baixo nivel adicionadas para permitir um emulador NES real em Snask puro. Nenhuma sintaxe nova e exigida: tudo usa chamadas de funcao normais e o perfil `systems`.

## Memoria crua

Memoria crua e manual. Use dentro de `@unsafe`.

```text
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

Leituras multi-byte usam little-endian, igual ao 6502.

## Conversao inteira

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

## Words 6502

- `lo_u8(word) -> u8`
- `hi_u8(word) -> u8`
- `make_u16(lo, hi) -> u16`

Essas funcoes servem para reset vectors, NMI/IRQ vectors e enderecamento indireto.

## Bits e flags

- `bit_test(value, bit) -> bool`
- `bit_set(value, bit)`
- `bit_clear(value, bit)`
- `bit_toggle(value, bit)`
- `bit_write(value, bit, enabled)`
- `flag_has(flags, bit) -> bool`
- `flag_set(flags, bit)`
- `flag_clear(flags, bit)`
- `flag_write(flags, bit, enabled)`
- `is_zero_u8(value) -> bool`
- `is_negative_u8(value) -> bool`

Registro de status do 6502:

```text
N V - B D I Z C
7 6 5 4 3 2 1 0
```

## Overflow, carry e borrow

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

Essas primitivas nao implementam a CPU sozinhas. Elas dao ao codigo Snask os tijolos deterministas para implementar ADC, SBC, flags, wraps e ciclos sem depender de comportamento indefinido.

## Por que isso importa para NES

Um NES real exige:

- barramento de 64KB;
- registradores `u8` e `u16`;
- stack na pagina `0x0100`;
- flags exatas;
- leitura little-endian;
- overflow e carry fieis;
- PPU com VRAM, OAM, scroll e timing;
- input deterministico;
- execucao previsivel por frame.

O app `apps/nes_emulator` usa essas fundacoes para executar uma ROM real NROM em Snask puro.
