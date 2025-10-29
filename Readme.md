# Stinkarm

> A stinky user-space arm emulator

## Usage

```text
$ nix develop
$ arm-none-eabi-gcc -nostdlib -Ttext=0x8000 target.c -o target.elf
$ ./stinkarm target.elf
[     0.012ms] opening binary `target.elf`
[     0.036ms] parsing elf...
[     0.040ms] \
  Magic:                                7f 45 4c 46
  Class:                                ELF32
  Data:                                 2's complement, little endian
  Version:                              1 (current)
  OS/ABI:                               0
  ABI Version:                          0
  Type:                                 Executable
  Machine:                              EM_ARM
  Version:                              0x1
  Entry point address:                  0x8000
  Start of program headers:             52 (bytes into file)
  Start of section headers:             4724 (bytes into file)
  Flags:                                0x5000200
  Size of this header:                  52 (bytes)
  Size of program headers:              32 (bytes)
  Number of program headers:            1
  Size of section headers:              40 (bytes)
  Number of section headers:            9
  Section header string table index:    8
[     0.082ms] booting...
[     0.083ms] shutting down
```

## Features

Emulate the ARM instruction set on a Linux kernel userspace level, forward
syscalls to the x86 host system and attempt to make it fast, current progress:

- [ ] `int main(void){ return 0; }`
- [ ] hello world with `puts`
- [ ] minimal libc based application
- [ ] "full" libc

### Detailed roadmap

- [x] parse ELF headers
- [ ] parse program headers and map PT_LOAD segments into guest memory
- [ ] compute initial brk and set up a stack region
- [ ] build the initial stack (argc/argv/envp/auxv), set SP
- [ ] initialize CPU state
- [ ] implement `svc` trapping and a minimal syscall passthrough

### Arm instructions

| Done | #   | Instruction | Operands / Notes          | Purpose / Use Case                     | Step |
| ---- | --- | ----------- | ------------------------- | -------------------------------------- | ---- |
| ❌   | 1   | MOV         | r0, #imm                  | Load immediate value (e.g., exit code) | 1    |
| ❌   | 2   | MOV         | r7, #imm                  | Load syscall number into r7            | 1    |
| ❌   | 3   | SVC         | #0                        | Trap into kernel (syscall)             | 1    |
| ❌   | 4   | ADR         | r1, label                 | Load address of string literal         | 2    |
| ❌   | 5   | MOV         | r2, #imm                  | Load string length                     | 2    |
| ❌   | 6   | MOV         | r0, #1                    | Load stdout fd                         | 2    |
| ❌   | 7   | MOV         | r7, #4                    | Syscall number for write               | 2    |
| ❌   | 8   | SVC         | #0                        | Write syscall                          | 2    |
| ❌   | 9   | LDR         | Rt, [Rn, #offset]         | Load word from memory (stack or heap)  | 3    |
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
| ❌   | 1   | exit            | 0x900001 | int error_code        | -                        | -                       | -                                  | -            | -            |
| ❌   | 3   | read            | 0x900003 | unsigned int fd       | char \*buf               | size_t count            | -                                  | -            | -            |
| ❌   | 4   | write           | 0x900004 | unsigned int fd       | const char \*buf         | size_t count            | -                                  | -            | -            |
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
