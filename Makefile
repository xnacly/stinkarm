all: examples/c.elf examples/asm.elf

examples/c.elf: examples/main.c
	arm-none-eabi-gcc -nostdlib -ffreestanding -Ttext=0x8000 $< -o $@

examples/asm.elf: examples/main.S
	arm-none-eabi-as -march=armv7-a $< -o main.o
	arm-none-eabi-ld -Ttext=0x8000 main.o -o $@
	rm main.o

clean:
	rm -f examples/c.elf examples/asm.elf
