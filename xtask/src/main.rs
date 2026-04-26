use std::env;
use std::process::{exit, Command};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo xtask <command>");
        eprintln!("  generate-headers  -- regenerate include/sokr.h via cbindgen");
        exit(1);
    }

    match args[1].as_str() {
        "generate-headers" => generate_headers(),
        cmd => {
            eprintln!("Unknown command: {}", cmd);
            eprintln!("  generate-headers  -- regenerate include/sokr.h via cbindgen");
            exit(1);
        }
    }
}

fn generate_headers() {
    let root = env!("CARGO_MANIFEST_DIR");
    let root_path = std::path::PathBuf::from(root);
    let workspace_root = root_path.parent().unwrap();

    let status = Command::new("cbindgen")
        .args(["--crate", "sokr", "--output", "include/sokr.h"])
        .current_dir(workspace_root)
        .status()
        .expect("cbindgen not found — install with `cargo install cbindgen`");

    if status.success() {
        println!("Generated {}", workspace_root.join("include/sokr.h").display());
    } else {
        eprintln!("cbindgen failed with status {}", status);
        exit(1);
    }
}
