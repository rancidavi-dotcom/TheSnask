# NES Emulator em Snask

Este app e o inicio do emulador de NES escrito em Snask puro.

O objetivo e manter a sintaxe Snask, sem C no codigo do usuario, usando `@unsafe` apenas nas zonas onde o core precisa tocar memoria crua. O resto continua passando pelo modelo da linguagem: tipos fixos, funcoes, diagnosticos, profiles e OM.

## Estado atual

- Barramento de 64KB alocado com `mem_alloc_zero`.
- Estado da CPU 6502 armazenado em uma area crua separada.
- Reset vector em `0xFFFC/0xFFFD`.
- Stack 6502 na pagina `0x0100`.
- Flags oficiais: C, Z, I, D, B, U, V, N.
- Execucao deterministica por steps.
- Programa de validacao carregado em `0x8000`.
- Subconjunto real de opcodes 6502 implementado como base.
- `snaskgui` nativo montado para janela 256x240, framebuffer RGBA e input.
- Loader binario `binfile_read_into` para ler ROM `.nes` sem truncar em bytes zero.
- Loader iNES em Snask para NROM inicial, copiando PRG/CHR para memoria crua.

## Proximos blocos para NES completo

- Completar a matriz oficial de 151 opcodes do 6502.
- Adicionar ciclos exatos, incluindo page-cross penalties.
- Expandir mappers alem de NROM: MMC1, UNROM, CNROM, MMC3.
- Implementar PPU com OAM, nametables, palettes e timing.
- Implementar APU.
- Integrar input e janela depois via OM-Snask-System.

## ROM real

O app esta apontado para:

```text
/home/davidev/Desktop/Emulator/roms/Super Mario Bros. (World).nes
```

Hoje o loader aceita o formato iNES e carrega NROM como primeiro alvo real.
O Super Mario Bros. e mapper 0, entao ele ja entra pela rota correta:
PRG em `0x8000/0xC000`, CHR em memoria dedicada, reset vector real.

## Estado jogavel

Ainda nao e jogavel completo. O que ja acontece:

- a ROM real do Mario e carregada;
- o CPU 6502 executa milhares de instrucoes da ROM sem cair no primeiro reset;
- a janela `snaskgui` fica viva com poll/delay/input;
- o framebuffer exibe os tiles CHR reais da ROM.

Falta para jogar de verdade:

- PPU com registradores reais `0x2000..0x2007`, VRAM, scroll e NMI;
- renderizacao por nametable/attribute table/sprites, nao so CHR viewer;
- APU audivel;
- input ligado ao registrador `0x4016`;
- mais opcodes/modos caso aparecam durante gameplay.

## Rodar

```bash
cargo build --bin snask
./target/debug/snask build apps/nes_emulator/nes_master.snask --profile systems --output /tmp/snask_nes
```
