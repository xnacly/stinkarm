#[repr(u16)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// This member’s value specifies the required architecture for an individual file.
/// https://gabi.xinuos.com/elf/02-eheader.html#contents-of-the-elf-header and https://gabi.xinuos.com/elf/a-emachine.html
pub enum Machine {
    EM_ARM = 40,
}

impl TryFrom<u16> for Machine {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            40 => Ok(Machine::EM_ARM),
            _ => Err("Unsupported machine"),
        }
    }
}
