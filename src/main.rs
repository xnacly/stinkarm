pub mod bus;
pub mod cpu;
pub mod elf;
pub mod mem;
pub mod util;

fn main() {
    util::init_timer();
    stinkln!("booting...");
    stinkln!("shutting down");
}
