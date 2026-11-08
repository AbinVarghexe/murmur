//! Before Tauri builds, copy the speech engine's runtime libraries into `libs/`, where the
//! installer picks them up (see `bundle.resources` in tauri.conf.json). They have to sit next to
//! the program: Windows looks for DLLs there first.

use std::path::Path;
use std::{env, fs};

fn is_runtime_library(name: &str) -> bool {
    let is_library = name.ends_with(".dll") || name.ends_with(".dylib");
    is_library && (name.contains("sherpa-onnx-c-api") || name.contains("onnxruntime"))
}

fn stage_libraries() {
    println!("cargo:rerun-if-env-changed=DEP_MURMUR_ENGINE_LIB_DIR");
    let Ok(source) = env::var("DEP_MURMUR_ENGINE_LIB_DIR") else {
        return;
    };
    let target = Path::new("libs");
    if fs::create_dir_all(target).is_err() {
        return;
    }
    // Start clean so a library from an older version can never be bundled by mistake.
    if let Ok(old) = fs::read_dir(target) {
        for entry in old.flatten() {
            let _ = fs::remove_file(entry.path());
        }
    }
    let Ok(entries) = fs::read_dir(&source) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_runtime_library(&name) {
            let _ = fs::copy(entry.path(), target.join(&name));
        }
    }
}

fn main() {
    stage_libraries();
    // On macOS the libraries live in the app bundle's Frameworks folder.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
    }
    tauri_build::build()
}
