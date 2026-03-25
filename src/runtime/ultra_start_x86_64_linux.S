.global _start
.extern main

// Minimal entrypoint for ultra-tiny builds (Linux x86_64).
// Calls `main()` and exits via syscall(60).
_start:
    call main
    mov %eax, %edi
    mov $60, %eax
    syscall

