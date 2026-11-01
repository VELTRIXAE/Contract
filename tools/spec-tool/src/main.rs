//! veltrixae-spec-tool
//!
//! Reads the `contractspecv0` section embedded in a compiled Soroban WASM and
//!   * `typescript <wasm> <out_dir> <npm_name>` - writes a TypeScript client
//!     package (same generator the Stellar CLI uses for
//!     `stellar contract bindings typescript`)
//!   * `json <wasm>` - prints the interface (functions, types, errors, events)
//!     as JSON, plus the WASM sha256, to stdout
//!
//! Works fully offline; no network or keys are involved.
use sha2::{Digest, Sha256};
use soroban_spec_typescript::boilerplate::Project;
use std::{env, fs, path::PathBuf, process::exit};

fn usage() -> ! {
    eprintln!("usage:\n  veltrixae-spec-tool typescript <wasm> <out_dir> <npm_name>\n  veltrixae-spec-tool json <wasm>");
    exit(2)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        usage();
    }
    let wasm = fs::read(&args[2]).unwrap_or_else(|e| {
        eprintln!("cannot read {}: {e}", args[2]);
        exit(1)
    });
    let spec = soroban_spec::read::from_wasm(&wasm).unwrap_or_else(|e| {
        eprintln!("cannot read contract spec: {e:?}");
        exit(1)
    });

    match args[1].as_str() {
        "typescript" => {
            if args.len() != 5 {
                usage();
            }
            let out = PathBuf::from(&args[3]);
            if out.exists() {
                fs::remove_dir_all(&out).expect("clean output dir");
            }
            fs::create_dir_all(&out).expect("create output dir");
            let project: Project = out.clone().try_into().expect("extract template");
            // Contract ID, RPC URL and passphrase are left unset on purpose:
            // consumers pass them at runtime, so no network value is baked in.
            project
                .init(&args[4], None, None, None, &spec)
                .expect("generate bindings");
        }
        "json" => {
            let sha = hex(&Sha256::digest(&wasm));
            let doc = serde_json::json!({
                "wasm_sha256": sha,
                "wasm_size_bytes": wasm.len(),
                "spec": spec,
            });
            println!("{}", serde_json::to_string_pretty(&doc).unwrap());
        }
        _ => usage(),
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
