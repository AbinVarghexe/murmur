import { useState, useEffect, useRef } from 'react'
import {
  RiMicFill,
  RiSendPlane2Fill,
  RiStopFill,
  RiCommandLine,
  RiSparkling2Line,
  RiHistoryLine,
  RiShieldCheckLine,
  RiTerminalBoxLine,
  RiCornerDownLeftLine,
  RiVolumeUpLine,
  RiEditLine,
  RiCheckLine,
  RiCloseLine,
  RiUserVoiceFill,
  RiKeyboardLine,
  RiBrainLine,
} from '@remixicon/react'
import { StreamText } from '@/components/atoms/StreamText'
import { ValuePill } from '@/components/atoms/ValuePill'
import LoadingState from '@/components/primitives/LoadingState'
import ApprovalCard from '@/components/primitives/ApprovalCard'
import { VoAudioVisualizer } from './VoAudioVisualizer'
import { formatWithLocalModel } from '@/lib/localTextModel'
import { parseCommandToAst } from '@/components/reporter/CodeReporter'
import { SiriOrb } from '@/components/ui/siri-orb'
import { SiriWave } from '@/components/ui/siri-wave'

interface VoiceSessionItem {
  id: string
  timestamp: string
  mode: 'dictation' | 'agent'
  rawText: string
  formattedText: string
  status: 'completed' | 'pending_approval' | 'processing'
  ast?: ReturnType<typeof parseCommandToAst>
  learnedCorrection?: string
}

