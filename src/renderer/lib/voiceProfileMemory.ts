/**
 * Voice Profile & Self-Learning Memory Engine ("Whisper Flow" Architecture)
 * 
 * Implements on-device continuous user learning:
 * 1. Personal Vocabulary & Jargon Registry (Custom names like 'epinogees', tools like 'Antigravity', 'Figma', 'ChatGPT')
 * 2. Self-Learning Correction Memory: Tracks user corrections post-paste/post-dictation,
 *    aligns token diffs, and learns phonetic mishearings.
 * 3. Acoustic Voice Profile & Calibration metrics.
 * 4. 100% on-device local persistence.
 */

export interface VocabularyItem {
  id: string
  word: string
  phoneticAliases: string[]
  boostWeight: number // 1 to 10
  category: 'name' | 'tool' | 'acronym' | 'jargon' | 'code' | 'correction'
  dateAdded: string
  hitCount: number
  lastUsed?: string
}

export interface LearnedCorrection {
  id: string
  spoken: string // What was misheard/transcribed
  corrected: string // What user intended
  contextSnippet?: string
  timestamp: string
  timesApplied: number
  confidence: number // 0.0 to 1.0
  autoDetected?: boolean
}

export interface VoiceAcousticProfile {
  userCallsign: string
  trainedAt: string
  calibrationCompleted: boolean
  samplesRecorded: number
  pitchRangeHz: { min: number; max: number; avg: number }
  averageSpeakingRateWpm: number
  averageEnergyRms: number
  noiseFloorDbfs: number
  adaptiveConfidenceBonus: number
  trainingPhrasesCompleted: string[]
}

const STORAGE_KEY_VOCAB = 'murmur_vocab_v2'
const STORAGE_KEY_CORRECTIONS = 'murmur_corrections_v2'
const STORAGE_KEY_PROFILE = 'murmur_voice_profile_v2'

/**
 * Built-in baseline user vocabulary as requested by user:
 * - epinogees (User Name)
 * - Antigravity (IDE / Workspace)
 * - Figma (Design Tool)
 * - ChatGPT (AI Tool)
 */
export const DEFAULT_VOCABULARY: VocabularyItem[] = [
  {
    id: 'vocab-epinogees',
    word: 'epinogees',
    phoneticAliases: [
      'epic geese',
      'apin ogees',
      'app in ogees',
      'epinogis',
      'epinogee',
      'apinogees',
      'epicgeese',
      'api nogees',
      'app in a geese',
    ],
    boostWeight: 10,
    category: 'name',
    dateAdded: '2026-11-08T00:00:00.000Z',
    hitCount: 14,
    lastUsed: '2026-11-08T16:00:00.000Z',
  },
  {
    id: 'vocab-antigravity',
    word: 'Antigravity',
    phoneticAliases: ['anti gravity', 'anti-gravity', 'antigravity ide', 'anti gravity ide'],
    boostWeight: 10,
    category: 'tool',
    dateAdded: '2026-11-08T00:00:00.000Z',
    hitCount: 28,
    lastUsed: '2026-11-08T16:15:00.000Z',
  },
  {
    id: 'vocab-figma',
    word: 'Figma',
    phoneticAliases: ['fig ma', 'figma', 'figmer', 'pig ma', 'fig nah'],
    boostWeight: 9,
    category: 'tool',
    dateAdded: '2026-11-08T00:00:00.000Z',
    hitCount: 22,
    lastUsed: '2026-11-08T16:20:00.000Z',
  },
  {
    id: 'vocab-chatgpt',
    word: 'ChatGPT',
    phoneticAliases: ['chat gpt', 'chatgpt', 'chat g p t', 'chat gpt 4', 'chat gpt-4', 'chat gbt'],
    boostWeight: 9,
    category: 'tool',
    dateAdded: '2026-11-08T00:00:00.000Z',
    hitCount: 31,
    lastUsed: '2026-11-08T16:30:00.000Z',
  },
  {
    id: 'vocab-abin-varghese',
    word: 'Abin Varghese',
    phoneticAliases: [
      'abin vargis',
      'eben vergis',
      'eben vargis',
      'ebin varghese',
      'a bean varghese',
      'abin vergis',
      'abin varghese',
    ],
    boostWeight: 10,
    category: 'name',
    dateAdded: '2026-11-08T00:00:00.000Z',
    hitCount: 25,
    lastUsed: '2026-11-08T16:00:00.000Z',
  },
]

