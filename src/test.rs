use std::env;
use std::path::Path;
use std::process::Command;

#[test]
fn test_interpreter_vs_transpiler() {
    let manifest_path = match env::var("CARGO_MANIFEST_DIR") {
        Ok(s) => s,
        Err(e) => {
            assert!(
                false,
                "Could not access enviroment variable `CARGO_MANIFEST_DIR`: {e}"
            );
            return;
        }
    };

    let _ = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(manifest_path.clone() + "/Cargo.toml")
        .output();
    let exe_path = manifest_path.clone() + "/target/debug/rust-bf";
    assert!(Path::new(&exe_path).exists());

    let bf_code_dir = manifest_path.clone() + "/bf_code";
    for entry in Path::new(&bf_code_dir).read_dir().unwrap() {
        let full_path = format!("{}", entry.unwrap().path().display());
        if !full_path.ends_with(".bf") {
            continue;
        }

        let interpreter_output = Command::new(exe_path.clone())
            .arg("interpret")
            .arg(full_path.clone())
            .output()
            .unwrap();

        let _ = Command::new(exe_path.clone())
            .arg("transpile")
            .arg(full_path.clone())
            .output()
            .unwrap();
        let compiled_path = full_path.strip_suffix(".bf").unwrap();
        assert!(
            Path::new(&compiled_path).exists(),
            "Compiled binay `{}` does not exist.",
            compiled_path
        );

        let compiled_output = Command::new(compiled_path).output().unwrap();

        assert_eq!(
            interpreter_output.stdout,
            compiled_output.stdout,
            "Output for {:?} is not the same for the interpreter and the transpiled version.",
            Path::new(&full_path).file_name().unwrap()
        );
    }
}
