    .global _start
_start:
    bl main
    mov r7, #1
    svc #0          
    
    .global main
main:
    mov r0, #161
    bx lr          