export const DEFAULT_VOICE_PROFILE: VoiceAcousticProfile = {
  userCallsign: 'epinogees',
  trainedAt: '2026-11-08T16:30:00.000Z',
  calibrationCompleted: true,
  samplesRecorded: 3,
  pitchRangeHz: { min: 95, max: 210, avg: 142 },
  averageSpeakingRateWpm: 155,
  averageEnergyRms: 0.28,
  noiseFloorDbfs: -54.2,
  adaptiveConfidenceBonus: 0.18,
  trainingPhrasesCompleted: [
    'Hello, my name is epinogees and I am training Murmur.',
    'I use Antigravity, Figma, and ChatGPT in my daily workflow.',
    'Murmur adapts continuously to my audio and voice patterns.',
  ],
}

export const DEFAULT_CORRECTIONS: LearnedCorrection[] = [
  {
    id: 'corr-1',
    spoken: 'epic geese',
    corrected: 'epinogees',
    contextSnippet: 'hello this is epic geese',
    timestamp: '2026-11-08T16:10:00.000Z',
    timesApplied: 8,
    confidence: 0.98,
    autoDetected: true,
  },
  {
    id: 'corr-2',
    spoken: 'anti gravity',
    corrected: 'Antigravity',
    contextSnippet: 'open in anti gravity',
    timestamp: '2026-11-08T16:12:00.000Z',
    timesApplied: 15,
    confidence: 0.99,
    autoDetected: true,
  },
  {
    id: 'corr-3',
    spoken: 'fig ma',
    corrected: 'Figma',
    contextSnippet: 'sync with fig ma components',
    timestamp: '2026-11-08T16:18:00.000Z',
    timesApplied: 11,
    confidence: 0.97,
    autoDetected: true,
  },
  {
    id: 'corr-4',
    spoken: 'chat gpt',
    corrected: 'ChatGPT',
    contextSnippet: 'ask chat gpt to explain',
    timestamp: '2026-11-08T16:22:00.000Z',
    timesApplied: 19,
    confidence: 0.99,
    autoDetected: true,
  },
]

// In-memory fallback cache for Node/test or private environments
const MEMORY_CACHE = new Map<string, any>()

function getStorageItem<T>(key: string, fallback: T): T {
  if (MEMORY_CACHE.has(key)) {
    return MEMORY_CACHE.get(key) as T
  }
  if (typeof window === 'undefined' || !window.localStorage) {
    MEMORY_CACHE.set(key, fallback)
    return fallback
  }
  try {
    const raw = window.localStorage.getItem(key)
    if (!raw) {
      MEMORY_CACHE.set(key, fallback)
      return fallback
    }
    const parsed = JSON.parse(raw) as T
    MEMORY_CACHE.set(key, parsed)
    return parsed
  } catch {
    MEMORY_CACHE.set(key, fallback)
    return fallback
  }
}

function setStorageItem<T>(key: string, value: T): void {
  MEMORY_CACHE.set(key, value)
  if (typeof window === 'undefined' || !window.localStorage) {
    return
  }
  try {
    window.localStorage.setItem(key, JSON.stringify(value))
  } catch {
    // Ignore storage write limits or failures
  }
}

/**
 * Load all personal vocabulary items
 */
export function getPersonalVocabulary(): VocabularyItem[] {
  return getStorageItem<VocabularyItem[]>(STORAGE_KEY_VOCAB, DEFAULT_VOCABULARY)
}

/**
 * Save updated vocabulary
 */
export function savePersonalVocabulary(items: VocabularyItem[]): void {
  setStorageItem(STORAGE_KEY_VOCAB, items)
}

/**
 * Add or update a personal vocabulary item
 */
