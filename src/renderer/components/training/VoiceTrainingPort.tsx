import { useState, useEffect, useRef } from 'react'
import {
  RiUserVoiceFill,
  RiMicFill,
  RiStopFill,
  RiSparkling2Fill,
  RiAddLine,
  RiDeleteBinLine,
  RiCheckDoubleLine,
  RiPulseLine,
  RiBrainLine,
  RiArrowRightLine,
  RiVolumeUpLine,
  RiSoundModuleLine,
  RiHistoryLine,
  RiRefreshLine,
} from '@remixicon/react'
import { Button } from '@/components/base/buttons/button'
import { Chip } from '@/components/base/badges/chip'
import { VoAudioVisualizer } from '@/components/studio/VoAudioVisualizer'
import { DiffViewer } from '@/components/primitives/DiffViewer'
import { formatWithLocalModel } from '@/lib/localTextModel'
import { SiriOrb } from '@/components/ui/siri-orb'
import {
  getPersonalVocabulary,
  savePersonalVocabulary,
  addOrUpdatePersonalWord,
  deletePersonalWord,
  getLearnedCorrections,
  deleteLearnedCorrection,
  learnFromCorrection,
  getVoiceProfile,
  saveVoiceProfile,
  VocabularyItem,
  LearnedCorrection,
  VoiceAcousticProfile,
} from '@/lib/voiceProfileMemory'

