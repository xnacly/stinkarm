use crate::cpu::translation::ArmSyscall;

const EACCES: i32 = 1;

pub fn syscall_sandbox(cpu: &mut super::Cpu) -> i32 {
    match ArmSyscall::try_from(cpu.r[7]).expect("Unregistered syscall") {
        // we catch exit fully, since we need to do cleanup after the program is done
        ArmSyscall::Exit => cpu.status = Some(cpu.r[0] as i32),
        c @ _ => todo!("{:?}", c),
    }

    return 0;
}

pub fn syscall_stub(cpu: &mut super::Cpu) -> i32 {
    match ArmSyscall::try_from(cpu.r[7]).expect("Unregistered syscall") {
        // we catch exit fully, since we need to do cleanup after the program is done
        ArmSyscall::Exit => cpu.status = Some(cpu.r[0] as i32),
        _ => (),
    }

    return -EACCES;
}
