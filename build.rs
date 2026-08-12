use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let tests_dir = manifest_dir.join("tests");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("asm-tests");
    fs::create_dir_all(&out_dir).expect("failed to create generated test directory");

    let mut sources: Vec<_> = fs::read_dir(&tests_dir)
        .expect("failed to read tests directory")
        .map(|entry| entry.expect("failed to read test entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "s"))
        .collect();
    sources.sort();

    let mut generated = String::from(
        "use std::{path::Path, process::{Command, Output}};\n\n\
         fn run_case(elf: &Path, args: &[&str]) -> Output {\n\
             Command::new(env!(\"CARGO_BIN_EXE_stinkarm\"))\n\
                 .args(args)\n\
                 .arg(elf)\n\
                 .output()\n\
                 .expect(\"failed to run stinkarm\")\n\
         }\n\n",
    );

    for source in sources {
        println!("cargo:rerun-if-changed={}", source.display());
        let spec = TestSpec::parse(&source);
        let name = source.file_stem().unwrap().to_str().unwrap();
        let object = out_dir.join(format!("{name}.o"));
        let elf = out_dir.join(format!("{name}.elf"));

        run_tool(
            Command::new("arm-none-eabi-as")
                .arg("-march=armv7-a")
                .arg("-o")
                .arg(&object)
                .arg(&source),
        );
        run_tool(
            Command::new("arm-none-eabi-ld")
                .arg(format!("-Ttext={:#x}", spec.address))
                .arg("-o")
                .arg(&elf)
                .arg(&object),
        );

        generated.push_str(&format!("#[test]\nfn {name}() {{\n"));
        generated.push_str(&format!(
            "    let output = run_case(Path::new({}), &[{}]);\n",
            rust_string(&elf.display().to_string()),
            spec.args
                .iter()
                .map(|arg| rust_string(arg))
                .collect::<Vec<_>>()
                .join(", "),
        ));
        if let Some(exit) = spec.exit {
            generated.push_str(&format!(
                "    assert_eq!(output.status.code(), Some({exit}));\n"
            ));
        }
        if let Some(success) = spec.success {
            generated.push_str(&format!(
                "    assert_eq!(output.status.success(), {success});\n"
            ));
        }
        if let Some(stdout) = spec.stdout {
            generated.push_str(&format!(
                "    assert_eq!(output.stdout, {}.as_bytes());\n",
                rust_string(&stdout)
            ));
        }
        for text in spec.stdout_contains {
            generated.push_str(&format!(
                "    assert!(String::from_utf8_lossy(&output.stdout).contains({}));\n",
                rust_string(&text)
            ));
        }
        for text in spec.stdout_not_contains {
            generated.push_str(&format!(
                "    assert!(!String::from_utf8_lossy(&output.stdout).contains({}));\n",
                rust_string(&text)
            ));
        }
        for text in spec.stderr_contains {
            generated.push_str(&format!(
                "    assert!(String::from_utf8_lossy(&output.stderr).contains({}));\n",
                rust_string(&text)
            ));
        }
        generated.push_str("}\n\n");
    }

    fs::write(out_dir.join("generated_tests.rs"), generated)
        .expect("failed to write generated tests");
}

#[derive(Default)]
struct TestSpec {
    address: u32,
    args: Vec<String>,
    exit: Option<i32>,
    success: Option<bool>,
    stdout: Option<String>,
    stdout_contains: Vec<String>,
    stdout_not_contains: Vec<String>,
    stderr_contains: Vec<String>,
}

impl TestSpec {
    fn parse(path: &Path) -> Self {
        let source = fs::read_to_string(path).expect("failed to read assembly test");
        let line = source
            .lines()
            .find_map(|line| line.trim().strip_prefix("@ stinkarm-test: "))
            .unwrap_or_else(|| panic!("{} is missing a stinkarm-test directive", path.display()));
        let mut spec = Self::default();

        for field in line
            .split(';')
            .map(str::trim)
            .filter(|field| !field.is_empty())
        {
            let (key, value) = field
                .split_once('=')
                .unwrap_or_else(|| panic!("invalid test field `{field}`"));
            let value = unescape(value);
            match key {
                "address" => {
                    spec.address = u32::from_str_radix(value.trim_start_matches("0x"), 16)
                        .expect("invalid address")
                }
                "args" => spec.args = value.split_whitespace().map(str::to_owned).collect(),
                "exit" => spec.exit = Some(value.parse().expect("invalid exit status")),
                "success" => spec.success = Some(value.parse().expect("invalid success value")),
                "stdout" => spec.stdout = Some(value),
                "stdout-contains" => spec.stdout_contains.push(value),
                "stdout-not-contains" => spec.stdout_not_contains.push(value),
                "stderr-contains" => spec.stderr_contains.push(value),
                _ => panic!("unknown test field `{key}`"),
            }
        }
        spec
    }
}

fn unescape(value: &str) -> String {
    value.replace("\\n", "\n")
}

fn rust_string(value: &str) -> String {
    format!("{value:?}")
}

fn run_tool(command: &mut Command) {
    let output = command
        .output()
        .unwrap_or_else(|error| panic!("failed to run {command:?}: {error}"));
    if !output.status.success() {
        panic!(
            "{command:?} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