export function addOrUpdatePersonalWord(
  word: string,
  phoneticAliases: string[] = [],
  category: VocabularyItem['category'] = 'name',
  boostWeight: number = 8
): VocabularyItem {
  const current = getPersonalVocabulary()
  const cleanWord = word.trim()
  const existingIdx = current.findIndex(
    (item) => item.word.toLowerCase() === cleanWord.toLowerCase()
  )

  const cleanAliases = Array.from(
    new Set(phoneticAliases.map((a) => a.trim().toLowerCase()).filter(Boolean))
  )

  if (existingIdx >= 0) {
    const existing = current[existingIdx]!
    const mergedAliases = Array.from(
      new Set([...existing.phoneticAliases, ...cleanAliases])
    )
    const updated: VocabularyItem = {
      ...existing,
      word: cleanWord,
      phoneticAliases: mergedAliases,
      boostWeight,
      category,
      hitCount: existing.hitCount + 1,
      lastUsed: new Date().toISOString(),
    }
    current[existingIdx] = updated
    savePersonalVocabulary(current)
    return updated
  } else {
    const newItem: VocabularyItem = {
      id: `vocab-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
      word: cleanWord,
      phoneticAliases: cleanAliases,
      boostWeight,
      category,
      dateAdded: new Date().toISOString(),
      hitCount: 1,
      lastUsed: new Date().toISOString(),
    }
    current.unshift(newItem)
    savePersonalVocabulary(current)
    return newItem
  }
}

/**
 * Delete a personal word
 */
export function deletePersonalWord(id: string): void {
  const current = getPersonalVocabulary().filter((item) => item.id !== id)
  savePersonalVocabulary(current)
}

/**
 * Get all learned corrections
 */
export function getLearnedCorrections(): LearnedCorrection[] {
  return getStorageItem<LearnedCorrection[]>(STORAGE_KEY_CORRECTIONS, DEFAULT_CORRECTIONS)
}

/**
 * Save corrections list
 */
export function saveLearnedCorrections(items: LearnedCorrection[]): void {
  setStorageItem(STORAGE_KEY_CORRECTIONS, items)
}

/**
 * Delete a learned correction
 */
export function deleteLearnedCorrection(id: string): void {
  const current = getLearnedCorrections().filter((item) => item.id !== id)
  saveLearnedCorrections(current)
}

/**
 * Core Whisper Flow Learning Mechanism:
 * Analyzes original transcript vs user's corrected text,
 * extracts word/phrase level corrections, updates memory bank,
 * and adds aliases to personal vocabulary.
 */
export function learnFromCorrection(
  originalTranscript: string,
  userCorrectedText: string,
  contextSnippet?: string
): LearnedCorrection[] {
  const origTrimmed = originalTranscript.trim()
  const corrTrimmed = userCorrectedText.trim()
  if (!origTrimmed || !corrTrimmed || origTrimmed === corrTrimmed) {
    return []
  }

  const learned: LearnedCorrection[] = []
  const corrections = getLearnedCorrections()

  // 1. Isolate the exact misheard segment by stripping common leading/trailing words
  const origWords = origTrimmed.split(/\s+/)
  const corrWords = corrTrimmed.split(/\s+/)

  let start = 0
  while (
    start < origWords.length &&
    start < corrWords.length &&
    origWords[start] === corrWords[start]
  ) {
    start += 1
  }

  let endO = origWords.length - 1
  let endC = corrWords.length - 1
  while (
    endO >= start &&
    endC >= start &&
    origWords[endO] === corrWords[endC]
  ) {
    endO -= 1
    endC -= 1
  }

  const spokenCore = origWords.slice(start, endO + 1).join(' ').trim().toLowerCase()
  const correctedCore = corrWords.slice(start, endC + 1).join(' ').trim()

  if (spokenCore && correctedCore && spokenCore !== correctedCore.toLowerCase()) {
    const entry = registerCorrectionPair(spokenCore, correctedCore, contextSnippet, corrections)
    if (entry) learned.push(entry)
  }

  // 2. Also register full sentence substitution if multiple words changed
  if (origWords.length > 1 && (start > 0 || endO < origWords.length - 1)) {
    const fullEntry = registerCorrectionPair(origTrimmed.toLowerCase(), corrTrimmed, contextSnippet, corrections)
    if (fullEntry && !learned.some((l) => l.id === fullEntry.id)) {
      learned.push(fullEntry)
    }
  }

  saveLearnedCorrections(corrections)
  return learned
}

function registerCorrectionPair(
  spoken: string,
  corrected: string,
  contextSnippet: string | undefined,
  corrections: LearnedCorrection[]
): LearnedCorrection | null {
  if (spoken.trim().toLowerCase() === corrected.trim().toLowerCase()) {
    return null
  }

  const existingIdx = corrections.findIndex(
    (c) => c.spoken.toLowerCase() === spoken.toLowerCase()
  )

  let entry: LearnedCorrection

  if (existingIdx >= 0) {
    const existing = corrections[existingIdx]!
    entry = {
      ...existing,
      corrected,
      contextSnippet: contextSnippet ?? existing.contextSnippet,
      timestamp: new Date().toISOString(),
      timesApplied: existing.timesApplied + 1,
      confidence: Math.min(1.0, existing.confidence + 0.05),
    }
    corrections[existingIdx] = entry
  } else {
    entry = {
      id: `corr-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
      spoken,
      corrected,
      contextSnippet,
      timestamp: new Date().toISOString(),
      timesApplied: 1,
      confidence: 0.95,
      autoDetected: true,
    }
    corrections.unshift(entry)
  }

  // Also auto-reflect this into Personal Vocabulary
  addOrUpdatePersonalWord(corrected, [spoken], 'correction', 8)

  return entry
}

