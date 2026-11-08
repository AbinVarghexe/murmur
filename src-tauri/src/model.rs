//! The speech model: where it lives, whether it is complete, and a verified download.
//!
//! NVIDIA Parakeet TDT 0.6B (int8) converted for sherpa-onnx. The download is pinned to an
//! exact revision and every file is checked against its SHA-256, so a corrupted or tampered
//! file is discarded and never loaded.

use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const REVISION: &str = "1ab9323565ddb038682214b292f588070a538ce2";
const BASE_URL: &str =
    "https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v2-int8/resolve";
const FOLDER: &str = "parakeet-tdt-0.6b-v2-int8";
const CHUNK: usize = 64 * 1024;
const STALL_TIMEOUT: Duration = Duration::from_secs(30);

pub struct ModelFile {
    pub name: &'static str,
    pub size: u64,
    pub sha256: &'static str,
}

pub const FILES: [ModelFile; 4] = [
    ModelFile {
        name: "tokens.txt",
        size: 9_384,
        sha256: "ec182b70dd42113aff6c5372c75cac58c952443eb22322f57bbd7f53977d497d",
    },
    ModelFile {
        name: "joiner.int8.onnx",
        size: 1_739_080,
        sha256: "7946164367946e7f9f29a122407c3252b680dbae9a51343eb2488d057c3c43d2",
    },
    ModelFile {
        name: "decoder.int8.onnx",
        size: 7_257_753,
        sha256: "b6bb64963457237b900e496ee9994b59294526439fbcc1fecf705b31a15c6b4e",
    },
    ModelFile {
        name: "encoder.int8.onnx",
        size: 652_184_296,
        sha256: "a32b12d17bbbc309d0686fbbcc2987b5e9b8333a7da83fa6b089f0a2acd651ab",
    },
];

pub fn total_bytes() -> u64 {
    FILES.iter().map(|f| f.size).sum()
}

/// `<app data>/models/parakeet-tdt-0.6b-v2-int8`
pub fn model_dir(app_data: &Path) -> PathBuf {
    app_data.join("models").join(FOLDER)
}

fn has_exact_size(path: &Path, size: u64) -> bool {
    fs::metadata(path).map(|m| m.len() == size).unwrap_or(false)
}

pub fn is_ready(dir: &Path) -> bool {
    FILES
        .iter()
        .all(|f| has_exact_size(&dir.join(f.name), f.size))
}

fn download_file(
    file: &ModelFile,
    dir: &Path,
    agent: &ureq::Agent,
    cancel: &AtomicBool,
    on_bytes: &mut dyn FnMut(u64),
) -> Result<(), String> {
    let destination = dir.join(file.name);
    if has_exact_size(&destination, file.size) {
        on_bytes(file.size);
        return Ok(());
    }
    let url = format!("{BASE_URL}/{REVISION}/{}", file.name);
    let response = agent
        .get(&url)
        .call()
        .map_err(|e| format!("Download failed for {}: {e}", file.name))?;
    let mut reader = response.into_reader();

    let partial = dir.join(format!("{}.part", file.name));
    let result = (|| -> Result<(), String> {
        let mut out =
            File::create(&partial).map_err(|e| format!("Cannot write {}: {e}", file.name))?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0_u8; CHUNK];
        let mut written = 0_u64;
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err("Download cancelled".into());
            }
            let read = reader
                .read(&mut buffer)
                .map_err(|e| format!("Download stalled or failed: {e}"))?;
            if read == 0 {
                break;
            }
            written += read as u64;
            if written > file.size {
                return Err(format!(
                    "{} is larger than expected; download aborted",
                    file.name
                ));
            }
            hasher.update(&buffer[..read]);
            out.write_all(&buffer[..read])
                .map_err(|e| format!("Cannot write {}: {e}", file.name))?;
            on_bytes(read as u64);
        }
        out.flush().map_err(|e| e.to_string())?;
        let digest = hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        if digest != file.sha256 {
            return Err(format!(
                "Checksum mismatch for {}; the file was discarded",
                file.name
            ));
        }
        Ok(())
    })();

    match result {
        Ok(()) => fs::rename(&partial, &destination)
            .map_err(|e| format!("Cannot finish {}: {e}", file.name)),
        Err(error) => {
            let _ = fs::remove_file(&partial);
            Err(error)
        }
    }
}

/// Download and verify every model file. Finished files are skipped, so it is safe to retry.
pub fn download(
    dir: &Path,
    cancel: &AtomicBool,
    mut progress: impl FnMut(u64, u64),
) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("Cannot create the model folder: {e}"))?;
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(STALL_TIMEOUT)
        .build();
    let total = total_bytes();
    let mut received = 0_u64;
    for file in &FILES {
        download_file(file, dir, &agent, cancel, &mut |n| {
            received += n;
            progress(received, total);
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_size_is_the_sum_of_the_files() {
        assert_eq!(total_bytes(), 9_384 + 1_739_080 + 7_257_753 + 652_184_296);
    }

    #[test]
    fn an_empty_folder_is_not_ready() {
        let dir = std::env::temp_dir().join("murmur-test-empty-model");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        assert!(!is_ready(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_of_the_wrong_size_does_not_count() {
        let dir = std::env::temp_dir().join("murmur-test-wrong-size-model");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        for file in &FILES {
            fs::write(dir.join(file.name), b"x").unwrap();
        }
        assert!(!is_ready(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_model_lives_under_the_app_data_folder() {
        let dir = model_dir(Path::new("/data/Murmur"));
        assert!(dir.ends_with(Path::new("models").join(FOLDER)));
    }
}