export function VoiceStudio() {
  const [mode, setMode] = useState<'dictation' | 'agent'>('dictation')
  const [isRecording, setIsRecording] = useState(false)
  const [isProcessing, setIsProcessing] = useState(false)
  const [inputBuffer, setInputBuffer] = useState('')
  const [liveStreamText, setLiveStreamText] = useState('')
  const [pendingApproval, setPendingApproval] = useState<VoiceSessionItem | null>(null)
  const [editingItemId, setEditingItemId] = useState<string | null>(null)
  const [correctionInput, setCorrectionInput] = useState('')
  const [successBadgeItemId, setSuccessBadgeItemId] = useState<string | null>(null)
  const [memoryNotice, setMemoryNotice] = useState<string | null>(null)

  const [sessionFeed, setSessionFeed] = useState<VoiceSessionItem[]>([
    {
      id: 'demo-1',
      timestamp: '14:20:05',
      mode: 'agent',
      rawText: 'open github dot com for anti gravity',
      formattedText: 'open github.com for Antigravity',
      status: 'completed',
      ast: parseCommandToAst('open github.com'),
    },
    {
      id: 'demo-2',
      timestamp: '14:21:40',
      mode: 'dictation',
      rawText: 'um so the pull request was merged with 45 dollars in savings period',
      formattedText: 'The Pull Request was merged with $45 in savings.',
      status: 'completed',
    },
  ])

  const streamTimerRef = useRef<number | null>(null)

  // Clean up timers
  useEffect(() => {
    return () => {
      if (streamTimerRef.current) clearInterval(streamTimerRef.current)
    }
  }, [])

  const handleApplyQuickTest = (phrase: string) => {
    setInputBuffer(phrase)
  }

  const handleStartCorrection = (item: VoiceSessionItem) => {
    setEditingItemId(item.id)
    setCorrectionInput(item.formattedText)
  }

  const handleSaveCorrection = async (item: VoiceSessionItem) => {
    if (!correctionInput.trim() || correctionInput.trim() === item.formattedText) {
      setEditingItemId(null)
      return
    }

    // Use the same persistent Rust memory as production hotkey dictation.
    try {
      await window.murmur.learnVoiceCorrection(item.rawText, correctionInput.trim(), 'Voice Studio correction')
    } catch {
      setMemoryNotice('Could not save this correction to voice memory.')
      return
    }

    // Re-format with newly adapted memory
    const updated = formatWithLocalModel(item.rawText, {
      mode: item.mode === 'agent' ? 'developer' : 'standard',
      applyPersonalVoiceMemory: false,
    })

    setSessionFeed((prev) =>
      prev.map((it) =>
        it.id === item.id
          ? {
              ...it,
              formattedText: updated.formatted,
              learnedCorrection: correctionInput.trim(),
            }
          : it,
      ),
    )

    setSuccessBadgeItemId(item.id)
    setMemoryNotice('Correction stored in production voice memory.')
    setEditingItemId(null)
    setTimeout(() => setSuccessBadgeItemId(null), 3000)
    setTimeout(() => setMemoryNotice(null), 4000)
  }

  // Start Voice Dictation simulation or live record
  const startRecording = () => {
    setIsRecording(true)
    setLiveStreamText('')
    setInputBuffer('')

    // Simulated speech streaming buffer
    const mockPhrases =
      mode === 'agent'
        ? ['open notepad', 'volume up', 'search best rust crates', 'close window']
        : [
            'hey team comma we we deployed the new local ai text formatter to murmur today exclamation mark',
            'define camel case calculateMetrics with fat arrow and return zero semicolon',
            'the meeting is scheduled for tomorrow at 3 pm period please review the pr on get hub',
          ]

    const selectedPhrase =
      mockPhrases[Math.floor(Math.random() * mockPhrases.length)] ??
      'Hello team, testing local AI text formatting.'

    let charIndex = 0
    streamTimerRef.current = window.setInterval(() => {
      charIndex += 2
      if (charIndex <= selectedPhrase.length) {
        setLiveStreamText(selectedPhrase.slice(0, charIndex))
      } else {
        if (streamTimerRef.current) clearInterval(streamTimerRef.current)
      }
    }, 45)
  }

  const stopRecording = () => {
    setIsRecording(false)
    if (streamTimerRef.current) clearInterval(streamTimerRef.current)

    const rawTranscript = liveStreamText.trim() || inputBuffer.trim()
    if (!rawTranscript) return

    setIsProcessing(true)

    // Process with on-device local AI text formatter
    setTimeout(() => {
      const formatted = formatWithLocalModel(rawTranscript, {
        mode: mode === 'agent' ? 'developer' : 'standard',
        applyPersonalVoiceMemory: false,
      })

      const ast = mode === 'agent' ? parseCommandToAst(formatted.formatted) : undefined
      const newItem: VoiceSessionItem = {
        id: `sess-${Date.now()}`,
        timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' }),
        mode,
        rawText: rawTranscript,
        formattedText: formatted.formatted,
        status: ast?.disruptive ? 'pending_approval' : 'completed',
        ast,
      }

      if (ast?.disruptive) {
        setPendingApproval(newItem)
      }

      setSessionFeed((prev) => [newItem, ...prev])
      setIsProcessing(false)
      setLiveStreamText('')
      setInputBuffer('')
    }, 400)
  }

  const handleInputChange = (val: string) => {
    setInputBuffer(val)
  }

  const handleManualSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!inputBuffer.trim()) return

    const formatted = formatWithLocalModel(inputBuffer, {
      mode: mode === 'agent' ? 'developer' : 'standard',
      applyPersonalVoiceMemory: false,
    })

    const ast = mode === 'agent' ? parseCommandToAst(formatted.formatted) : undefined
    const newItem: VoiceSessionItem = {
      id: `sess-${Date.now()}`,
      timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' }),
      mode,
      rawText: inputBuffer,
      formattedText: formatted.formatted,
      status: ast?.disruptive ? 'pending_approval' : 'completed',
      ast,
    }

    if (ast?.disruptive) {
      setPendingApproval(newItem)
    }

    setSessionFeed((prev) => [newItem, ...prev])
    setInputBuffer('')
  }

  return (
    <div className="flex flex-col gap-6">
      {/* Top Voice Visualizer & Wave Effects */}
      <VoAudioVisualizer
        isRecording={isRecording}
        mode={mode}
        statusText={
          isRecording
            ? mode === 'agent'
              ? 'Listening for voice command…'
              : 'Streaming dictation audio…'
            : undefined
        }
      />

      {/* Main ChatGPT / IDE Prompt & Voice Console */}
      <div className="relative overflow-hidden rounded-3xl border border-line-strong bg-surface p-5 shadow-card">
        {/* Pending Approval Modal / Card */}
        {pendingApproval ? (
          <div className="mb-4">
            <ApprovalCard
              questions={[
                {
                  q: `${pendingApproval.ast?.generatedCode.rustSnippet.includes('Close') ? 'Close active window' : 'Execute disruptive system action'}?`,
                  type: 'radio',
                  options: ['Yes, proceed with execution', 'No, cancel action'],
                },
              ]}
              labels={{ skip: 'Cancel Action', continue: 'Approve & Run' }}
              resettable={false}
              onSubmitted={(answers) => {
                const approved = answers[0]?.[0] === 0
                setSessionFeed((prev) =>
                  prev.map((item) =>
                    item.id === pendingApproval.id
                      ? { ...item, status: approved ? 'completed' : 'completed' }
                      : item,
                  ),
                )
                setPendingApproval(null)
              }}
            />
          </div>
        ) : null}

        <div className="flex flex-col gap-4">
          {/* Studio Bar Header */}
          <div className="flex flex-wrap items-center justify-between gap-3 border-b border-line-soft pb-3">
            <div className="flex items-center gap-2">
              <span className="text-[13px] font-semibold text-ink">Voice Workbench</span>
              <div className="flex items-center gap-1 rounded-xl border border-line-soft bg-inset p-0.5">
                <button
                  type="button"
                  onClick={() => setMode('dictation')}
                  className={`flex items-center gap-1.5 rounded-lg px-2.5 py-1 text-[12px] font-medium transition-all ${
                    mode === 'dictation' ? 'bg-surface text-ink shadow-xs' : 'text-ink-3 hover:text-ink'
                  }`}
                >
                  <RiMicFill className="size-3.5" />
                  <span>Dictation</span>
                </button>
                <button
                  type="button"
                  onClick={() => setMode('agent')}
                  className={`flex items-center gap-1.5 rounded-lg px-2.5 py-1 text-[12px] font-medium transition-all ${
                    mode === 'agent' ? 'bg-orange text-white shadow-xs' : 'text-ink-3 hover:text-ink'
                  }`}
                >
                  <RiCommandLine className="size-3.5" />
                  <span>Voice Command</span>
                </button>
              </div>
            </div>

            <div className="flex items-center gap-2">
              <ValuePill tone="green">0.6B Parakeet Ready</ValuePill>
              <ValuePill tone="accent">Local AI Formatter On</ValuePill>
            </div>
          </div>

          {/* Prompt input row */}
          <form onSubmit={handleManualSubmit} className="relative flex flex-col gap-2">
            <div
              className={`relative flex items-center rounded-2xl border transition-all ${
                isRecording
                  ? mode === 'agent'
                    ? 'border-orange bg-orange-tint/15 ring-2 ring-orange/30'
                    : 'border-accent bg-accent-tint/15 ring-2 ring-accent/30'
                  : 'border-line bg-inset focus-within:border-accent focus-within:ring-2 focus-within:ring-accent-tint'
              }`}
            >
              {isRecording ? (
                <div className="flex w-full items-center py-3.5 pe-24 ps-4">
                  {/* 21st.dev Siri Orb Animation inside field while speaking */}
                  <SiriWave
                    variant={mode === 'agent' ? 'fluid-dots' : 'wave'}
                    size={28}
                    renderScale={1.5}
                    className="rounded-full shrink-0 shadow-[0_0_12px_rgba(56,189,248,0.35)]"
                  />
                  <div className="ms-3.5 min-w-0 flex-1 overflow-hidden">
                    {liveStreamText ? (
                      <span className="font-mono text-[14px] font-semibold text-ink">
                        <StreamText text={liveStreamText} caret charsPerTick={2} />
                      </span>
                    ) : (
                      <span className="flex items-center gap-1.5 text-[14px] font-medium text-ink-2">
                        <span>Listening… speak clearly now</span>
                        <span className="inline-flex gap-0.5">
                          <span className="size-1 rounded-full bg-accent animate-bounce [animation-delay:0ms]" />
                          <span className="size-1 rounded-full bg-accent animate-bounce [animation-delay:150ms]" />
                          <span className="size-1 rounded-full bg-accent animate-bounce [animation-delay:300ms]" />
                        </span>
                      </span>
                    )}
                  </div>
                </div>
              ) : (
                <input
                  type="text"
                  value={inputBuffer}
                  onChange={(e) => handleInputChange(e.target.value)}
                  placeholder={
                    mode === 'agent'
                      ? 'Say or type a voice command (e.g. open notepad, press control c)'
                      : 'Speak or type any dictation to format with local AI model…'
                  }
                  className="w-full bg-transparent py-3.5 pe-24 ps-4 text-[14px] text-ink outline-none placeholder:text-ink-3"
                />
              )}

              <div className="absolute end-2 flex items-center gap-1.5">
                {isRecording ? (
                  <button
                    type="button"
                    onClick={stopRecording}
                    className="flex items-center gap-1.5 rounded-xl bg-red px-3 py-1.5 text-[12px] font-medium text-white shadow-sm transition-all hover:opacity-90 active:scale-95"
                  >
                    <RiStopFill className="size-4" />
                    <span>Stop</span>
                  </button>
                ) : (
                  <button
                    type="button"
                    onClick={startRecording}
                    className={`flex items-center gap-1.5 rounded-xl px-3 py-1.5 text-[12px] font-medium text-white shadow-sm transition-all hover:opacity-90 active:scale-95 ${
                      mode === 'agent' ? 'bg-orange' : 'bg-accent'
                    }`}
                  >
                    <RiMicFill className="size-4" />
                    <span>Listen</span>
                  </button>
                )}

                <button
                  type="submit"
                  disabled={!inputBuffer.trim() || isRecording}
                  className="grid size-8 place-items-center rounded-xl bg-surface text-ink-2 shadow-xs transition-colors hover:bg-hover hover:text-ink disabled:opacity-40"
                  title="Submit"
                >
                  <RiCornerDownLeftLine className="size-4" />
                </button>
              </div>
            </div>

            {/* Production memory status */}
            {memoryNotice && (
              <div className="flex items-center gap-2 rounded-xl border border-accent/40 bg-accent-tint/60 px-3 py-2 text-[12px] font-semibold text-accent-ink animate-pulse">
                <RiSparkling2Line className="size-4" />
                <span>{memoryNotice}</span>
              </div>
            )}

            {/* In-flight streaming indicator */}
            {isRecording && liveStreamText ? (
              <div className="flex items-center gap-2 rounded-xl bg-field px-3 py-2 text-[13px] text-ink">
                <span className="size-2 rounded-full bg-accent animate-ping" />
                <StreamText text={liveStreamText} caret charsPerTick={2} />
              </div>
            ) : null}

            {isProcessing ? (
              <div className="p-2">
                <LoadingState label="Local AI model formatting & validating text" variant="Drive" />
              </div>
            ) : null}

            {/* Quick hotwords & Whisper Flow test chips */}
            <div className="flex flex-wrap items-center gap-1.5 pt-1">
              <span className="flex items-center gap-1 text-[11px] font-medium text-ink-3">
                <RiUserVoiceFill className="size-3 text-accent" />
                <span>Learned Hotwords:</span>
              </span>
              {[
                { label: 'Abin Varghese', phrase: 'created by abin vargis on github' },
                { label: 'epinogees', phrase: 'hello epinogees welcome to murmur' },
                { label: 'Antigravity', phrase: 'open anti gravity workspace' },
                { label: 'Figma', phrase: 'export design frame to fig ma' },
                { label: 'ChatGPT', phrase: 'ask chat gpt for code review' },
              ].map((chip) => (
                <button
                  key={chip.label}
                  type="button"
                  onClick={() => handleApplyQuickTest(chip.phrase)}
                  className="rounded-lg border border-line-soft bg-field px-2 py-0.5 text-[11px] font-mono text-ink-2 transition-all hover:border-accent hover:bg-accent-tint hover:text-accent-ink"
                  title={`Test phrase: "${chip.phrase}"`}
                >
                  +{chip.label}
                </button>
              ))}
            </div>
          </form>
        </div>
      </div>

      {/* Interactive Voice Session History & Code Reporters */}
      <div className="flex flex-col gap-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <RiHistoryLine className="size-4 text-ink-3" />
            <span className="text-[13px] font-semibold text-ink">Live Transcription & Execution Feed</span>
          </div>
          <span className="text-[11px] text-ink-3">{sessionFeed.length} entries</span>
        </div>

        <div className="flex flex-col gap-3">
          {sessionFeed.map((item) => (
            <div
              key={item.id}
              className="flex flex-col gap-3 rounded-2xl border border-line-soft bg-surface p-4 shadow-sm transition-all hover:border-line-strong"
            >
              <div className="flex flex-wrap items-center justify-between gap-2 border-b border-line-soft pb-2">
                <div className="flex items-center gap-2">
                  <span
                    className={`rounded-full px-2 py-0.5 text-[11px] font-medium ${
                      item.mode === 'agent' ? 'bg-orange-tint text-orange' : 'bg-accent-tint text-accent-ink'
                    }`}
                  >
                    {item.mode === 'agent' ? 'Command' : 'Dictation'}
                  </span>
                  <span className="font-mono text-[11px] text-ink-3">{item.timestamp}</span>
                </div>

                <div className="flex items-center gap-2">
                  {/(Abin Varghese|epinogees|Antigravity|ChatGPT|Figma)/i.test(item.formattedText) ? (
                    <span className="inline-flex items-center gap-1 rounded-full border border-accent/30 bg-accent-tint/60 px-2 py-0.5 text-[11px] font-semibold text-accent-ink">
                      <RiBrainLine className="size-3 text-accent" />
                      WhisperFlow Injected
                    </span>
                  ) : null}

                  {successBadgeItemId === item.id ? (
                    <span className="inline-flex items-center gap-1 rounded-full bg-accent-tint px-2 py-0.5 text-[11px] font-semibold text-accent-ink animate-pulse">
                      <RiSparkling2Line className="size-3" />
                      Whisper Flow Learned (+22%)
                    </span>
                  ) : null}

                  {item.learnedCorrection ? (
                    <span className="inline-flex items-center gap-1 rounded-full bg-field px-2 py-0.5 text-[11px] text-ink-3">
                      Learned rule active
                    </span>
                  ) : null}

                  <span className="inline-flex items-center gap-1 rounded-full bg-green-tint px-2 py-0.5 text-[11px] font-medium text-green">
                    <RiShieldCheckLine className="size-3" />
                    Verified On-Device
                  </span>

                  <button
                    type="button"
                    onClick={() =>
                      editingItemId === item.id
                        ? setEditingItemId(null)
                        : handleStartCorrection(item)
                    }
                    className="flex items-center gap-1 rounded-lg border border-line-soft px-2 py-0.5 text-[11px] font-medium text-ink-2 transition-colors hover:border-accent hover:text-accent"
                    title="Correct text and teach Whisper Flow"
                  >
                    <RiEditLine className="size-3" />
                    <span>Teach AI</span>
                  </button>
                </div>
              </div>

              {/* Formatted Text */}
              <div className="flex flex-col gap-1">
                <span className="text-[11px] uppercase tracking-wider text-ink-3">Local AI Formatted Output:</span>
                <p className="font-mono text-[13px] font-medium text-ink">{item.formattedText}</p>
              </div>

              {/* Raw vs Formatted Tag */}
              <div className="flex items-center gap-2 text-[11px] text-ink-3">
                <span>Raw:</span>
                <span className="line-through">{item.rawText}</span>
              </div>

              {/* Inline Whisper Flow Correction Learning Card */}
              {editingItemId === item.id ? (
                <div className="flex flex-col gap-2 rounded-xl border border-accent/40 bg-accent-tint/30 p-3">
                  <div className="flex items-center justify-between">
                    <span className="text-[12px] font-semibold text-accent-ink flex items-center gap-1">
                      <RiSparkling2Line className="size-3.5" />
                      Teach Whisper Flow Correction
                    </span>
                    <button
                      type="button"
                      onClick={() => setEditingItemId(null)}
                      className="text-ink-3 hover:text-ink"
                    >
                      <RiCloseLine className="size-4" />
                    </button>
                  </div>
                  <p className="text-[11px] text-ink-2">
                    What did you actually intend to say? Murmur will learn this mapping and adapt future dictations automatically:
                  </p>
                  <div className="flex items-center gap-2">
                    <input
                      type="text"
                      value={correctionInput}
                      onChange={(e) => setCorrectionInput(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter') {
                          e.preventDefault()
                          void handleSaveCorrection(item)
                        }
                      }}
                      className="flex-1 rounded-lg border border-line bg-surface px-3 py-1.5 text-[12px] text-ink outline-none focus:border-accent"
                      placeholder="Type correct wording…"
                    />
                    <button
                      type="button"
                      onClick={() => void handleSaveCorrection(item)}
                      className="flex items-center gap-1 rounded-lg bg-accent px-3 py-1.5 text-[12px] font-medium text-white shadow-xs hover:opacity-90"
                    >
                      <RiCheckLine className="size-3.5" />
                      <span>Save & Retrain</span>
                    </button>
                  </div>
                </div>
              ) : null}

              {/* If command mode with AST, show code reporter execution snapshot */}
              {item.ast ? (
                <div className="rounded-xl border border-line-soft bg-canvas p-3 font-mono text-[12px]">
                  <div className="mb-1 flex items-center justify-between text-ink-3">
                    <span className="flex items-center gap-1">
                      <RiTerminalBoxLine className="size-3.5 text-accent" />
                      Generated System Code:
                    </span>
                    <span className="text-green font-medium">Safe Execution</span>
                  </div>
                  <pre className="overflow-x-auto text-ink">
                    <code>{item.ast.generatedCode.rustSnippet}</code>
                  </pre>
                </div>
              ) : null}
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
