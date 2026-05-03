#include <stdio.h>
typedef struct CPU
{
    unsigned char A;
    unsigned char PC;
}CPU;

int main(){
    int opcode;
    unsigned char memory[256];
    CPU cpu;
    cpu.A = 0;
    cpu.PC = 0;
    memory[0] = 1;
    memory[1] = 50;
    memory[2] = 2;
    memory[3] = 0;
    opcode = memory[cpu.PC];
    printf("Opcode: %d\n", opcode);
    switch (opcode)
    { case 1:
        printf("carregar valor");
        cpu.A = memory[cpu.PC + 1];
        cpu.PC += 2;
        break;
    case 2:
        printf("imprimir valor");
        printf("Valor: %d\n", cpu.A);
        break;
    case 0:
        printf("parar execução");
        break;
     default:
        printf("opcode desconhecido");
    }
    }
      
