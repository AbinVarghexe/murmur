# Contributing to Murmur

We welcome contributions from the community! This guide walks you through setting up your development environment, running tests, and opening pull requests.

---

## 1. Prerequisites

### Windows Development Machine
1. **Rust (stable):** Install via [rustup.rs](https://rustup.rs).
2. **Node.js (v20+ or v22+):** Install via [nodejs.org](https://nodejs.org).
3. **LLVM / libclang:** Required by the speech-engine bindgen.
   - Install LLVM or point `$env:LIBCLANG_PATH` to your LLVM bin directory.
4. **C++ Build Tools:** Visual Studio 2022 Build Tools with the *Desktop development with C++* workload.

---

## 2. Project Layout

```
.
├── crates/
│   ├── murmur-core/     # Pure Rust matching, formatting, and memory rules (135+ tests)
│   ├── murmur-engine/   # Audio capture & Sherpa-ONNX ASR execution pipeline
│   └── sherpa-sys/      # Native bindings to sherpa-onnx and onnxruntime
├── src-tauri/           # Tauri v2 desktop application, windows, tray, hotkeys
├── src/renderer/        # React 19 + Tailwind CSS v4 frontend (Beautiful UI)
│   ├── components/      # Beautiful UI primitives, studio workbench, WebGL GLSL shaders
│   ├── lib/             # Local neural formatter and voice profile memory
│   ├── pill/            # Floating HUD pill window
│   └── settings/        # Main IDE window & settings cards
└── docs/                # Architecture and user documentation
```

---

## 3. Local Development

### 1. Install Dependencies
```powershell
npm install
```

### 2. Run Test Suites
Ensure both the frontend and Rust core test suites pass:
```powershell
# Frontend TypeScript unit tests
npm test

# Rust engine & memory rules
cargo test --workspace
```

### 3. Run in Development Mode
To launch the desktop application with live reload:
```powershell
npm run tauri -- dev
```

To preview only the frontend in a web browser with the development bridge:
```powershell
npm run ui:dev
```

### 4. Build Production Installer
```powershell
npm run tauri -- build
```
The installer will be generated in `target/release/bundle/nsis/`.

---

## 4. Code Standards

- **Rust:** Format with `cargo fmt` and verify with `cargo clippy`.
- **TypeScript:** Strict type checking with `npm run typecheck`.
- **UI Guidelines:** Follow Beautiful UI rules — semantic color tokens only, composite typography classes, and no hardcoded palette hexes.
- **Privacy:** Murmur is 100% offline. Never introduce external network calls, telemetry, or remote telemetry logging into the dictation or memory pipeline.
