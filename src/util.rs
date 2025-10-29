use std::sync::OnceLock;
use std::time::Instant;

pub static START: OnceLock<Instant> = OnceLock::new();

pub fn init_timer() {
    START.set(Instant::now()).ok();
}

pub fn since_start_ns() -> u128 {
    START.get().map(|s| s.elapsed().as_nanos()).unwrap_or(0)
}

#[macro_export]
macro_rules! stinkln {
    () => {
        $crate::stinkln!("");
    };
    ($fmt:expr) => {
        {
            let ns = $crate::util::since_start_ns();
            println!("[{:>10.3}ms] {}", ns as f64 / 1_000_000.0, $fmt);
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            let ns = $crate::util::since_start_ns();
            println!(
                "[{:>10.3}ms] {}",
                ns as f64 / 1_000_000.0,
                format!($fmt, $($arg)*)
            );
        }
    };
}

#[macro_export]
macro_rules! le16 {
    ($bytes:expr) => {
        u16::from_le_bytes($bytes.try_into().unwrap())
    };
}

#[macro_export]
macro_rules! le32 {
    ($bytes:expr) => {
        u32::from_le_bytes($bytes.try_into().unwrap())
    };
}
