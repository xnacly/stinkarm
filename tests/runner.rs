use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn valid_write_and_exit_runs_to_completion() {
    let elf = build_case("valid_write_exit.s", 0x8000);
    let output = run_stinkarm(&elf, &[]);

    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout, b"ok\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn write_from_null_guest_pointer_fails_without_host_memory_access() {
    let elf = build_case("write_null.s", 0x8000);
    let output = run_stinkarm(&elf, &["--log", "syscalls"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(stdout.contains("=EFAULT"));
    assert!(!stdout.contains("ignored"));
}

#[test]
fn write_past_guest_memory_fails_without_host_memory_access() {
    let elf = build_case("write_oob.s", 0x8000);
    let output = run_stinkarm(&elf, &["--log", "syscalls"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(stdout.contains("=EFAULT"));
    assert!(!stdout.contains("ignored"));
}

#[test]
fn load_segment_at_guest_null_page_is_rejected() {
    let elf = build_case("load_at_null.s", 0);
    let output = run_stinkarm(&elf, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(stderr.contains("program header has a zero virtual address"));
}

fn build_case(case: &str, text_addr: u32) -> PathBuf {
    let dir = temp_case_dir(case);
    fs::create_dir_all(&dir).expect("failed to create test temp dir");

    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(case);
    let object = dir.join("case.o");
    let elf = dir.join("case.elf");

    run_tool(
        Command::new("arm-none-eabi-as")
            .arg("-march=armv7-a")
            .arg("-o")
            .arg(&object)
            .arg(&source),
    );
    run_tool(
        Command::new("arm-none-eabi-ld")
            .arg(format!("-Ttext={text_addr:#x}"))
            .arg("-o")
            .arg(&elf)
            .arg(&object),
    );

    elf
}

fn run_stinkarm(elf: &Path, args: &[&str]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_stinkarm"))
        .args(args)
        .arg(elf)
        .output()
        .expect("failed to run stinkarm");

    if let Some(dir) = elf.parent() {
        fs::remove_dir_all(dir).expect("failed to remove test temp dir");
    }

    output
}

fn run_tool(command: &mut Command) {
    let output = command.output().unwrap_or_else(|e| {
        panic!(
            "failed to run {:?}: {e}. Enter the flake dev shell with `nix develop`.",
            command
        )
    });

    if !output.status.success() {
        panic!(
            "{:?} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
            command,
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn temp_case_dir(case: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    let case = case.trim_end_matches(".s");
    path.push(format!("stinkarm-{case}-{}-{now}", std::process::id()));
    path
}
