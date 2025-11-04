#![allow(dead_code)]

use std::{fs::File, io::Read};

use clap::Parser;

/// configure stinkarm via cli
pub mod config;
/// emulating the processor
pub mod cpu;
/// parsing Executable and Linkable Format
pub mod elf;
/// utilities for all other modules
pub mod util;

fn main() {
    util::init_timer();
    let conf = config::Config::parse();

    let path = conf.target;
    stinkln!("opening binary {:?}", path);
    let mut file = File::open(path).expect("Failed to open binary");
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .expect("Failed to dump file into memory");

    stinkln!("parsing elf...");
    let elf: elf::Elf = (&buf as &[u8]).try_into().expect("Failed to parse binary");

    if conf.log == config::Log::Elf {
        stinkln!("\\\n{}", elf);
    }

    stinkln!("booting...");
    stinkln!("shutting down");
}
