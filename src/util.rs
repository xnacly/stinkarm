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
        $crate::stinkln!(""); // reuse the main rule
    };
    ($fmt:expr) => {
        {
            let ns = $crate::util::since_start_ns();
            println!("[{:>10.3}ms] [stinkarm] {}", ns as f64 / 1_000_000.0, $fmt);
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            let ns = $crate::util::since_start_ns();
            println!(
                "[{:>10.3}ms] [stinkarm] {}",
                ns as f64 / 1_000_000.0,
                format!($fmt, $($arg)*)
            );
        }
    };
}
