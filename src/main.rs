#![allow(dead_code)]
use clap::Parser;
use std::{fs::File, io::Read, process::exit};

/// configure stinkarm via cli
pub mod config;
/// emulating the processor
pub mod cpu;
/// parsing Executable and Linkable Format
pub mod elf;
pub mod err;
/// memory translation from guest to host
pub mod mem;
/// emulating and forwarding syscalls
pub mod sys;
/// utilities for all other modules
pub mod util;

fn main() {
    util::init_timer();
    let conf = config::Config::parse();

    let path = &conf.target;
    stinkln!("opening binary {:?}", path);
    let mut file = File::open(path).expect("Failed to open binary");
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .expect("Failed to dump file into memory");

    stinkln!("parsing ELF...");
    let elf: elf::Elf = (&buf as &[u8]).try_into().expect("Failed to parse binary");

    if conf.log == config::Log::Elf {
        stinkln!("\\\n{}", elf);
    }

    let mut mem = mem::Mem::new();

    for phdr in elf.pheaders {
        if phdr.r#type == elf::pheader::Type::LOAD {
            phdr.map(&buf, &mut mem)
                .expect("Mapping program header failed");

            if conf.log == config::Log::Elf {
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

    stinkln!(
        "jumping to entry G={:#X}:H={:?}, starting cpu",
        elf.header.entry,
        entry
    );

    let mut cpu = cpu::Cpu::new(&conf, &mut mem, elf.header.entry);
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

        if let Some(status) = cpu.status {
            stinkln!("got exit code `{}`, forwarding to host", status);
            break;
        }
    }

    let status = cpu.status.unwrap_or_else(|| 0);
    mem.destroy();
    exit(status);
}
