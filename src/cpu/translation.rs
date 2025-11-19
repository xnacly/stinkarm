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

impl ArmSyscall {
    // TODO: rework this with some kind of writer
    pub fn print(&self, cpu: &super::Cpu) -> String {
        let mut buf = String::with_capacity(32);
        buf.push_str(&format!("{} {:?}(", std::process::id(), self));
        let args = match self {
            ArmSyscall::exit => format!("code={}", cpu.r[0]),
            ArmSyscall::fork => todo!(),
            ArmSyscall::read => todo!(),
            ArmSyscall::write => format!("fd={}, buf={:#x}, len={}", cpu.r[0], cpu.r[1], cpu.r[2]),
            ArmSyscall::open => todo!(),
            ArmSyscall::close => todo!(),
            _ => "unimplemented".into(),
        };
        buf.push_str(&args);
        buf.push(')');
        buf.shrink_to_fit();
        buf
    }
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

pub fn syscall_forward(cpu: &mut super::Cpu, syscall: ArmSyscall) -> i32 {
    match syscall {
        // we catch exit fully, since we need to do cleanup after the program is done
        ArmSyscall::exit => {
            cpu.status = Some(cpu.r[0] as i32);
            0
        }
        ArmSyscall::write => sys::write(cpu, cpu.r[0], cpu.r[1], cpu.r[2]),
        c => todo!("{:?}", c),
    }
}
