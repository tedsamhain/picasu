mod generator;

use std::process::Command;

fn main() {
    let subcommand = std::env::args().nth(1);

    match subcommand.as_deref() {
        Some("emit-openapi") => emit_openapi(),
        Some("test-backend") => {
            let just_generate = std::env::args().any(|a| a == "--generate-only");
            generator::generate_all();
            if just_generate {
                return;
            }
            let status = Command::new("cargo")
                .args([
                    "nextest",
                    "run",
                    "--package",
                    "urocissa",
                    "--",
                    "scenarios_generated",
                ])
                .current_dir(workspace_root())
                .status()
                .expect("failed to run cargo nextest for backend scenarios");
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
        Some("test-generator") => {
            generator::generate_negative_tests();
            let status = Command::new("cargo")
                .args(["nextest", "run", "--", "test_generator_generated"])
                .current_dir(workspace_root())
                .status()
                .expect("failed to run cargo nextest for negative tests");
            // Write an empty placeholder so the `mod test_generator_generated`
            // declaration in mod.rs always finds the file.
            let out_path =
                workspace_root().join("gallery-backend/src/tests/test_generator_generated.rs");
            let empty = "// @generated — empty placeholder; re-run `cargo xtask test-generator`\n";
            let _ = std::fs::write(&out_path, empty);
            if !status.success() {
                eprintln!("negative tests FAILED — assertion machinery may not be catching errors");
                std::process::exit(1);
            }
            eprintln!("all negative tests passed (each panicked as expected)");
        }
        Some(other) => {
            eprintln!("unknown subcommand: {other}");
            eprintln!("available: emit-openapi, test-backend, test-generator");
            std::process::exit(1);
        }
        None => {
            eprintln!("missing subcommand");
            eprintln!("available: emit-openapi, test-backend, test-generator");
            std::process::exit(1);
        }
    }
}

fn workspace_root() -> std::path::PathBuf {
    let dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| {
        std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }));
    // CARGO_MANIFEST_DIR is xtask/; workspace root is one level up
    dir.parent().unwrap().to_path_buf()
}

fn emit_openapi() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--package",
            "urocissa",
            "--bin",
            "urocissa-openapi",
            "--features",
            "openapi",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .output()
        .expect("failed to run urocissa-openapi");

    if !output.status.success() {
        std::process::exit(output.status.code().unwrap_or(1));
    }

    let spec = String::from_utf8(output.stdout).expect("openapi output is not valid UTF-8");
    let path = std::path::Path::new("gallery-backend").join("openapi.json");
    std::fs::write(&path, spec.as_bytes()).expect("failed to write openapi.json");

    eprintln!("wrote {}", path.display());
}