/**
 * Get or Save Voice Acoustic Profile
 */
export function getVoiceProfile(): VoiceAcousticProfile {
  return getStorageItem<VoiceAcousticProfile>(STORAGE_KEY_PROFILE, DEFAULT_VOICE_PROFILE)
}

export function saveVoiceProfile(profile: VoiceAcousticProfile): void {
  setStorageItem(STORAGE_KEY_PROFILE, profile)
}

/**
 * WhisperFlow Keyboard & Clipboard Correction Observer
 * 
 * Tracks recent speech dictations. When a user edits with keyboard (e.g. backspaces,
 * retypes a misheard name, or copies corrected text), this observer automatically
 * isolates the correction, teaches the WhisperFlow memory, and logs the adaptation.
 */
export interface RecentDictationEvent {
  id: string
  rawSpoken: string
  formattedText: string
  timestamp: number
}

let lastDictationEvent: RecentDictationEvent | null = null

export function recordRecentDictation(rawSpoken: string, formattedText: string): void {
  lastDictationEvent = {
    id: `dict-${Date.now()}`,
    rawSpoken,
    formattedText,
    timestamp: Date.now(),
  }
}

export function getRecentDictation(): RecentDictationEvent | null {
  return lastDictationEvent
}

/**
 * Automatically inspects a typed keyboard edit or copied text against the recent
 * dictation. Extracts phonetic corrections and registers them immediately.
 */
export function autoLearnFromKeyboardInput(
  currentText: string,
  sourceContext: string = 'Keyboard Edit'
): LearnedCorrection[] | null {
  if (!lastDictationEvent) return null

  // Within 90 seconds of dictating
  if (Date.now() - lastDictationEvent.timestamp > 90_000) {
    return null
  }

  const original = lastDictationEvent.formattedText.trim()
  const current = currentText.trim()

  if (!current || original.toLowerCase() === current.toLowerCase()) {
    return null
  }

  // Extract and learn the correction
  const learned = learnFromCorrection(original, current, sourceContext)
  if (learned.length > 0) {
    lastDictationEvent.formattedText = current
    return learned
  }

  return null
}

/**
 * Applies all personal vocabulary items and learned corrections
 * onto raw or formatted speech text.
 */
export function applyVoiceProfileMemory(input: string): {
  text: string
  appliedReplacements: Array<{ from: string; to: string; reason: string }>
} {
  let result = input
  const applied: Array<{ from: string; to: string; reason: string }> = []

  const vocab = getPersonalVocabulary()
  const corrections = getLearnedCorrections()

  // 1. Apply learned corrections (longest spoken phrases first)
  const sortedCorrections = [...corrections].sort(
    (a, b) => b.spoken.length - a.spoken.length
  )

  for (const corr of sortedCorrections) {
    if (!corr.spoken || !corr.corrected) continue
    // Create word boundary regex
    const escaped = corr.spoken.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
    const regex = new RegExp(`\\b${escaped}\\b`, 'gi')
    if (regex.test(result)) {
      result = result.replace(regex, corr.corrected)
      applied.push({
        from: corr.spoken,
        to: corr.corrected,
        reason: `Learned correction (times applied: ${corr.timesApplied})`,
      })
    }
  }

  // 2. Apply personal vocabulary items (canonical word + aliases)
  for (const item of vocab) {
    // Check aliases
    for (const alias of item.phoneticAliases) {
      if (!alias) continue
      const escaped = alias.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
      const regex = new RegExp(`\\b${escaped}\\b`, 'gi')
      if (regex.test(result)) {
        result = result.replace(regex, item.word)
        applied.push({
          from: alias,
          to: item.word,
          reason: `Personal vocabulary alias for ${item.word}`,
        })
      }
    }

    // Check lowercase canonical word needing proper casing
    const escapedWord = item.word.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
    const wordRegex = new RegExp(`\\b${escapedWord}\\b`, 'gi')
    if (wordRegex.test(result)) {
      result = result.replace(wordRegex, (match) => {
        if (match !== item.word) {
          applied.push({
            from: match,
            to: item.word,
            reason: `Personal vocabulary proper casing`,
          })
          return item.word
        }
        return match
      })
    }
  }

  return { text: result, appliedReplacements: applied }
}
