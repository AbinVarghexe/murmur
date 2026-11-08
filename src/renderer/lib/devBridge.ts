// Development-only stand-in for the Tauri bridge, so the interface can be previewed in a plain
// browser. It is imported behind import.meta.env.DEV and is removed from production builds.
//
// Preview a pill state:  /pill.html?state=recording|preparing|processing|result|notice
//                        add &mode=agent for voice-command mode, &confirm=1 for the approval card

type Listener<T> = (value: T) => void

const TOTAL_BYTES = 660 * 1024 * 1024

function emitter<T>(): { on: (cb: Listener<T>) => () => void; emit: (value: T) => void } {
  const listeners = new Set<Listener<T>>()
  return {
    on: (cb) => {
      listeners.add(cb)
      return () => listeners.delete(cb)
    },
    emit: (value) => listeners.forEach((cb) => cb(value)),
  }
}

const MESSAGES: Record<MurmurPillState['phase'], string> = {
  idle: '',
  preparing: 'Getting ready…',
  recording: 'Listening…',
  processing: 'Transcribing…',
  result: 'Meeting moved to Thursday at 3pm',
  notice: "Didn't catch that.",
}

export function installDevBridge(): void {
  const progress = emitter<{ receivedBytes: number; totalBytes: number }>()
  const voiceTraining = emitter<VoiceTrainingEvent>()
  const learningUpdate = emitter<unknown>()
  const params = new URLSearchParams(window.location.search)
  let modelReady = params.get('model') === 'ready'
  let settings: MurmurSettings = {
    dictationHotkey: 'Control+Alt+Space',
    agentHotkey: 'Control+Alt+A',
    cleanup: { removeFillers: true, spokenPunctuation: false, autoCapitalize: true, smartFormat: true },
    warmOnStartup: true,
    autoStopSeconds: 3,
  }
  let voiceMemory: MurmurVoiceMemory = {
    schemaVersion: 2,
    entries: [
      {
        id: 'memory-epinogees',
        kind: 'person',
        canonical: 'epinogees',
        aliases: [
          'epic geese',
          'apin ogees',
          'app in ogees',
          'epinogis',
          'epinogee',
          'apinogees',
          'epicgeese',
          'api nogees',
        ],
        contexts: ['personal', 'identity'],
        frequency: 14,
        confidence: 0.98,
        createdAt: '0',
        updatedAt: '0',
      },
      {
        id: 'memory-abin-varghese',
        kind: 'person',
        canonical: 'Abin Varghese',
        aliases: ['abin vargis', 'eben vergis', 'eben vargis', 'ebin varghese', 'a bean varghese'],
        contexts: ['name', 'author'],
        frequency: 18,
        confidence: 0.98,
        createdAt: '0',
        updatedAt: '0',
      },
      {
        id: 'memory-antigravity',
        kind: 'vocabulary',
        canonical: 'Antigravity',
        aliases: ['anti gravity', 'anti-gravity', 'antigravity ide', 'anti gravity ide'],
        contexts: ['editor', 'ide'],
        frequency: 24,
        confidence: 0.98,
        createdAt: '0',
        updatedAt: '0',
      },
      {
        id: 'memory-chatgpt',
        kind: 'vocabulary',
        canonical: 'ChatGPT',
        aliases: ['chat gpt', 'chat-gpt', 'chatgpt', 'chat gbt', 'gpt'],
        contexts: ['ai', 'tool'],
        frequency: 28,
        confidence: 0.98,
        createdAt: '0',
        updatedAt: '0',
      },
      {
        id: 'memory-figma',
        kind: 'vocabulary',
        canonical: 'Figma',
        aliases: ['fig ma', 'figma', 'figmer', 'pig ma'],
        contexts: ['design', 'ui'],
        frequency: 20,
        confidence: 0.98,
        createdAt: '0',
        updatedAt: '0',
      },
    ],
    corrections: [],
    recentTranscripts: [],
    learningExamples: [],
  }

  const phase = (params.get('state') ?? 'idle') as MurmurPillState['phase']
  const pinned: MurmurPillState = {
    phase,
    mode: params.get('mode') === 'agent' ? 'agent' : 'dictation',
    message: MESSAGES[phase] ?? '',
  }

  window.murmur = {
    // Replay the pinned state to each new subscriber so previews are deterministic.
    onPillState: (cb) => {
      cb(pinned)
      return () => undefined
    },
    onConfirm: (cb) => {
      if (params.get('confirm')) cb({ summary: 'Close the active window' })
      return () => undefined
    },
    confirm: () => undefined,
    getSettings: async () => settings,
    saveSettings: async (next) => {
      settings = next
      return { ok: true, failed: [], error: '' }
    },
    getModelStatus: async () => ({ ready: modelReady, totalBytes: TOTAL_BYTES, downloading: false }),
    downloadModel: async () => {
      for (let step = 1; step <= 20; step += 1) {
        await new Promise((resolve) => window.setTimeout(resolve, 150))
        progress.emit({ receivedBytes: (TOTAL_BYTES / 20) * step, totalBytes: TOTAL_BYTES })
      }
      modelReady = true
      return { ok: true }
    },
    onModelProgress: progress.on,
    getCustomApps: async () => [
      {
        id: 'chatgpt',
        name: 'ChatGPT',
        aliases: ['chat gpt', 'chatgpt', 'chat-gpt', 'chat gbt', 'gpt'],
        target: 'chatgpt-desktop.exe',
      },
      {
        id: 'antigravity',
        name: 'Antigravity',
        aliases: ['anti gravity', 'antigravity', 'antigravity ide'],
        target: 'antigravity.exe',
      },
      {
        id: 'figma',
        name: 'Figma',
        aliases: ['fig ma', 'figma'],
        target: 'figma.exe',
      },
    ],
    saveCustomApps: async () => undefined,
    launchCustomApp: async () => undefined,
    getVoiceMemory: async () => voiceMemory,
    learnVoiceCorrection: async (original, corrected, context) => {
      const now = String(Date.now())
      const correction: LearnedVoiceCorrection = {
        id: `correction-${now}`,
        original,
        corrected,
        context,
        frequency: 1,
        confidence: 0.62,
        successes: 0,
        failures: 0,
        promoted: false,
        createdAt: now,
        updatedAt: now,
      }
      voiceMemory = {
        ...voiceMemory,
        corrections: [correction, ...voiceMemory.corrections],
      }
      return { learned: [correction], entryIds: [] }
    },
    addVoiceMemoryEntry: async ({ canonical, aliases, kind, id }) => {
      const now = String(Date.now())
      const entry: VoiceMemoryEntry = {
        id: id ?? `memory-${now}`,
        kind,
        canonical,
        aliases,
        contexts: [],
        frequency: 1,
        confidence: 0.8,
        createdAt: now,
        updatedAt: now,
      }
      voiceMemory = {
        ...voiceMemory,
        entries: [...voiceMemory.entries.filter((item) => item.id !== entry.id), entry],
      }
    },
    deleteVoiceMemoryEntry: async (id) => {
      const before = voiceMemory.entries.length
      voiceMemory = { ...voiceMemory, entries: voiceMemory.entries.filter((item) => item.id !== id) }
      return before !== voiceMemory.entries.length
    },
    deleteVoiceCorrection: async (id) => {
      const before = voiceMemory.corrections.length
      voiceMemory = { ...voiceMemory, corrections: voiceMemory.corrections.filter((item) => item.id !== id) }
      return before !== voiceMemory.corrections.length
    },
    clearVoiceMemory: async () => {
      voiceMemory = { schemaVersion: 2, entries: [], corrections: [], recentTranscripts: [], learningExamples: [] }
    },
    onVoiceTraining: voiceTraining.on,
    onLearningUpdate: learningUpdate.on,
    startVoiceTraining: async () => {
      voiceTraining.emit({ status: 'recording', message: 'Speak a sentence containing the term you want to teach.' })
    },
    stopVoiceTraining: async () => {
      voiceTraining.emit({ status: 'processing', message: 'Transcribing your teaching example locally…' })
      await new Promise((resolve) => window.setTimeout(resolve, 450))
      voiceTraining.emit({
        status: 'completed',
        message: 'Transcript ready. Review it and enter what you meant.',
        raw: 'I sent it to Eben Vargis',
        personalized: 'I sent it to Eben Vargis',
      })
    },
  }
}
