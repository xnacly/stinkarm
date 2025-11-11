use crate::{cpu::translation::ArmSyscall, sys};

pub fn syscall_sandbox(cpu: &mut super::Cpu, syscall: ArmSyscall) -> i32 {
    match syscall {
        // we catch exit fully, since we need to do cleanup after the program is done
        ArmSyscall::exit => {
            cpu.status = Some(cpu.r[0] as i32);
            0
        }
        ArmSyscall::write => {
            let (r0, r1, r2) = (cpu.r[0], cpu.r[1], cpu.r[2]);
            // only allow writing to stdout, stderr and stdin
            if r0 > 2 {
                return -(sys::Errno::ENOSYS as i32);
            }

            sys::write(cpu, r0, r1, r2)
        }
        c @ _ => todo!("{:?}", c),
    }
}

pub fn syscall_stub(cpu: &mut super::Cpu, syscall: ArmSyscall) -> i32 {
    match syscall {
        // we catch exit fully, since we need to do cleanup after the program is done
        ArmSyscall::exit => cpu.status = Some(cpu.r[0] as i32),
        _ => (),
    }

    // TODO: big big thinking moment here, should we allow write? Otherwise logs will just not
    // logged

    return -(sys::Errno::ENOSYS as i32);
}
