@ Attempts to write from 0x08000000, exactly one byte past the default guest arena.
@ The emulator should bounds-check the full write buffer and return EFAULT.

    .section .rodata
msg:
    .ascii "ignored"

    .section .text
    .global _start
_start:
    mov r0, #1
    ldr r1, =0x08000000
    mov r2, #7
    mov r7, #4
    svc #0

    mov r0, #0
    mov r7, #1
    svc #0
