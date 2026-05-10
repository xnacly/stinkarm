mod asm;

use clap::Parser;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

/// Build, link, and execute an ARM assembly file with stinkarm
#[derive(Parser)]
struct Args {
    /// ARM assembly file to run
    input: PathBuf,

    /// Guest address used as the linker text address
    #[arg(long, default_value = "0x8000", value_parser = parse_u32)]
    text_addr: u32,

    /// Directory for generated object and ELF files
    #[arg(long, default_value = "target/run_asm")]
    out_dir: PathBuf,

    /// Extra arguments passed to stinkarm before the generated ELF path
    #[arg(last = true)]
    emulator_args: Vec<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("run_asm: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<u8, Box<dyn std::error::Error>> {
    let args = Args::parse();
    let input = args.input.canonicalize()?;
    fs::create_dir_all(&args.out_dir)?;

    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("input path has no valid file stem")?;
    let elf = args.out_dir.join(format!("{stem}.elf"));

    asm::build_asm(&input, &elf, args.text_addr)?;
    execute(&elf, &args.emulator_args)
}

fn execute(elf: &Path, emulator_args: &[String]) -> Result<u8, Box<dyn std::error::Error>> {
    let emulator = ensure_stinkarm_path()?;
    let status = Command::new(&emulator)
        .args(emulator_args)
        .arg(elf)
        .status()?;

    Ok(status.code().unwrap_or(1).try_into().unwrap_or(1))
}

fn ensure_stinkarm_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let current = std::env::current_exe()?;
    let dir = current.parent().ok_or("current executable has no parent")?;
    let exe_suffix = std::env::consts::EXE_SUFFIX;
    let emulator = dir.join(format!("stinkarm{exe_suffix}"));

    if emulator.exists() {
        return Ok(emulator);
    }

    let mut build = Command::new("cargo");
    build.arg("build").arg("--bin").arg("stinkarm");

    if current
        .components()
        .any(|component| component.as_os_str() == "release")
    {
        build.arg("--release");
    }

    let status = build.status()?;
    if !status.success() {
        return Err("failed to build stinkarm".into());
    }

    Ok(emulator)
}

fn parse_u32(raw: &str) -> Result<u32, String> {
    if let Some(hex) = raw.strip_prefix("0x") {
        u32::from_str_radix(hex, 16).map_err(|e| e.to_string())
    } else {
        raw.parse::<u32>().map_err(|e| e.to_string())
    }
}
