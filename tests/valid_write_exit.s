@ Writes a short in-bounds message to stdout, then exits with code 7.
@ This is the happy-path e2e case for write and exit syscall handling.

    .section .rodata
msg:
    .ascii "ok\n"

    .section .text
    .global _start
_start:
    mov r0, #1
    ldr r1, =msg
    mov r2, #3
    mov r7, #4
    svc #0

    mov r0, #7
    mov r7, #1
    svc #0
