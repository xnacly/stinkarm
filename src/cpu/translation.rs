use crate::err;

/// sourced from https://chromium.googlesource.com/chromiumos/docs/+/master/constants/syscalls.md#arm-32_bit_EABI
#[derive(Debug)]
pub enum ArmSyscall {
    Restart = 0x00,
    Exit = 0x01,
    Fork = 0x02,
    Read = 0x03,
    Write = 0x04,
    Open = 0x05,
    Close = 0x06,
}

impl TryFrom<u32> for ArmSyscall {
    type Error = err::Err;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::Restart,
            0x01 => Self::Exit,
            0x02 => Self::Fork,
            0x03 => Self::Read,
            0x04 => Self::Write,
            0x05 => Self::Open,
            0x06 => Self::Close,
            _ => return Err(err::Err::UnknownSyscall(value)),
        })
    }
}

pub fn syscall_forward(cpu: &mut super::Cpu) -> i32 {
    match ArmSyscall::try_from(cpu.r[7]).expect("Unregistered syscall") {
        // we catch exit fully, since we need to do cleanup after the program is done
        ArmSyscall::Exit => cpu.status = Some(cpu.r[0] as i32),
        c @ _ => todo!("{:?}", c),
    }

    return 0;
}
