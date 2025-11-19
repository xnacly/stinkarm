use crate::{cpu, sys};

pub fn write(cpu: &mut cpu::Cpu, fd: u32, buf: u32, len: u32) -> i32 {
    if len == 0 {
        return 0;
    }

    let Some(buf_ptr) = cpu.mem.translate(buf) else {
        // segfault :O
        return -(sys::Errno::EFAULT as i32);
    };

    let ret: i64;
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 1_u64,
            in("rdi") fd as u64,
            in("rsi") buf_ptr as u64,
            in("rdx") len as u64,
            lateout("rax") ret,
            out("rcx") _,
            out("r11") _,
            options(nostack),
        );
    }

    ret.try_into().unwrap_or(i32::MAX)
}
