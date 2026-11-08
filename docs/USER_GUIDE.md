# Murmur User Guide

A practical guide to configuring, using, and mastering Murmur on Windows.

---

## 1. Quick Start

### Installation
1. Download the latest installer `Murmur_2.0.0_x64-setup.exe` from the [Releases](https://github.com/AbinVarghexe/murmur/releases) page.
2. Run the installer. On first launch, Murmur will verify or download the offline ASR model weights (~660 MB).
3. The floating Murmur Pill HUD will appear in standby mode at the top or bottom of your screen.

---

## 2. Default Shortcuts & Dictation Modes

Murmur operates in two distinct operational modes:

| Mode | Default Hotkey | Function | Visual Indicator |
| :--- | :--- | :--- | :--- |
| **Dictation Mode** | `Ctrl + Alt + Space` | Transcribes spoken audio, formats grammar/casing, and pastes into focused application. | Cyan / Blue WebGL SiriWave |
| **Agent / Action Mode** | `Ctrl + Alt + A` | Interprets voice commands (launch apps, adjust volume, manage windows, media). | Amber / Orange Fluid Dots |

### Trigger Methods
- **Tap to Speak (Latched):** Press the hotkey once, release, speak naturally. When you finish speaking, Murmur's VAD automatically detects the pause, transcribes, and inserts the text.
- **Push-to-Talk (Hold):** Hold the hotkey down while speaking. Release when finished to trigger immediate transcription.
- **Cancel:** Press `Escape` or the opposite hotkey while recording to abort without inserting text.

---

## 3. WhisperFlow Self-Learning Memory

Murmur doesn't guess names randomly. It continuously learns your vocabulary through three built-in mechanisms:

### Method A: Zero-Click Post-Paste Keyboard Observation
1. Dictate any sentence. Murmur pastes the text into your active document or editor.
2. If Murmur misheard a technical term or name (e.g. transcribing *"Eben Vargis"* instead of *"Abin Varghese"*), simply backspace and retype the correct word using your keyboard.
3. Murmur monitors your focused editor for 90 seconds. It automatically detects the substitution, learns the phonetic pair, and saves it to your local memory bank. The next time you say that word, it transcribes correctly.

### Method B: Live Teaching Terminal (Voice Memory Tab)
1. Open Murmur Settings (`Right-click tray icon` $\to$ `Open Studio` or navigate to the **Voice Memory** tab).
2. Click **Teach Murmur with Speech**. Speak a sentence containing your custom word.
3. Review the transcript, enter what you meant, and click **Learn Correction**.

### Method C: Explicit Term Management
Add known proper nouns, project names, handles, or acronyms directly into the **Add a Known Term** card.

---

## 4. Custom App Launchers

If you use desktop applications installed via Windows Store execution aliases or custom paths (e.g., ChatGPT Desktop, Antigravity, Figma, Cursor, VS Code):

1. Open **Settings** $\to$ **App Launchers**.
2. Click **+ Add Custom Launcher**.
3. Fill in:
   - **Name:** e.g., `ChatGPT`
   - **Target Executable:** e.g., `chatgpt-desktop.exe` or full path
   - **Voice Aliases:** `chat gpt, chatgpt, chat-gpt, chat gbt, gpt`
4. Say: *"Open ChatGPT"* in Agent Mode (`Ctrl + Alt + A`). Murmur will launch the application immediately.

---

## 5. Voice Command Reference

In Agent Mode (`Ctrl + Alt + A`), you can speak natural computer actions:

- **Launch Apps:** *"Open Antigravity"*, *"Launch Spotify"*, *"Start Notepad"*
- **Media & Volume:** *"Set volume to 50%"*, *"Mute audio"*, *"Next track"*, *"Play/pause music"*
- **Web & Search:** *"Search google for rust async streams"*, *"Open youtube dot com"*
- **Window Management:** *"Minimize window"*, *"Close window"* (prompts approval card)
- **Drafting:** *"Send email to dev@company.com subject quarterly report"*

---

## 6. Privacy & Offline Guarantees

- **No telemetry or audio transmission:** Audio is processed entirely in volatile RAM using on-device INT8 quantized models.
- **Local persistence:** Settings and memory stay on your machine in `%APPDATA%\com.murmur.desktop\`.
