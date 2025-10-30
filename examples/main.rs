#![no_std]
#![no_main]

#[cfg(all(target_arch = "arm", target_feature = "v7"))]
mod armv7_entry {
    use core::arch::asm;

    #[unsafe(no_mangle)]
    pub extern "C" fn _start() -> ! {
        let ret = main();
        unsafe {
            const SYS_EXIT: u32 = 1;

            asm!(
                "mov r0, {0}",
                "mov r7, {1}",
                "svc #0",
                in(reg) ret,
                const SYS_EXIT,
                options(noreturn)
            );
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn main() -> i32 {
        161
    }
}

#[cfg(not(all(target_arch = "arm", target_feature = "v7")))]
compile_error!("This binary must be built for ARMv7 (e.g., --target armv7-unknown-linux-gnueabi)");
