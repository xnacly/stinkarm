__asm__("\
    .global _start\
_start:\
    bl main\n\
    mov r7, #1\n\
    svc #0\n"
        "
);
int main(void) { return 0; }
