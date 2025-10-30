all: examples/c.elf examples/asm.elf examples/rust.elf

examples/c.elf: examples/main.c
	arm-none-eabi-gcc -nostdlib -ffreestanding -Ttext=0x8000 $< -o $@

examples/asm.elf: examples/main.S
	arm-none-eabi-as -march=armv7-a $< -o main.o
	arm-none-eabi-ld -Ttext=0x8000 main.o -o $@
	rm main.o

examples/rust.elf: examples/main.rs
	cd examples/ & cargo build --release --target=armv7-unknown-linux-gnueabi

clean:
	rm -f examples/c.elf examples/asm.elf examples/rust.elf
