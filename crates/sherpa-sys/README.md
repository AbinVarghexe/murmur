# murmur-sherpa-sys

Raw bindings to the [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) C API, pinned to **v2.0.1**.

Adapted from `sherpa-rs-sys` 0.6.8 (MIT, thewh1teagle). Changes: the bundled C header and the download
table (`dist.json`, `checksum.txt`) now point at sherpa-onnx v2.0.1 and its smaller "no-tts" prebuilt
libraries (text-to-speech is not used). v2.0.1 transcribes short function words ("at", "to") more
reliably than the 1.12.9 libraries that sherpa-rs-sys 0.6.8 downloads.

The C header in `sherpa-onnx/sherpa-onnx/c-api/` is from the sherpa-onnx project (Apache-2.0).
Every downloaded archive is checked against the SHA-256 in `checksum.txt` before use.
