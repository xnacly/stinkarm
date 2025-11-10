@ write a 0 ptr, len 0 to stdout (fd1)
    .global _start
_start:
    mov r0, #1              
    mov r1, #0  @ null ptr
    mov r2, #0  @ length 0 
    mov r7, #4              
    svc #0                 
