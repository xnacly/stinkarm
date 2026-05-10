@ Attempts to write seven bytes from guest address 0.
@ The emulator should reject the null guest pointer with EFAULT.

    .section .rodata
msg:
    .ascii "ignored"

    .section .text
    .global _start
_start:
    mov r0, #1
    mov r1, #0
    mov r2, #7
    mov r7, #4
    svc #0

    mov r0, #0
    mov r7, #1
    svc #0
