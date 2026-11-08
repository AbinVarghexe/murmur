//! Runs 16 kHz mono WAV files through the real speech model, then the same text formatting
//! and command parsing the app uses, and prints each step with timings.
//!
//! cargo run -p murmur-engine --example probe --release -- <model-dir> <wav-dir> [name...]

use murmur_core::format::{format_transcript, FormatOptions};
use murmur_core::planner::plan_command;
use murmur_engine::Engine;
use std::fs;
use std::path::Path;
use std::time::Instant;

fn read_wav(path: &Path) -> Result<Vec<f32>, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut at = 12;
    while at + 8 <= bytes.len() {
        let id = &bytes[at..at + 4];
        let size = u32::from_le_bytes([bytes[at + 4], bytes[at + 5], bytes[at + 6], bytes[at + 7]])
            as usize;
        if id == b"data" {
            let end = (at + 8 + size).min(bytes.len());
            return Ok(bytes[at + 8..end]
                .chunks_exact(2)
                .map(|pair| f32::from(i16::from_le_bytes([pair[0], pair[1]])) / 32768.0)
                .collect());
        }
        at += 8 + size;
    }
    Err("no data chunk".into())
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (model_dir, wav_dir) = match args.as_slice() {
        [model, wavs, ..] => (model.clone(), wavs.clone()),
        _ => return Err("usage: probe <model-dir> <wav-dir> [name...]".into()),
    };
    let started = Instant::now();
    let mut engine = Engine::load(Path::new(&model_dir))?;
    println!("model loaded in {} ms\n", started.elapsed().as_millis());

    let mut names: Vec<String> = args[2..].to_vec();
    if names.is_empty() {
        names = fs::read_dir(&wav_dir)
            .map_err(|e| e.to_string())?
            .flatten()
            .filter_map(|entry| {
                entry
                    .path()
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
            })
            .filter(|name| entry_is_wav(&wav_dir, name))
            .collect();
        names.sort();
    }

    for name in names {
        let samples = read_wav(&Path::new(&wav_dir).join(format!("{name}.wav")))?;
        let started = Instant::now();
        let raw = engine.transcribe(&samples);
        let took = started.elapsed().as_millis();
        let seconds = samples.len() as f32 / 16_000.0;
        println!(
            "=== {name}  ({seconds:.1}s audio, {took} ms, {:.1}x realtime)",
            seconds * 1000.0 / took.max(1) as f32
        );
        println!("  raw:       {raw:?}");
        if name.starts_with("cmd") {
            match plan_command(&raw) {
                Some(plan) => println!(
                    "  command:   {:?}{}",
                    plan.action,
                    if plan.confirm { " [asks first]" } else { "" }
                ),
                None => println!("  command:   NOT RECOGNIZED"),
            }
        } else {
            println!(
                "  formatted: {:?}",
                format_transcript(&raw, FormatOptions::default())
            );
        }
        println!();
    }
    Ok(())
}

fn entry_is_wav(dir: &str, stem: &str) -> bool {
    Path::new(dir).join(format!("{stem}.wav")).exists()
}
