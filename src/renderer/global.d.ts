interface MurmurCleanup {
  removeFillers: boolean
  spokenPunctuation: boolean
  autoCapitalize: boolean
  smartFormat: boolean
}

interface MurmurSettings {
  dictationHotkey: string
  agentHotkey: string
  cleanup: MurmurCleanup
  warmOnStartup: boolean
  autoStopSeconds: number
}

interface MurmurPillState {
  phase: 'idle' | 'preparing' | 'recording' | 'processing' | 'result' | 'notice'
  mode: 'dictation' | 'agent'
  message: string
}

interface CustomAppItem {
  id: string
  name: string
  aliases: string[]
  target: string
}

interface VoiceMemoryEntry {
  id: string
  kind: 'person' | 'vocabulary' | 'phrase'
  canonical: string
  aliases: string[]
  contexts: string[]
  frequency: number
  confidence: number
  explicit?: boolean
  promoted?: boolean
  createdAt: string
  updatedAt: string
}

interface LearnedVoiceCorrection {
  id: string
  original: string
  corrected: string
  context: string
  frequency: number
  confidence: number
  successes: number
  failures: number
  promoted: boolean
  createdAt: string
  updatedAt: string
}

interface VoiceTranscriptRecord {
  id: string
  raw: string
  personalized: string
  corrected?: string | null
  mode: string
  timestamp: string
}

interface VoiceLearningExample {
  id: string
  audioPath: string
  raw: string
  inserted: string
  observed?: string | null
  corrected?: string | null
  correctionId?: string | null
  status: string
  successes: number
  failures: number
  createdAt: string
  updatedAt: string
}

interface MurmurVoiceMemory {
  schemaVersion: number
  entries: VoiceMemoryEntry[]
  corrections: LearnedVoiceCorrection[]
  recentTranscripts: VoiceTranscriptRecord[]
  learningExamples: VoiceLearningExample[]
}

interface VoiceTrainingEvent {
  status: 'recording' | 'processing' | 'completed' | 'error'
  message?: string | null
  raw?: string | null
  personalized?: string | null
}

interface MurmurApi {
  /** Pill: what the app is doing right now. */
  onPillState(cb: (state: MurmurPillState) => void): () => void
  /** Pill: a disruptive command is waiting for the user's yes or no. */
  onConfirm(cb: (request: { summary: string }) => void): () => void
  /** Pill: answer the pending confirmation. */
  confirm(approved: boolean): void

  /** Settings. */
  getSettings(): Promise<MurmurSettings | null>
  saveSettings(settings: MurmurSettings): Promise<{ ok: boolean; failed: string[]; error: string }>
  getModelStatus(): Promise<{ ready: boolean; totalBytes: number; downloading: boolean } | null>
  downloadModel(): Promise<{ ok: boolean; error?: string | null }>
  onModelProgress(cb: (p: { receivedBytes: number; totalBytes: number }) => void): () => void

  /** Custom App Launchers & Command Aliases. */
  getCustomApps(): Promise<CustomAppItem[]>
  saveCustomApps(apps: CustomAppItem[]): Promise<void>
  launchCustomApp(target: string): Promise<void>

  /** Local WhisperFlow memory. */
  getVoiceMemory(): Promise<MurmurVoiceMemory | null>
  learnVoiceCorrection(original: string, corrected: string, context: string): Promise<unknown>
  addVoiceMemoryEntry(input: {
    canonical: string
    aliases: string[]
    kind: VoiceMemoryEntry['kind']
    id?: string
  }): Promise<void>
  deleteVoiceMemoryEntry(id: string): Promise<boolean>
  deleteVoiceCorrection(id: string): Promise<boolean>
  clearVoiceMemory(): Promise<void>
  onVoiceTraining(cb: (event: VoiceTrainingEvent) => void): () => void
  onLearningUpdate(cb: (event: unknown) => void): () => void
  startVoiceTraining(): Promise<void>
  stopVoiceTraining(): Promise<void>
}

interface Window {
  murmur: MurmurApi
}
