use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

pub fn build_source(
    input: &Path,
    output: &Path,
    text_addr: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let object = output.with_extension("o");
    let objects = compile(input, &object)?;
    link(&objects, output, text_addr)?;

    for object in objects {
        if let Err(e) = fs::remove_file(&object) {
            eprintln!(
                "Warning: failed to remove temporary object {}: {e}",
                object.display()
            );
        }
    }

    Ok(())
}

fn compile(input: &Path, object: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    match input.extension().and_then(|s| s.to_str()) {
        Some("S" | "s") => {
            assemble(input, object)?;
            Ok(vec![object.to_path_buf()])
        }
        Some("c") => compile_c(input, object),
        Some(ext) => Err(format!("unsupported input extension .{ext}").into()),
        None => Err("input path has no extension".into()),
    }
}

fn assemble(input: &Path, object: &Path) -> Result<(), Box<dyn std::error::Error>> {
    run_tool(
        Command::new("arm-none-eabi-as")
            .arg("-march=armv7-a")
            .arg("-o")
            .arg(&object)
            .arg(input)
            .output()?,
        "assembler",
    )?;

    Ok(())
}

fn compile_c(input: &Path, object: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    run_tool(
        Command::new("arm-none-eabi-gcc")
            .arg("-march=armv7-a")
            .arg("-ffreestanding")
            .arg("-nostdlib")
            .arg("-c")
            .arg("-o")
            .arg(object)
            .arg(input)
            .output()?,
        "C compiler",
    )?;

    let crt0 = object.with_file_name("crt0.o");
    assemble_crt0(&crt0)?;

    Ok(vec![crt0, object.to_path_buf()])
}

fn assemble_crt0(object: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source = object.with_file_name("crt0.S");
    fs::write(
        &source,
        r#"
    .global _start
_start:
    bl main
    mov r7, #1
    svc #0
"#,
    )?;
    assemble(&source, object)?;
    fs::remove_file(source)?;
    Ok(())
}

fn link(
    objects: &[PathBuf],
    output: &Path,
    text_addr: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::new("arm-none-eabi-ld");
    command
        .arg(format!("-Ttext={text_addr:#x}"))
        .arg("-o")
        .arg(output);
    command.args(objects);

    run_tool(command.output()?, "linker")?;

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
