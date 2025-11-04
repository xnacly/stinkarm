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
    ($bytes:expr) => {{
        let b: [u8; 2] = $bytes
            .try_into()
            .map_err(|_| "Failed to create u32 from 4*u8")?;
        u16::from_le_bytes(b)
    }};
}

#[macro_export]
macro_rules! le32 {
    ($bytes:expr) => {{
        let b: [u8; 4] = $bytes
            .try_into()
            .map_err(|_| "Failed to create u32 from 4*u8")?;
        u32::from_le_bytes(b)
    }};
}
