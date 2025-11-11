    .section .rodata
msg:
    .asciz "Hello, world!\n"

    .section .text
    .global _start
_start:
    ldr r0, =1
    ldr r1, =msg                
    mov r2, #14                 
    mov r7, #4                  
    svc #0                      

    mov r0, #0                  
    mov r7, #1                  
    svc #0                      
