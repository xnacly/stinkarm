.text
    .global _start

_start:
    mov r0, #42
loop:
    b loop
