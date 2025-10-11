run: build
	cargo run -- target.elf

qemu: build
	qemu-arm -d in_asm target.elf

build:
	arm-none-eabi-gcc -nostdlib -Ttext=0x8000 target.c -o target.elf
