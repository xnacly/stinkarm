use crate::{err, sys};

/// sourced from https://chromium.googlesource.com/chromiumos/docs/+/master/constants/syscalls.md#arm-32_bit_EABI
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ArmSyscall {
    restart = 0x00,
    exit = 0x01,
    fork = 0x02,
    read = 0x03,
    write = 0x04,
    open = 0x05,
    close = 0x06,
}

impl TryFrom<u32> for ArmSyscall {
    type Error = err::Err;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::restart,
            0x01 => Self::exit,
            0x02 => Self::fork,
            0x03 => Self::read,
            0x04 => Self::write,
            0x05 => Self::open,
            0x06 => Self::close,
            _ => return Err(err::Err::UnknownSyscall(value)),
        })
    }
}

pub fn syscall_forward(cpu: &mut super::Cpu) -> i32 {
    match ArmSyscall::try_from(cpu.r[7]).expect("Unregistered syscall") {
        // we catch exit fully, since we need to do cleanup after the program is done
        ArmSyscall::exit => {
            cpu.status = Some(cpu.r[0] as i32);
            0
        }
        ArmSyscall::write => sys::write(cpu, cpu.r[0], cpu.r[1], cpu.r[2]),
        c @ _ => todo!("{:?}", c),
    }
}
