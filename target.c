int main(void);

// prelude so we have something the cpu can start at
__asm__(".global _start        \n"
        "_start:               \n"
        "    bl main           \n"
        "    b .               \n");

int main(void) { return 161; }
