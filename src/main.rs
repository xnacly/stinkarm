#![allow(dead_code)]
use clap::Parser;
use std::{fs::File, io::Read, process::exit};

use crate::config::Log;

/// configure stinkarm via cli
pub mod config;
/// emulating the processor
pub mod cpu;
/// parsing Executable and Linkable Format
pub mod elf;
pub mod err;
/// memory translation from guest to host
pub mod mem;
/// emulating, forwarding and implementing syscalls
pub mod sys;
/// utilities for all other modules
pub mod util;

fn main() {
    util::init_timer();
    let mut conf = config::Config::parse();

    let path = &conf.target;
    if conf.verbose {
        stinkln!("opening binary {:?}", path);
        conf.log
            .extend_from_slice(&[Log::Elf, Log::Syscalls, Log::Memory]);
    }
    let mut file = File::open(path).expect("Failed to open binary");
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .expect("Failed to dump file into memory");

    if conf.verbose {
        stinkln!("parsing ELF...");
    }
    let elf: elf::Elf = (&buf as &[u8]).try_into().expect("Failed to parse binary");

    if conf.log.contains(&config::Log::Elf) {
        stinkln!("\\\n{}", elf);
    }

    let mut mem = mem::Mem::new();

    for phdr in elf.pheaders {
        if phdr.r#type == elf::pheader::Type::LOAD {
            phdr.map(&buf, &mut mem)
                .expect("Mapping program header failed");

            if conf.log.contains(&config::Log::Elf) {
                stinkln!(
                    "mapped program header `{:?}` of {}B (G={:#X} -> H={:?})",
                    phdr.r#type,
                    phdr.memsz,
                    phdr.vaddr,
                    mem.translate(phdr.vaddr).unwrap_or(std::ptr::null_mut())
                );
            }
        }
    }

    let entry = mem
        .translate(elf.header.entry)
        .unwrap_or(std::ptr::null_mut());

    if conf.verbose {
        stinkln!(
            "jumping to entry G={:#X} at H={:?}",
            elf.header.entry,
            entry
        );
    }

    let mut cpu = cpu::Cpu::new(&conf, &mut mem, elf.header.entry);
    if conf.verbose {
        stinkln!("starting the emulator");
    }
    loop {
        match cpu.step() {
            // EOI - end of instructions :^)
            Ok(false) => break,
            Err(err) => {
                println!("err: `{:?}`, exiting emulation", err);
                break;
            }
            Ok(true) => {}
        }

        if cpu.status.is_some() {
            break;
        }
    }

    let status = cpu.status.unwrap_or_else(|| 0);
    mem.destroy();
    if conf.verbose {
        stinkln!("exiting with `{}`", status);
    }
    exit(status);
}
