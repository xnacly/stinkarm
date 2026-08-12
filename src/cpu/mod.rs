use crate::{
    config::{self, Log, SyscallMode},
    cpu::{
        decoder::{Decoded, InstructionKind},
        translation::ArmSyscall,
    },
    err, mem, stinkln, sys,
};

/// decoding ARM instructions
mod decoder;
/// sandboxing the emulator
mod sandbox;
/// translating various things from arm to x86
mod translation;

type SyscallHandlerFn<'cpu, const PRINT_INSTR: bool> =
    fn(&mut Cpu<'cpu, PRINT_INSTR>, ArmSyscall) -> i32;

/// Usermode emulation
pub struct Cpu<'cpu, const PRINT_INSTR: bool> {
    /// r0-r15 (r13=SP, r14=LR, r15=PC)
    pub r: [u32; 16],
    pub cpsr: u32,
    pub mem: &'cpu mut mem::Mem,
    syscall_handler: SyscallHandlerFn<'cpu, PRINT_INSTR>,
    /// only set by ArmSyscall::Exit, necessary to propagate exit code to the host
    pub status: Option<i32>,
}

fn print_i32_or_errno(r: i32) -> i32 {
    if r < 0 {
        println!("={:?}", sys::Errno::from(r));
    } else {
        println!("={}", r);
    }

    r
}

impl<'cpu, const PRINT_INSTR: bool> Cpu<'cpu, PRINT_INSTR> {
    pub fn new(conf: &'cpu config::Config, mem: &'cpu mut mem::Mem, pc: u32) -> Self {
        let syscall_handler: SyscallHandlerFn<'cpu, PRINT_INSTR> = if conf
            .log
            .contains(&Log::Syscalls)
        {
            match conf.syscalls {
                SyscallMode::Forward => |cpu, syscall| {
                    println!("{}", syscall.print(cpu));
                    print_i32_or_errno(translation::syscall_forward(cpu, syscall))
                },
                SyscallMode::Sandbox => |cpu, syscall| {
                    println!("{} [sandbox]", syscall.print(cpu));
                    print_i32_or_errno(sandbox::syscall_sandbox(cpu, syscall))
                },
                SyscallMode::Deny => |cpu, syscall| {
                    println!("{} [deny]", syscall.print(cpu));
                    print_i32_or_errno(sandbox::syscall_deny(cpu, syscall))
                },
            }
        } else {
            match conf.syscalls {
                SyscallMode::Forward => |cpu, syscall| translation::syscall_forward(cpu, syscall),
                SyscallMode::Sandbox => |cpu, syscall| sandbox::syscall_sandbox(cpu, syscall),
                SyscallMode::Deny => |cpu, syscall| sandbox::syscall_deny(cpu, syscall),
            }
        };

        let mut s = Self {
            r: [0; 16],
            cpsr: 0x60000010,
            mem,
            syscall_handler,
            status: None,
        };
        s.r[15] = pc;
        s
    }

    pub fn reset(&mut self) {
        self.r = [0; 16];
        self.cpsr = 0x60000010;
    }

    #[inline(always)]
    pub fn instr_addr(&self) -> u32 {
        self.r[15] & !3
    }

    #[inline(always)]
    fn arm_pc(&self) -> u32 {
        self.instr_addr().wrapping_add(8)
    }

    /// moves pc forward a word
    #[inline(always)]
    fn advance(&mut self) {
        self.r[15] = self.r[15].wrapping_add(4);
    }

    #[inline(always)]
    fn cond_passes(&self, cond: u8) -> bool {
        match cond {
            0x0 => (self.cpsr >> 30) & 1 == 1, // EQ: Z == 1
            0x1 => (self.cpsr >> 30) & 1 == 0, // NE
            0xE => true,                       // AL (always)
            0xF => false,                      // NV (never)
            _ => false,                        // strict false
        }
    }

    /// fetch-decode-execute step, will only return false on exit svc
    pub fn step(&mut self) -> Result<bool, err::Err> {
        let Some(word) = self.mem.read_u32(self.instr_addr()) else {
            return Err(err::Err::MemoryAccessViolation {
                guest: self.instr_addr(),
                instr: 0xDEADAFFE,
            });
        };

        let Decoded { kind, cond, raw } = decoder::decode_word(word);

        if PRINT_INSTR {
            stinkln!("{:?} {:04b} {:X}", kind, cond, raw);
        }

        // we dont execute this instruction, moving along
        if !self.cond_passes(cond) {
            self.advance();
            return Ok(true);
        }

        // we keep track of PC changes, if an instruction writes to it, then we should not advance,
        // otherwise, we advance.
        let mut pc_changed = false;

        match kind {
            InstructionKind::MovImm => {
                let rd = decoder::bits(raw, 15, 12) as usize;
                let imm12 = decoder::bits(raw, 11, 0);
                self.r[rd] = decoder::decode_rotated_imm(imm12);
            }
            InstructionKind::Svc => {
                self.r[0] = match ArmSyscall::try_from(self.r[7]) {
                    Ok(kind) => (self.syscall_handler)(self, kind) as u32,
                    Err(_) => sys::Errno::ENOSYS.as_ret(),
                };
            }
            InstructionKind::LdrLiteral => {
                let rd = decoder::bits(raw, 15, 12) as usize;
                let imm12 = decoder::bits(raw, 11, 0);
                let addr = self.arm_pc().wrapping_add(imm12);
                self.r[rd] =
                    self.mem
                        .read_u32(addr)
                        .ok_or_else(|| err::Err::MemoryAccessViolation {
                            guest: addr,
                            instr: raw,
                        })?;
            }
            InstructionKind::Branch => {
                let l = decoder::bit(raw, 24);

                // BL
                if l {
                    // save return addr to LR (next addr though)
                    self.r[14] = self.instr_addr().wrapping_add(4);
                }

                let imm24 = decoder::bits(raw, 23, 0);
                let imm26 = imm24 << 2;
                let imm32 = decoder::sign_extend(imm26, 26);

                self.r[15] = self.arm_pc().wrapping_add(imm32 as u32);
                pc_changed = true;
            }
            InstructionKind::Unknown => {
                stinkln!("found unimplemented instruction, exiting: {:#x}", word);
                return Err(err::Err::UnknownOrUnsupportedInstruction(raw));
            }
        }

        if !pc_changed {
            self.advance();
        }

        Ok(true)
    }
}
