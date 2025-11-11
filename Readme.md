# Stinkarm

> ARMv7 userspace binary emulator for x86 linux system supporting syscall sandboxing and other configurations

## Usage

```text
ARMv7 userspace binary emulator for x86 linux systems

Usage: stinkarm [OPTIONS] <TARGET>

Arguments:
  <TARGET>
          Path to the ARM ELF binary to execute

Options:
  -C, --syscalls <SYSCALLS>
          Syscall handling mode

          Possible values:
          - forward: Forward syscalls to the host system (via ARMv7->x86 translation layer)
          - deny:    Deny syscalls: return -ENOSYS on all invocations
          - sandbox: Sandbox: only allow a safe subset: no file IO (except fd 0,1,2), no network, no process spawns

          [default: sandbox]

  -s, --stack-size <STACK_SIZE>
          Stack size for the emulated process (in bytes)

          [default: 1048576]

  -n, --no-env
          Don't pass host env to emulated process

  -l, --log <LOG>
          Configure what data to log

          [possible values: none, elf, syscalls, memory]

  -v, --verbose
          Log everything and anything

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```


### Example

```text
# enter build env with nix or have 'arm-none-eabi-as', 'arm-none-eabi-ld' and 'arm-none-eabi-gcc'
$ nix develop
$ cargo run --bin bld_exmpl # builds examples to elf binaries in examples/
$ cargo run -- -lsyscalls --syscalls=deny examples/helloWorld.elf
46691 write(fd=1, buf=0x8024, len=14) [deny]
=ENOSYS
46691 exit(code=0) [deny]
=ENOSYS
$ cargo run -- -v --syscalls=sandbox examples/helloWorld.elf
[     0.438ms] opening binary "examples/helloWorld.elf"
[     0.496ms] parsing ELF...
[     0.511ms] \
ELF Header:
  Magic:              [7f, 45, 4c, 46]
  Class:              ELF32
  Data:               Little endian
  Type:               Executable
  Machine:            EM_ARM
  Version:            1
  Entry point:        0x8000
  Program hdr offset: 52 (32 bytes each)
  Section hdr offset: 4696
  Flags:              0x05000200
  EH size:            52
  # Program headers:  1
  # Section headers:  9
  Str tbl index:      8

Program Headers:
  Type       Offset   VirtAddr   PhysAddr   FileSz    MemSz  Flags  Align
  LOAD     0x001000 0x00008000 0x00008000 0x000033 0x000033    R|X 0x1000

[     0.571ms] mapped program header `LOAD` of 51B (G=0x8000 -> H=0x7ffff7f87000)
[     0.591ms] jumping to entry G=0x8000 at H=0x7ffff7f87000
[     0.600ms] starting the emulator
47164 write(fd=1, buf=0x8024, len=14) [sandbox]
Hello, world!
=14
47164 exit(code=0) [sandbox]
=0
[     0.658ms] exiting with `0`
```

## Features

Emulate the ARM instruction set on a Linux kernel userspace level, forward
syscalls to the x86 host system and attempt to make it fast, current progress:

- [x] minimal asm entry

```asm
    .global _start
_start:
    mov r0, #161
    mov r7, #1
    svc #0
```

- [x] asm hello world 

```asm
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
```

- [ ] `int main(void){ return 0; }`
- [ ] hello world with `puts`
- [ ] minimal libc based application
- [ ] "full" libc

### Detailed roadmap

- [x] parse ELF headers
- [x] parse program headers and map PT_LOAD segments into guest memory
- [ ] compute initial brk and set up a stack region
- [ ] build the initial stack (argc/argv/envp/auxv), set SP
- [x] initialize CPU state
- [x] start decoding instruction words and cpu steps
- [x] implement `svc` trapping and a minimal syscall passthrough

### Arm instructions

