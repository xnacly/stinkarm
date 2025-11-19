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
            in("rax") 1,
            in("rdi") fd,
            in("rsi") buf_ptr,
            in("rdx") len,
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
    }

    ret as i32
}