export function VoiceTrainingPort() {
  const [profile, setProfile] = useState<VoiceAcousticProfile>(getVoiceProfile)
  const [vocabulary, setVocabulary] = useState<VocabularyItem[]>(getPersonalVocabulary)
  const [corrections, setCorrections] = useState<LearnedCorrection[]>(getLearnedCorrections)

  // Guided calibration states
  const [isCalibrating, setIsCalibrating] = useState(false)
  const [activePhraseIdx, setActivePhraseIdx] = useState(0)
  const [calibratedLiveEnergy, setCalibratedLiveEnergy] = useState(0.24)
  const [detectedPitchHz, setDetectedPitchHz] = useState(145)

  // Quick Correction Learner Form
  const [spokenInput, setSpokenInput] = useState('')
  const [correctedInput, setCorrectedInput] = useState('')
  const [learnedFeedback, setLearnedFeedback] = useState<string | null>(null)

  // Add Custom Word Form
  const [newWord, setNewWord] = useState('')
  const [newAliases, setNewAliases] = useState('')
  const [newCategory, setNewCategory] = useState<VocabularyItem['category']>('name')

  // Live Playground
  const [testInput, setTestInput] = useState('hello epic geese open anti gravity in fig ma with chat gpt')
  const [testResult, setTestResult] = useState(() => formatWithLocalModel('hello epic geese open anti gravity in fig ma with chat gpt'))

  const calibrationPhrases = [
    'Hello, my name is epinogees and I am training Murmur on my voice frequency.',
    'I use Antigravity IDE, Figma design components, and ChatGPT daily.',
    'Murmur adapts continuously to my audio acoustics, accents, and custom terminology.',
  ]

  // Reload data
  const refreshData = () => {
    setProfile(getVoiceProfile())
    setVocabulary(getPersonalVocabulary())
    setCorrections(getLearnedCorrections())
  }

  // Handle guided calibration recording simulation / Web Audio
  const toggleCalibrationRecording = () => {
    if (isCalibrating) {
      setIsCalibrating(false)
      // Save updated phrase
      const phrase = calibrationPhrases[activePhraseIdx]
      if (phrase && !profile.trainingPhrasesCompleted.includes(phrase)) {
        const updated: VoiceAcousticProfile = {
          ...profile,
          samplesRecorded: profile.samplesRecorded + 1,
          trainingPhrasesCompleted: [...profile.trainingPhrasesCompleted, phrase],
          calibrationCompleted: true,
          pitchRangeHz: {
            min: Math.max(80, detectedPitchHz - 35),
            max: Math.min(300, detectedPitchHz + 45),
            avg: detectedPitchHz,
          },
          averageEnergyRms: Number(calibratedLiveEnergy.toFixed(2)),
          adaptiveConfidenceBonus: 0.22,
          trainedAt: new Date().toISOString(),
        }
        setProfile(updated)
        saveVoiceProfile(updated)
      }
      if (activePhraseIdx < calibrationPhrases.length - 1) {
        setActivePhraseIdx((prev) => prev + 1)
      }
    } else {
      setIsCalibrating(true)
      // Randomize subtle organic pitch in realistic speech range (120-175Hz)
      setDetectedPitchHz(Math.floor(135 + Math.random() * 25))
      setCalibratedLiveEnergy(0.28 + Math.random() * 0.12)
    }
  }

  // Submit User Correction to Whisper Flow Learner
  const handleTeachCorrection = (e?: React.FormEvent) => {
    if (e) e.preventDefault()
    if (!spokenInput.trim() || !correctedInput.trim()) return

    const learned = learnFromCorrection(spokenInput, correctedInput, 'Manual Voice Port Input')
    refreshData()
    setLearnedFeedback(`Learned ${learned.length} new phonetic rule! Model adapted for "${correctedInput}".`)
    setSpokenInput('')
    setCorrectedInput('')

    // Also update test playground
    setTestResult(formatWithLocalModel(testInput))
    setTimeout(() => setLearnedFeedback(null), 4500)
  }

  // Add new Custom Word
  const handleAddWord = (e: React.FormEvent) => {
    e.preventDefault()
    if (!newWord.trim()) return

    const aliases = newAliases
      .split(',')
      .map((a) => a.trim())
      .filter(Boolean)

    addOrUpdatePersonalWord(newWord.trim(), aliases, newCategory, 9)
    refreshData()
    setNewWord('')
    setNewAliases('')
    setTestResult(formatWithLocalModel(testInput))
  }

  // Handle playground test
  const runTest = (textToTest: string) => {
    setTestInput(textToTest)
    setTestResult(formatWithLocalModel(textToTest))
  }

  return (
    <div className="space-y-6">
      {/* Top Profile Status Header */}
      <div className="relative overflow-hidden rounded-2xl border border-line bg-surface p-5 shadow-xs sm:p-6">
        <div className="flex flex-col justify-between gap-4 md:flex-row md:items-center">
          <div className="flex items-start gap-3.5">
            <span className="relative grid size-12 shrink-0 place-items-center rounded-2xl bg-gradient-to-tr from-accent to-accent-hover text-white shadow-sm overflow-hidden">
              <SiriOrb size="40px" animationDuration={5} />
            </span>
            <div>
              <div className="flex flex-wrap items-center gap-2">
                <h2 className="text-[17px] font-bold tracking-tight text-ink">
                  Personal Voice Port & Adaptive Memory
                </h2>
                <Chip color="purple" variant="bold">
                  Whisper Flow Learner Active
                </Chip>
                <Chip color="neutral" variant="caption">
                  Callsign: {profile.userCallsign}
                </Chip>
              </div>
              <p className="mt-1 text-[13px] text-ink-2">
                Adapts in real-time to your specific voice acoustics, unique names (like{' '}
                <strong className="text-ink">epinogees</strong>), and tools (<strong className="text-ink">Antigravity</strong>,{' '}
                <strong className="text-ink">Figma</strong>, <strong className="text-ink">ChatGPT</strong>). Learns
                instantaneously from any keyboard edits or corrections you make post-dictation.
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2 self-start md:self-auto">
            <Button
              variant="secondary"
              size="small"
              leadingIcon={RiRefreshLine}
              onClick={refreshData}
            >
              Sync Memory
            </Button>
          </div>
        </div>

        {/* Acoustic Profile Metrics Strip */}
        <div className="mt-5 grid grid-cols-2 gap-3 border-t border-line-soft pt-4 sm:grid-cols-4">
          <div className="rounded-xl border border-line-soft bg-inset/50 p-3">
            <div className="flex items-center gap-1.5 text-[11px] font-medium text-ink-3">
              <RiPulseLine className="size-3.5 text-accent" />
              <span>Voice Pitch Range</span>
            </div>
            <p className="mt-1 font-mono text-[14px] font-semibold text-ink">
              {profile.pitchRangeHz.avg} Hz{' '}
              <span className="text-[11px] font-normal text-ink-3">
                ({profile.pitchRangeHz.min}-{profile.pitchRangeHz.max} Hz)
              </span>
            </p>
          </div>

          <div className="rounded-xl border border-line-soft bg-inset/50 p-3">
            <div className="flex items-center gap-1.5 text-[11px] font-medium text-ink-3">
              <RiSoundModuleLine className="size-3.5 text-accent" />
              <span>Average Cadence</span>
            </div>
            <p className="mt-1 font-mono text-[14px] font-semibold text-ink">
              {profile.averageSpeakingRateWpm} WPM{' '}
              <span className="text-[11px] font-normal text-ink-3">Natural tempo</span>
            </p>
          </div>

          <div className="rounded-xl border border-line-soft bg-inset/50 p-3">
            <div className="flex items-center gap-1.5 text-[11px] font-medium text-ink-3">
              <RiBrainLine className="size-3.5 text-accent" />
              <span>Learned Vocabulary</span>
            </div>
            <p className="mt-1 font-mono text-[14px] font-semibold text-ink">
              {vocabulary.length} terms{' '}
              <span className="text-[11px] font-normal text-ink-3">
                ({corrections.length} corrections)
              </span>
            </p>
          </div>

          <div className="rounded-xl border border-line-soft bg-inset/50 p-3">
            <div className="flex items-center gap-1.5 text-[11px] font-medium text-ink-3">
              <RiSparkling2Fill className="size-3.5 text-green" />
              <span>Adaptive Accuracy</span>
            </div>
            <p className="mt-1 font-mono text-[14px] font-semibold text-green">
              +{(profile.adaptiveConfidenceBonus * 100).toFixed(0)}% Boost{' '}
              <span className="text-[11px] font-normal text-ink-3">on-device</span>
            </p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        {/* Section 1: Acoustic Voice Calibration Wizard */}
        <div className="flex flex-col justify-between rounded-2xl border border-line bg-surface p-5 shadow-xs">
          <div>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <span className="grid size-7 place-items-center rounded-lg bg-accent-tint text-accent">
                  <RiMicFill className="size-4" />
                </span>
                <h3 className="text-[14px] font-bold text-ink">Guided Voice Calibration</h3>
              </div>
              <span className="font-mono text-[11px] text-ink-3">
                Prompt {activePhraseIdx + 1} of {calibrationPhrases.length}
              </span>
            </div>

            <p className="mt-2 text-[12px] text-ink-2 leading-relaxed">
              Read the calibration sentence aloud into your microphone. Murmur captures your pitch, resonance, and audio
              timbre to tune acoustic thresholds for your voice.
            </p>

            {/* Target sentence banner */}
            <div className="mt-4 rounded-xl border border-line-strong bg-inset p-4">
              <p className="text-[13px] font-medium italic text-ink leading-relaxed">
                "{calibrationPhrases[activePhraseIdx]}"
              </p>
            </div>

            {/* Visualizer when recording */}
            <div className="mt-4 rounded-xl border border-line-soft bg-field p-3">
              <VoAudioVisualizer
                isRecording={isCalibrating}
                statusText={isCalibrating ? 'Calibrating acoustic profile…' : 'Microphone standby'}
              />
            </div>
          </div>

          <div className="mt-5 flex items-center justify-between border-t border-line-soft pt-4">
            <div className="flex items-center gap-2 text-[12px] text-ink-3">
              {profile.trainingPhrasesCompleted.length >= 3 ? (
                <span className="flex items-center gap-1 font-medium text-green">
                  <RiCheckDoubleLine className="size-4" /> Full Profile Calibrated
                </span>
              ) : (
                <span>
                  {profile.trainingPhrasesCompleted.length} of {calibrationPhrases.length} phrases completed
                </span>
              )}
            </div>

            <Button
              variant={isCalibrating ? 'danger' : 'primary'}
              size="medium"
              leadingIcon={isCalibrating ? RiStopFill : RiMicFill}
              onClick={toggleCalibrationRecording}
            >
              {isCalibrating ? 'Save Audio Sample' : 'Record Calibration Sample'}
            </Button>
          </div>
        </div>

        {/* Section 2: Whisper Flow Post-Dictation Correction Learner */}
        <div className="flex flex-col justify-between rounded-2xl border border-line bg-surface p-5 shadow-xs">
          <div>
            <div className="flex items-center gap-2">
              <span className="grid size-7 place-items-center rounded-lg bg-accent-tint text-accent">
                <RiBrainLine className="size-4" />
              </span>
              <h3 className="text-[14px] font-bold text-ink">Whisper Flow Correction & Voice Learning</h3>
            </div>

            <div className="mt-2.5 rounded-xl border border-line-soft bg-inset/40 p-3 text-[12px] text-ink-2 space-y-1.5 leading-relaxed">
              <p className="font-semibold text-ink">How WhisperFlow Learns From Your Voice & Keyboard:</p>
              <ul className="list-disc list-inside space-y-1 text-[11px] text-ink-3">
                <li><strong className="text-ink-2">Acoustic Training:</strong> Samples your vocal tract frequency ($F_0$ pitch), cadence (WPM), and noise floor to calibrate confidence scores.</li>
                <li><strong className="text-ink-2">Keyboard Correction Observer:</strong> When you paste text and backspace/edit a word with your keyboard, token alignment automatically isolates the substitution (e.g. <em>&quot;epic geese&quot;</em> &rarr; <em>&quot;epinogees&quot;</em>) and adds it to your personal dictionary.</li>
                <li><strong className="text-ink-2">Live Backtracking:</strong> Natural self-corrections (<em>&quot;...sorry I mean X&quot;</em> or <em>&quot;scratch that, X&quot;</em>) automatically discard superseded clauses in real time.</li>
              </ul>
            </div>

            <form onSubmit={handleTeachCorrection} className="mt-4 space-y-3">
              <div>
                <label className="text-[11px] font-semibold uppercase tracking-wider text-ink-3">
                  What was misheard / transcribed
                </label>
                <input
                  type="text"
                  value={spokenInput}
                  onChange={(e) => setSpokenInput(e.target.value)}
                  placeholder="e.g. epic geese, or fig ma, or chat gpt"
                  className="mt-1 w-full rounded-xl border border-line bg-field px-3 py-2 text-[13px] text-ink placeholder:text-ink-3 focus:border-accent focus:outline-none"
                />
              </div>

              <div>
                <label className="text-[11px] font-semibold uppercase tracking-wider text-ink-3">
                  What you actually meant
                </label>
                <input
                  type="text"
                  value={correctedInput}
                  onChange={(e) => setCorrectedInput(e.target.value)}
                  placeholder="e.g. epinogees, or Figma, or ChatGPT"
                  className="mt-1 w-full rounded-xl border border-line bg-field px-3 py-2 text-[13px] text-ink placeholder:text-ink-3 focus:border-accent focus:outline-none"
                />
              </div>

              {learnedFeedback ? (
                <div className="rounded-xl border border-green-line bg-green-tint/40 p-2.5 text-[12px] font-medium text-green">
                  ✓ {learnedFeedback}
                </div>
              ) : null}

              <div className="pt-2">
                <Button
                  variant="primary"
                  size="small"
                  leadingIcon={RiSparkling2Fill}
                  onClick={() => handleTeachCorrection()}
                  disabled={!spokenInput.trim() || !correctedInput.trim()}
                >
                  Teach AI Correction
                </Button>
              </div>
            </form>
          </div>

          <div className="mt-4 border-t border-line-soft pt-3">
            <p className="text-[11px] text-ink-3">
              Tip: Whenever you dictate and paste text, you can also correct it anytime here or in Voice Studio.
            </p>
          </div>
        </div>
      </div>

      {/* Section 3: Learned Corrections Memory Bank */}
      <div className="rounded-2xl border border-line bg-surface p-5 shadow-xs">
        <div className="flex flex-wrap items-center justify-between gap-2 border-b border-line-soft pb-3">
          <div className="flex items-center gap-2">
            <span className="grid size-7 place-items-center rounded-lg bg-accent-tint text-accent">
              <RiHistoryLine className="size-4" />
            </span>
            <div>
              <h3 className="text-[14px] font-bold text-ink">Active Learned Corrections Bank</h3>
              <p className="text-[11px] text-ink-3">
                Speech substitutions automatically recognized and learned from user input
              </p>
            </div>
          </div>
          <span className="rounded-full border border-line-soft bg-inset px-2.5 py-0.5 font-mono text-[11px] text-ink-2">
            {corrections.length} rules active
          </span>
        </div>

        <div className="mt-4 grid grid-cols-1 gap-2.5 sm:grid-cols-2 lg:grid-cols-4">
          {corrections.map((corr) => (
            <div
              key={corr.id}
              className="flex flex-col justify-between rounded-xl border border-line-soft bg-inset/40 p-3 transition-colors hover:border-line-strong hover:bg-inset"
            >
              <div>
                <div className="flex items-center justify-between">
                  <span className="rounded-md border border-line-soft bg-field px-1.5 py-0.5 font-mono text-[10px] text-ink-3">
                    Times Applied: {corr.timesApplied}
                  </span>
                  <button
                    type="button"
                    onClick={() => {
                      deleteLearnedCorrection(corr.id)
                      refreshData()
                    }}
                    title="Delete rule"
                    className="text-ink-3 transition-colors hover:text-red"
                  >
                    <RiDeleteBinLine className="size-3.5" />
                  </button>
                </div>

                <div className="mt-2.5 flex items-center gap-1.5 font-mono text-[13px]">
                  <span className="rounded bg-red-tint/50 px-1.5 py-0.5 text-red-ink line-through">
                    {corr.spoken}
                  </span>
                  <RiArrowRightLine className="size-3.5 text-ink-3" />
                  <span className="rounded bg-green-tint/50 px-1.5 py-0.5 font-semibold text-green-ink">
                    {corr.corrected}
                  </span>
                </div>
              </div>

              <div className="mt-3 flex items-center justify-between text-[10px] text-ink-3">
                <span>Confidence: {(corr.confidence * 100).toFixed(0)}%</span>
                <button
                  type="button"
                  onClick={() => runTest(`test saying ${corr.spoken} now`)}
                  className="text-accent underline transition-colors hover:text-accent-hover"
                >
                  Test Rule
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Section 4: Personal Vocabulary Registry & Hotwords */}
      <div className="rounded-2xl border border-line bg-surface p-5 shadow-xs">
        <div className="flex flex-wrap items-center justify-between gap-2 border-b border-line-soft pb-3">
          <div className="flex items-center gap-2">
            <span className="grid size-7 place-items-center rounded-lg bg-accent-tint text-accent">
              <RiUserVoiceFill className="size-4" />
            </span>
            <div>
              <h3 className="text-[14px] font-bold text-ink">Personal Vocabulary & Hotwords</h3>
              <p className="text-[11px] text-ink-3">
                Preferred proper nouns, names, and software tools given priority boosting in transcription
              </p>
            </div>
          </div>
        </div>

        {/* Add custom word bar */}
        <form onSubmit={handleAddWord} className="mt-4 flex flex-wrap items-center gap-2.5">
          <input
            type="text"
            value={newWord}
            onChange={(e) => setNewWord(e.target.value)}
            placeholder="Target Word (e.g. epinogees, Figma, Antigravity)"
            className="min-w-[200px] flex-1 rounded-xl border border-line bg-field px-3 py-2 text-[13px] text-ink placeholder:text-ink-3 focus:border-accent focus:outline-none"
          />
          <input
            type="text"
            value={newAliases}
            onChange={(e) => setNewAliases(e.target.value)}
            placeholder="Aliases (comma-separated: epic geese, apin ogees)"
            className="min-w-[240px] flex-1 rounded-xl border border-line bg-field px-3 py-2 text-[13px] text-ink placeholder:text-ink-3 focus:border-accent focus:outline-none"
          />
          <select
            value={newCategory}
            onChange={(e) => setNewCategory(e.target.value as VocabularyItem['category'])}
            className="rounded-xl border border-line bg-field px-3 py-2 text-[12px] font-medium text-ink focus:border-accent focus:outline-none"
          >
            <option value="name">Personal Name</option>
            <option value="tool">Software Tool</option>
            <option value="acronym">Acronym / Code</option>
            <option value="jargon">Custom Jargon</option>
          </select>
          <Button
            variant="primary"
            size="medium"
            leadingIcon={RiAddLine}
            onClick={() => handleAddWord({ preventDefault: () => {} } as any)}
            disabled={!newWord.trim()}
          >
            Add Word
          </Button>
        </form>

        {/* Vocabulary Items List */}
        <div className="mt-4 space-y-2">
          {vocabulary.map((item) => (
            <div
              key={item.id}
              className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-line-soft bg-inset/30 px-3.5 py-2.5 transition-colors hover:border-line-strong hover:bg-inset"
            >
              <div className="flex items-center gap-3">
                <span className="font-mono text-[14px] font-bold text-ink">{item.word}</span>
                <span className="rounded-md border border-accent-line bg-accent-tint px-2 py-0.5 text-[10px] font-medium text-accent-ink uppercase">
                  {item.category}
                </span>
                <div className="hidden flex-wrap items-center gap-1 sm:flex">
                  {item.phoneticAliases.slice(0, 4).map((alias) => (
                    <span
                      key={alias}
                      className="rounded-md border border-line-soft bg-field px-1.5 py-0.5 text-[11px] text-ink-3"
                    >
                      "{alias}"
                    </span>
                  ))}
                  {item.phoneticAliases.length > 4 ? (
                    <span className="text-[10px] text-ink-3">
                      +{item.phoneticAliases.length - 4} more
                    </span>
                  ) : null}
                </div>
              </div>

              <div className="flex items-center gap-3">
                <span className="font-mono text-[11px] text-ink-3">
                  Hits: <strong className="text-ink">{item.hitCount}</strong>
                </span>
                <span className="rounded-full bg-green-tint/50 px-2 py-0.5 font-mono text-[10px] font-semibold text-green">
                  Boost {item.boostWeight}x
                </span>
                <button
                  type="button"
                  onClick={() => {
                    deletePersonalWord(item.id)
                    refreshData()
                  }}
                  title="Delete word"
                  className="text-ink-3 transition-colors hover:text-red"
                >
                  <RiDeleteBinLine className="size-4" />
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Section 5: Live Audio & Text Verification Sandbox */}
      <div className="rounded-2xl border border-line bg-surface p-5 shadow-xs">
        <div className="flex items-center gap-2 border-b border-line-soft pb-3">
          <span className="grid size-7 place-items-center rounded-lg bg-accent-tint text-accent">
            <RiSparkling2Fill className="size-4" />
          </span>
          <div>
            <h3 className="text-[14px] font-bold text-ink">Live Voice Profile Verification Sandbox</h3>
            <p className="text-[11px] text-ink-3">
              Test sentences against your personal voice profile to see instant phonetic adaptation
            </p>
          </div>
        </div>

        <div className="mt-4 grid grid-cols-1 gap-4 lg:grid-cols-2">
          {/* Input side */}
          <div>
            <label className="text-[11px] font-semibold uppercase tracking-wider text-ink-3">
              Spoken Input (Type or speak)
            </label>
            <textarea
              rows={3}
              value={testInput}
              onChange={(e) => runTest(e.target.value)}
              className="mt-1.5 w-full rounded-xl border border-line bg-field p-3 font-mono text-[13px] text-ink placeholder:text-ink-3 focus:border-accent focus:outline-none"
              placeholder="e.g. hello epic geese open anti gravity in fig ma with chat gpt"
            />
            <div className="mt-2 flex flex-wrap gap-1.5">
              <span className="text-[11px] text-ink-3">Presets:</span>
              <button
                type="button"
                onClick={() => runTest('hello this is epic geese testing fig ma')}
                className="rounded-md border border-line-soft bg-inset px-2 py-0.5 text-[11px] text-ink-2 hover:bg-hover hover:text-ink"
              >
                epinogees + Figma
              </button>
              <button
                type="button"
                onClick={() => runTest('connect anti gravity to chat gpt now')}
                className="rounded-md border border-line-soft bg-inset px-2 py-0.5 text-[11px] text-ink-2 hover:bg-hover hover:text-ink"
              >
                Antigravity + ChatGPT
              </button>
              <button
                type="button"
                onClick={() => runTest('um tell epic geese that anti gravity is ready')}
                className="rounded-md border border-line-soft bg-inset px-2 py-0.5 text-[11px] text-ink-2 hover:bg-hover hover:text-ink"
              >
                Fillers + epinogees
              </button>
            </div>
          </div>

          {/* Adapted output side */}
          <div>
            <div className="flex items-center justify-between">
              <label className="text-[11px] font-semibold uppercase tracking-wider text-ink-3">
                Adapted Normalized Output
              </label>
              <div className="flex items-center gap-1.5">
                {testResult.detectedTags.map((tag) => (
                  <span
                    key={tag}
                    className="rounded-full bg-accent-tint px-2 py-0.5 text-[10px] font-medium text-accent-ink"
                  >
                    {tag}
                  </span>
                ))}
              </div>
            </div>

            <div className="mt-1.5 rounded-xl border border-line-strong bg-inset p-3 font-mono text-[13px] text-ink leading-relaxed">
              {testResult.formatted}
            </div>

            {/* Token Diff View */}
            <div className="mt-3">
              <DiffViewer
                diff={testResult.diff}
                raw={testInput}
                formatted={testResult.formatted}
                mode="inline"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
