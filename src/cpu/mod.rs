use crate::{
    config::{self, Log, SyscallMode},
    cpu::{decoder::InstructionContainer, translation::ArmSyscall},
    err, mem, stinkln,
};

/// decoding ARM instructions
mod decoder;
/// sandboxing the emulator
mod sandbox;
/// translating various things from arm to x86
mod translation;

type SyscallHandlerFn = fn(&mut Cpu) -> i32;

/// Usermode emulation
pub struct Cpu<'cpu> {
    /// r0-r15 (r13=SP, r14=LR, r15=PC)
    pub r: [u32; 16],
    pub cpsr: u32,
    pub mem: &'cpu mut mem::Mem,
    syscall_handler: SyscallHandlerFn,
    /// only set by ArmSyscall::Exit, necessary to propagate exit code to the host
    pub status: Option<i32>,
}

impl<'cpu> Cpu<'cpu> {
    pub fn new(conf: &'cpu config::Config, mem: &'cpu mut mem::Mem, pc: u32) -> Self {
        let syscall_handler: SyscallHandlerFn = if conf.log == Log::Syscalls {
            match conf.syscalls {
                SyscallMode::Forward => |cpu| {
                    let r = translation::syscall_forward(cpu);
                    stinkln!(
                        "[syscall] {:?} ret={}",
                        ArmSyscall::try_from(cpu.r[7]).unwrap(),
                        r
                    );
                    r
                },
                SyscallMode::Sandbox => |cpu| {
                    let r = sandbox::syscall_sandbox(cpu);
                    stinkln!(
                        "[syscall] sandbox {:?} ret={}",
                        ArmSyscall::try_from(cpu.r[7]).unwrap(),
                        r
                    );
                    r
                },
                SyscallMode::Stub => |cpu| {
                    stinkln!(
                        "[syscall] stub {:?}, returning -EACCES",
                        ArmSyscall::try_from(cpu.r[7]).unwrap()
                    );
                    sandbox::syscall_stub(cpu)
                },
            }
        } else {
            match conf.syscalls {
                SyscallMode::Forward => translation::syscall_forward,
                SyscallMode::Sandbox => sandbox::syscall_sandbox,
                SyscallMode::Stub => sandbox::syscall_stub,
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
    fn pc(&self) -> u32 {
        self.r[15] & !0b11
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
        let Some(word) = self.mem.read_u32(self.pc()) else {
            // TODO: segfault of some kind, do more research before creating a
            // err::Err::UnallowedMemoryAccess or something
            return Ok(false);
        };

        if word == 0 {
            // zero instruction means we hit zeroed out rest of the page
            return Ok(false);
        }

        let InstructionContainer { instruction, cond } = decoder::decode_word(word, self.pc());

        // we dont execute this instruction, moving along
        if !self.cond_passes(cond) {
            self.advance();
            return Ok(true);
        }

        match instruction {
            decoder::Instruction::MovImm { rd, rhs } => {
                self.r[rd as usize] = rhs;
            }
            decoder::Instruction::Svc => {
                self.r[0] = (self.syscall_handler)(self) as u32;
            }
            decoder::Instruction::LdrLiteral { rd, addr } => {
                // buf is a guest ptr pointing to the guest pointer pointing to the literal we
                // want, thus we read the u32 addr points to to get the addr to the literal
                self.r[rd as usize] = self.mem.read_u32(addr).expect("Segfault");
            }
            decoder::Instruction::Unknown(w) => {
                return Err(err::Err::UnknownOrUnsupportedInstruction(w));
            }
            i @ _ => {
                stinkln!(
                    "found unimplemented instruction, exiting: {:#x}:={:?}",
                    word,
                    i
                );
                self.status = Some(1);
            }
        }

        self.advance();

        Ok(true)
    }
}
