use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
pub enum SyscallMode {
    /// Forward syscalls to the host system (via ARMv7->x86 translation layer)
    Forward,
    /// Stub syscalls: return success on all invocations
    Stub,
    /// Sandbox: only allow a safe subset: no file IO (except fd 0,1,2), no network, no process spawns
    Sandbox,
}

#[derive(Debug, Clone, ValueEnum, PartialEq, PartialOrd)]
pub enum Log {
    None,
    Elf,
    Segments,
    Syscalls,
    Memory,
}

#[derive(Debug, Parser)]
#[command(
    name = "stinkarm",
    about = "ARMv7 userspace binary emulator for x86 linux systems",
    version
)]
pub struct Config {
    /// Path to the ARM ELF binary to execute
    pub target: PathBuf,

    /// Syscall handling mode
    #[arg(long, value_enum, default_value_t = SyscallMode::Forward)]
    pub syscalls: SyscallMode,

    /// Stack size for the emulated process (in bytes)
    #[arg(short, long, default_value_t = 1024 * 1024)]
    pub stack_size: usize,

    /// Don't pass host env to emulated process
    #[arg(short, long)]
    pub clear_env: bool,

    /// Configure what data to log
    #[arg(short, long, value_enum, default_value_t = Log::None)]
    pub log: Log,
}
