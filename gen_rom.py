import os

# Cria uma ROM minima do SNES (32KB)
rom = bytearray(32768)

# Endereço base de mapeamento = 0x8000 (offset 0 do arquivo)

# Instrução 1: LDA #$40
rom[0] = 0xA9
rom[1] = 0x40 # '@' em ASCII

# Instrução 2: STA $2000
rom[2] = 0x8D
rom[3] = 0x00
rom[4] = 0x20

# Instrução 3: JMP $8005 (Loop infinito no NOP/JMP)
rom[5] = 0x4C
rom[6] = 0x05
rom[7] = 0x80

with open("c:/Users/ranci/Desktop/TheSnask/smw.sfc", "wb") as f:
    f.write(rom)

print("ROM de teste gerada com payload 65c816!")
