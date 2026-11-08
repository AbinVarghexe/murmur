//! Copies the sherpa-onnx runtime libraries that match the bindings next to the built program.
//! A stale copy of an older version in the same folder would be loaded instead and crash, so
//! the right files are always put there fresh.

use std::path::{Path, PathBuf};
use std::{env, fs};

fn is_runtime_library(name: &str) -> bool {
    let wanted = ["sherpa-onnx-c-api", "onnxruntime"];
    let is_library = name.ends_with(".dll") || name.ends_with(".dylib") || name.contains(".so");
    is_library && wanted.iter().any(|stem| name.contains(stem))
}

fn copy_into(source: &Path, folder: &Path) {
    if fs::create_dir_all(folder).is_ok() {
        if let Some(name) = source.file_name() {
            let _ = fs::copy(source, folder.join(name));
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=DEP_SHERPA_ONNX_LIB_DIR");
    let Ok(lib_dir) = env::var("DEP_SHERPA_ONNX_LIB_DIR") else {
        return;
    };
    // Pass the folder on to the app's build script, which bundles these libraries in the installer.
    println!("cargo:lib_dir={lib_dir}");
    let Ok(out_dir) = env::var("OUT_DIR") else {
        return;
    };
    // OUT_DIR is <target>/<profile>/build/<crate>-<hash>/out, so the profile folder is 3 levels up.
    let Some(profile_dir) = PathBuf::from(out_dir)
        .ancestors()
        .nth(3)
        .map(Path::to_path_buf)
    else {
        return;
    };

    let Ok(entries) = fs::read_dir(&lib_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_runtime_library(&name) {
            for folder in [
                profile_dir.clone(),
                profile_dir.join("deps"),
                profile_dir.join("examples"),
            ] {
                copy_into(&path, &folder);
            }
        }
    }
}
