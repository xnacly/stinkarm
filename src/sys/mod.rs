mod write;

pub use write::write;

#[repr(i32)]
pub enum Errno {
    /// Operation not permitted
    EPERM = 1,
    /// No such file or directory
    ENOENT,
    /// No such process
    ESRCH,
    /// Interrupted system call
    EINTR,
    /// I/O Error
    EIO,
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
