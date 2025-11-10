use crate::sys::Errno;

pub fn write(fd: u32, buf: u32, len: u32) -> i32 {
    if len == 0 {
        return 0;
    }

    return Errno::ENOSYS as i32;
}
