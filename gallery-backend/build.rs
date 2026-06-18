use std::fmt::Write;
use std::path::Path;

fn main() {
    println!("cargo::rerun-if-changed=../xtask/data/scenarios/backend");
    println!("cargo::rerun-if-changed=../xtask/data/scenarios/generator");

    let out_dir = std::env::var("OUT_DIR").unwrap();

    let mut lines = String::new();

    let backend_dir = workspace_root().join("xtask/data/scenarios/backend");
    let mut entries: Vec<_> = std::fs::read_dir(&backend_dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", backend_dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
        })
        .collect();
    entries.sort();

    for path in &entries {
        let name = path.file_stem().unwrap().to_str().unwrap();
        let _ = writeln!(
            lines,
            r#"#[test]
fn backend_{name}() {{
    run_backend_scenario("{name}");
}}
"#,
        );
    }

    let gen_dir = workspace_root().join("xtask/data/scenarios/generator");
    let mut entries: Vec<_> = std::fs::read_dir(&gen_dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", gen_dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
        })
        .collect();
    entries.sort();

    for path in &entries {
        let name = path.file_stem().unwrap().to_str().unwrap();
        let _ = writeln!(
            lines,
            r#"#[test]
fn generator_{name}() {{
    run_generator_scenario("{name}");
}}
"#,
        );
    }

    let out_path = Path::new(&out_dir).join("scenarios.rs");
    std::fs::write(&out_path, &lines)
        .unwrap_or_else(|e| panic!("write {}: {e}", out_path.display()));
}

fn workspace_root() -> std::path::PathBuf {
    let dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| {
        std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }));
    dir.parent().unwrap().to_path_buf()
}
