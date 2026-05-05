# Emulador NES em Snask

Este app e o laboratorio mais agressivo do perfil `systems`: uma ROM real de NES rodando em Snask puro, sem escrever C no codigo do emulador.

## Objetivo

Criar um emulador NES fiel o bastante para jogos reais, mantendo:

- sintaxe Snask;
- memoria crua isolada por `@unsafe`/perfil `systems`;
- runtime e diagnosticos Snask;
- GUI via biblioteca nativa `snaskgui` ja integrada ao projeto;
- CPU, PPU, barramento, controle e loader em Snask.

## Estado atual

Ja existe:

- loader iNES em Snask;
- suporte ao mapper 0/NROM, usado por Super Mario Bros.;
- PRG carregado em `0x8000/0xC000`;
- CHR carregado em memoria dedicada;
- barramento de 64KB;
- CPU 6502 grande o bastante para executar a ROM real;
- NMI, reset vector, stack e flags;
- PPU em evolucao com VRAM, nametables, palettes, OAM, scroll e framebuffer;
- input de controle ligado ao registrador `0x4016`;
- janela 256x240 com framebuffer RGBA.

## Estado jogavel

O jogo ja entra em gameplay e responde ao controle. A renderizacao ainda esta em fase de precisao: camera/scroll, atributos, sprites e timing PPU estao sendo aproximados para chegar no comportamento real.

Isso significa: o projeto nao e mais apenas CHR viewer, mas ainda nao deve ser chamado de NES completo.

## ROM usada no ambiente local

```text
/home/davidev/Desktop/Emulator/roms/Super Mario Bros. (World).nes
```

## Controles atuais

- Setas: direcional.
- `X` ou espaco: A / pulo.
- `Z`: B.
- Enter: Start.
- Shift: Select.
- Esc: sair.

## Build e execucao

```bash
cargo build --bin snask
./target/debug/snask build apps/nes_emulator/nes_master.snask --profile systems --output /tmp/snask_nes
/tmp/snask_nes
```

## Proximos passos tecnicos

- PPU mais fiel por ciclo, nao apenas por frame.
- Scroll e fetch de nametable/attribute table alinhados ao hardware.
- Sprite evaluation mais correta.
- Sprite zero hit e flags no timing correto.
- APU.
- Mappers alem de NROM: MMC1, UNROM, CNROM, MMC3.

## Regra do projeto

O emulador deve continuar Snask puro. A existencia do runtime C nativo nao autoriza portar a CPU/PPU para C; C fica como runtime/base nativa, nao como codigo do usuario.
