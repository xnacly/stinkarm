#[derive(Debug)]
pub enum Err {
    // TODO: return this from the whole ELF parsing aparatus
    ElfConstraintViolation(String),
    UnknownOrUnsupportedInstruction(u32),
}
