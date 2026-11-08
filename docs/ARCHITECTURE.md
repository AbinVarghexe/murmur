# Murmur Architecture & Engineering Specification

Murmur is an on-device, zero-cloud voice dictation and system action environment for Windows. It couples a low-latency automatic speech recognition (ASR) pipeline with a continuous self-learning memory engine ("WhisperFlow" architecture) and a WebGL GLSL-driven desktop HUD.

---

## 1. System Architecture Overview

```
 ┌────────────────────────────────────────────────────────────────────────┐
 │                              Windows OS                                │
 │  Focused Window / Foreground Application (Editor, Browser, Terminal)  │
 └─────────────┬────────────────────────────────────────────▲─────────────┘
               │                                            │
        Audio Capture (CPAL)                         Simulated Typing /
        16 kHz Mono PCM                               Clipboard Paste
               │                                            │
               ▼                                            │
 ┌──────────────────────────────────────────────────────────┴─────────────┐
 │                         murmur-core & src-tauri                        │
 │                                                                        │
 │  ┌───────────────────────┐        ┌─────────────────────────────────┐  │
 │  │      Silero VAD       │        │         Action Planner          │  │
 │  │ Voice Activity Engine │        │ AST Parser · App Matcher (Apps) │  │
 │  └──────────┬────────────┘        └────────────────▲────────────────┘  │
 │             │ Speech Segments                      │ Cleaned Text      │
 │             ▼                                      │                   │
 │  ┌───────────────────────┐        ┌────────────────┴────────────────┐  │
 │  │ Sherpa-ONNX (TDT 0.6B)│───────▶│    WhisperFlow Memory Engine    │  │
 │  │ Parakeet Transducer   │ Raw    │ Lexicon Biasing · Phonetic LCS  │  │
 │  └───────────────────────┘ Text   └────────────────▲────────────────┘  │
 │                                                    │                   │
 │                                            Editor Observation          │
 │                                            (Zero-Click Learn)          │
 └─────────────────────────────┬──────────────────────┴───────────────────┘
                               │
                       IPC Events & State
                               │
                               ▼
 ┌────────────────────────────────────────────────────────────────────────┐
 │                      Tauri v2 / React 19 Frontend                      │
 │                                                                        │
 │   ┌───────────────────────────┐      ┌─────────────────────────────┐   │
 │   │  Compact Floating Pill    │      │     Voice Studio & IDE      │   │
 │   │  WebGL GLSL SiriWave HUD  │      │  Beautiful UI Design System │   │
 │   └───────────────────────────┘      └─────────────────────────────┘   │
 └────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Core Subsystems

### 2.1 Audio Ingestion & Voice Activity Detection (VAD)
- **Engine:** `cpal` with WASAPI loop on Windows.
- **Format:** 16,000 Hz, 16-bit Mono PCM float samples (`f32`).
- **Voice Activity Detection:** Powered by Silero VAD running chunked acoustic windows (512 samples / 32 ms).
- **Latching Modes:** Supports both Push-to-Talk (press-and-hold with auto-finish on release) and Tap-to-Latch (tap to begin recording, speak, then auto-finish after silence timeout or second tap).

### 2.2 Automatic Speech Recognition (ASR)
- **Model:** NeMo Parakeet-TDT-0.6B (Transducer-Duration-Time) quantized to INT8.
- **Runtime:** `sherpa-onnx` static/dynamic C-API bindings with multi-threaded CPU execution.
- **Latency Floor:** Sub-150 ms acoustic response on standard x64 processors.
- **Memory Footprint:** ~660 MB weights resident in RAM; zero data is transmitted over local or wide-area networks.

### 2.3 WhisperFlow Continuous Memory Engine
Rather than allowing the acoustic decoder to guess phonetically ambiguous proper nouns, Murmur layers a personal entity biasing and learning engine:

1. **Compound Word & Multi-Word Sliding Windowing:**
   - Speech decoders often split compound proper nouns into separate tokens (e.g. spoken `"anti gravity"` vs `"Antigravity"`).
   - Murmur implements compact key normalization:
     $$\text{compact\_key}(w) = \{ c \in w \mid c \text{ is alphanumeric} \}_{\text{lowercase}}$$
   - Sliding multi-word spans ($w_{\text{min}} \le w \le w_{\text{max}}$) evaluate character-level identity regardless of token spacing.
2. **Phonetic & Levenshtein Similarity:**
   - Multi-stage candidate scoring combining:
     $$\text{Score} = 0.55 \cdot \text{Levenshtein} + 0.25 \cdot \text{PhoneticMetaphone} + 0.20 \cdot \text{ExactScore}$$
   - Conflicts require a minimum margin ($\ge 0.08$) to prevent false replacements.
3. **Zero-Click Keyboard Correction Observation:**
   - When text is inserted into a target application, Murmur spawns a scoped observer on the focused window.
   - If the user backspaces or types a correction within 90 seconds, the engine performs Longest Common Subsequence (LCS) token diffing, isolates the misheard/intended pair, updates `voice-memory.json`, and promotes the rule upon repeated evidence.
4. **Spoken Backtracking:**
   - Detects live speech retraction cues (*"scratch that"*, *"never mind"*, *"sorry I mean X"*, *"correction X"*) and updates the transcript in real time.

### 2.4 Action Planner & Typo-Tolerant Launcher
- Translates voice phrases into executable actions:
  - System volume, media controls, active window management.
  - Browser queries and mail drafts.
  - Desktop application launches.
- **Windows Store & App Execution Aliases:**
  - Scans both standard Registry paths and `%LOCALAPPDATA%\Microsoft\WindowsApps` (e.g., resolving `chatgpt-desktop.exe`).
  - Levenshtein-tolerant fuzzy matching prevents typo failures.
- **Disruptive Action Safety:**
  - Destructive commands (closing windows, shell scripts) are trapped in an `ApprovalCard` requiring explicit user confirmation.

### 2.5 WebGL GLSL Shaders & UI Layer
- **Component System:** Built using Beautiful UI (React 19 + Tailwind CSS v4) with semantic token pairs for automatic dark mode.
- **HUD Pill:** Compact $280 \times 48\text{ px}$ rounded capsule.
- **Active SiriWave Shader:** Single fullscreen WebGL canvas rendering GLSL fragment shaders with chromatic aberration, cosine amplitude envelopes, and low/mid/high voice frequency harmonics.

---

## 3. Data Flow & Security Guarantees

| Security Property | Implementation Details |
| :--- | :--- |
| **Network Isolation** | 0 external network requests during transcription or memory learning. |
| **Microphone Access** | Mic stream is opened only upon hotkey trigger and freed immediately upon final chunk. |
| **Storage Locality** | Personal lexicon and acoustic profiles reside in `%APPDATA%\com.murmur.desktop\voice-memory.json`. |
| **Audio Retention** | Audio samples remain in volatile memory; temporary WAV samples for user teaching are stored in local app data and never uploaded. |
