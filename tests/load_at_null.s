@ Linked by the test runner with .text at guest address 0.
@ The ELF loader should reject a PT_LOAD segment mapped onto the null page.

    .global _start
_start:
    mov r0, #0
    mov r7, #1
    svc #0
