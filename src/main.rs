#![allow(dead_code)]

use std::{env, fs::File, io::Read};

/// emulating the processor
pub mod cpu;
/// parsing Executable and Linkable Format
pub mod elf;
/// utilities for all other modules
pub mod util;

fn main() {
    util::init_timer();

    let path = env::args().nth(1).expect("No binary path given");
    stinkln!("opening binary `{}`", path);
    let mut file = File::open(path).expect("Failed to open binary");
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .expect("Failed to dump file into memory");

    stinkln!("parsing elf...");
    let header: elf::header::Header = (&buf as &[u8])
        .try_into()
        .expect("Failed to parse binary header");
    stinkln!("\\\n{}", header);
    stinkln!("booting...");
    stinkln!("shutting down");
}