| Done | #   | Instruction | Operands / Notes          | Purpose / Use Case                     | Step |
| ---- | --- | ----------- | ------------------------- | -------------------------------------- | ---- |
| ✅   | 1   | MOV         | r0, #imm                  | Load immediate value (e.g., exit code) | 1    |
| ✅   | 3   | SVC         | #0                        | Trap into kernel (syscall)             | 1    |
| ✅   | 9   | LDR         | Rt, [Rn, #offset]         | Load word from memory (stack or heap)  | 3    |
| ❌   | 4   | ADR         | r1, label                 | Load address of string literal         | 2    |
| ❌   | 10  | STR         | Rt, [Rn, #offset]         | Store word to memory (stack or heap)   | 3    |
| ❌   | 11  | ADD         | Rd, Rn, Rm / Rd, Rn, #imm | Arithmetic / address calculation       | 3    |
| ❌   | 12  | SUB         | Rd, Rn, Rm / Rd, Rn, #imm | Arithmetic / address calculation       | 3    |
| ❌   | 13  | CMP         | Rn, Rm / Rn, #imm         | Compare registers / conditional logic  | 3    |
| ❌   | 14  | B / BL      | label                     | Branch / call subroutine               | 3    |
| ❌   | 15  | BX          | Rm                        | Return from subroutine (switch to LR)  | 3    |
| ❌   | 16  | BNE / BEQ   | label                     | Conditional branch                     | 3    |
| ❌   | 17  | NOP         | -                         | Optional padding / alignment           | 3    |

### Syscalls

| Done | #   | Name            | r7       | r0                    | r1                       | r2                      | r3                                 | r4           | r5           |
| ---- | --- | --------------- | -------- | --------------------- | ------------------------ | ----------------------- | ---------------------------------- | ------------ | ------------ |
| ✅   | 1   | exit            | 0x900001 | int error_code        | -                        | -                       | -                                  | -            | -            |
| ❌   | 3   | read            | 0x900003 | unsigned int fd       | char \*buf               | size_t count            | -                                  | -            | -            |
| ✅   | 4   | write           | 0x900004 | unsigned int fd       | const char \*buf         | size_t count            | -                                  | -            | -            |
| ❌   | 5   | open            | 0x900005 | const char \*filename | int flags                | umode_t mode            | -                                  | -            | -            |
| ❌   | 6   | close           | 0x900006 | unsigned int fd       | -                        | -                       | -                                  | -            | -            |
| ❌   | 10  | execve          | 0x90000b | const char \*filename | const char *const *argv  | const char *const *envp | -                                  | -            | -            |
| ❌   | 29  | mmap            | 0x90001d | void \*addr           | size_t length            | int prot                | int flags                          | int fd       | off_t offset |
| ❌   | 30  | munmap          | 0x90001e | void \*addr           | size_t length            | -                       | -                                  | -            | -            |
| ❌   | 39  | mprotect        | 0x900027 | void \*addr           | size_t len               | int prot                | -                                  | -            | -            |
| ❌   | 45  | brk             | 0x90002d | void \*end_data       | -                        | -                       | -                                  | -            | -            |
| ❌   | 59  | wait4           | 0x90003b | pid_t pid             | int \*stat_loc           | int options             | struct rusage \*ru                 | -            | -            |
| ❌   | 63  | set_tid_address | 0x90003f | int \*tidptr          | -                        | -                       | -                                  | -            | -            |
| ❌   | 64  | futex           | 0x900040 | u32 \*uaddr           | int op                   | u32 val                 | struct \_\_kernel_timespec \*utime | u32 \*uaddr2 | u32 val3     |
| ❌   | 87  | set_robust_list | 0x900057 | void \*head           | size_t len               | -                       | -                                  | -            | -            |
| ❌   | 93  | exit_group      | 0x90005d | int error_code        | -                        | -                       | -                                  | -            | -            |
| ❌   | 165 | fcntl           | 0x9000a5 | unsigned int fd       | unsigned int cmd         | unsigned long arg       | -                                  | -            | -            |
| ❌   | 174 | ioctl           | 0x9000ae | unsigned int fd       | unsigned int cmd         | unsigned long arg       | -                                  | -            | -            |
| ❌   | 19  | readv           | 0x900013 | unsigned long fd      | const struct iovec \*vec | unsigned long vlen      | -                                  | -            | -            |
| ❌   | 20  | writev          | 0x900014 | unsigned long fd      | const struct iovec \*vec | unsigned long vlen      | -                                  | -            | -            |
| ❌   | 21  | access          | 0x900015 | const char \*filename | int mode                 | -                       | -                                  | -            | -            |
| ❌   | 16  | lseek           | 0x900011 | unsigned int fd       | off_t offset             | unsigned int origin     | -                                  | -            | -            |
