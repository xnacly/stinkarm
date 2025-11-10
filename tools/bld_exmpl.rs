use clap::Parser;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Build all ARM assembly examples into .elf binaries
#[derive(Parser)]
struct Args {
    /// Directory containing .S examples
    #[arg(long, default_value = "examples")]
    examples_dir: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let dir = Path::new(&args.examples_dir);

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("S") {
            let name = path.file_stem().unwrap().to_str().unwrap();
            let output = dir.join(format!("{}.elf", name));
            build_asm(&path, &output)?;
        }
    }

    Ok(())
}

fn build_asm(input: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Building {} -> {}", input.display(), output.display());

    let obj_file = input.with_extension("o");

    let status = Command::new("arm-none-eabi-as")
        .arg("-march=armv7-a")
        .arg(input)
        .arg("-o")
        .arg(&obj_file)
        .status()?;

    if !status.success() {
        return Err(format!("Assembler failed for {}", input.display()).into());
    }

    let status = Command::new("arm-none-eabi-ld")
        .arg("-Ttext=0x8000")
        .arg(&obj_file)
        .arg("-o")
        .arg(output)
        .status()?;

    if !status.success() {
        return Err(format!("Linker failed for {}", output.display()).into());
    }

    Ok(fs::remove_file(obj_file)?)
}
