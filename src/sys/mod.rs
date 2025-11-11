mod write;

pub use write::write;

#[repr(i32)]
#[derive(Debug)]
pub enum Errno {
    /// Operation not permitted
    EPERM = 1,
    /// No such file or directory
    ENOENT = 2,
    /// No such process
    ESRCH = 3,
    /// Interrupted system call
    EINTR = 4,
    /// I/O Error
    EIO = 5,
    /// Bad file descriptor
    EBADF = 9,
    /// Resource temporarily unavailable
    EAGAIN = 11,
    /// Out of memory
    ENOMEM = 12,
    /// Permission denied
    EACCES = 13,
    /// Bad address
    EFAULT = 14,
    /// System call unimplemented
    ENOSYS = 38,
}

impl From<i32> for Errno {
    fn from(value: i32) -> Self {
        match -value {
            1 => Self::EPERM,
            2 => Self::ENOENT,
            3 => Self::ESRCH,
            4 => Self::EINTR,
            5 => Self::EIO,
            9 => Self::EBADF,
            11 => Self::EAGAIN,
            12 => Self::ENOMEM,
            13 => Self::EACCES,
            14 => Self::EFAULT,
            38 => Self::ENOSYS,
            _ => panic!("Programming error, errno `{}` is not mapped yet!", value),
        }
    }
}
