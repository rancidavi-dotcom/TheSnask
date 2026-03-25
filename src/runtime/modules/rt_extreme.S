.section .text
.global s_print
.global s_println
.global _start

s_print:
    # v em rdi (SnaskValue*)
    mov 0(%rdi), %rax   # tag em rax
    cvttsd2si %rax, %rcx
    cmp $3, %rcx        # STR tag is 3
    jne .Ldone
    
    mov 16(%rdi), %rsi  # ptr para a string em rsi
    # Calcular strlen (muito simples)
    mov %rsi, %rdx
.Llen:
    cmpb $0, (%rdx)
    je .Lwrite
    inc %rdx
    jmp .Llen
.Lwrite:
    sub %rsi, %rdx      # rdx = len
    mov $1, %rax        # write syscall
    mov $1, %rdi        # fd stdout
    syscall
    
    # Print space
    sub $16, %rsp
    movb $32, (%rsp)
    mov $1, %rax
    mov $1, %rdi
    mov %rsp, %rsi
    mov $1, %rdx
    syscall
    add $16, %rsp

.Ldone:
    ret

s_println:
    sub $16, %rsp
    movb $10, (%rsp)
    mov $1, %rax
    mov $1, %rdi
    mov %rsp, %rsi
    mov $1, %rdx
    syscall
    add $16, %rsp
    ret

_start:
    # Zerar rbp para debuggers
    xor %rbp, %rbp
    # Chamar start da classe main (nome gerado pelo compilador)
    call main_NS_start
    # Exit syscall
    mov $60, %rax
    xor %rdi, %rdi
    syscall
