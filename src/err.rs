#[derive(Debug)]
pub enum Err {
    ElfConstraintViolation(String),
    UnknownSyscall(u32),
    UnknownOrUnsupportedInstruction(u32),
}
