#[derive(Debug)]
pub enum Err {
    ElfConstraintViolation(String),
    UnknownSyscall(u32),
    UnknownOrUnsupportedInstruction(u32),
    MemoryAccessViolation { guest: u32, instr: u32 },
}
