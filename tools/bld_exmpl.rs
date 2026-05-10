mod asm;

use clap::Parser;
use std::fs;
use std::path::Path;

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
            println!("Building {} -> {}", path.display(), output.display());
            asm::build_asm(&path, &output, 0x8000)?;
        }
    }

    Ok(())
}
