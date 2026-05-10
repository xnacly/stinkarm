use std::{
    path::Path,
    process::{Command, Output},
};

pub fn build_asm(
    input: &Path,
    output: &Path,
    text_addr: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let object = output.with_extension("o");

    run_tool(
        Command::new("arm-none-eabi-as")
            .arg("-march=armv7-a")
            .arg("-o")
            .arg(&object)
            .arg(input)
            .output()?,
        "assembler",
    )?;

    run_tool(
        Command::new("arm-none-eabi-ld")
            .arg(format!("-Ttext={text_addr:#x}"))
            .arg("-o")
            .arg(output)
            .arg(&object)
            .output()?,
        "linker",
    )?;

    if let Err(e) = std::fs::remove_file(&object) {
        eprintln!(
            "Warning: failed to remove temporary object {}: {e}",
            object.display()
        );
    }

    Ok(())
}

fn run_tool(output: Output, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if output.status.success() {
        return Ok(());
    }

    Err(std::io::Error::other(format!(
        "{name} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
    .into())
}
