//! Local training-example storage and post-insertion observation helpers.
//!
//! Audio is kept as PCM WAV beside the voice-memory index. The observer is deliberately bounded:
//! it reads only the focused editor, for a short window after Murmur inserts a dictation, and
//! never records background applications or sends text/audio over the network.

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

const SAMPLE_RATE: u32 = 16_000;

pub fn write_wav(path: &Path, samples: &[f32]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    let data_bytes = samples.len() as u32 * 2;
    let riff_bytes = 36 + data_bytes;
    file.write_all(b"RIFF")?;
    file.write_all(&riff_bytes.to_le_bytes())?;
    file.write_all(b"WAVEfmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&SAMPLE_RATE.to_le_bytes())?;
    file.write_all(&(SAMPLE_RATE * 2).to_le_bytes())?;
    file.write_all(&2u16.to_le_bytes())?;
    file.write_all(&16u16.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&data_bytes.to_le_bytes())?;
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        file.write_all(&value.to_le_bytes())?;
    }
    file.flush()
}
