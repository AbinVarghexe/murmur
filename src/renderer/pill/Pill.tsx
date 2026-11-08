import { useEffect, useState, useRef } from 'react'
import { StreamText } from '@/components/atoms/StreamText'
import ApprovalCard from '@/components/primitives/ApprovalCard'
import { SiriWave } from '@/components/ui/siri-wave'

const IDLE: MurmurPillState = { phase: 'idle', mode: 'dictation', message: '' }
const CONFIRM_OPTIONS = ['Yes, execute', 'No, cancel']

/**
 * Compact pill progress indicator with dynamic GLSL WebGL SiriWave animation
 * and live animated speech text in the field.
 */
function Progress({ state }: { readonly state: MurmurPillState }) {
  const isAgent = state.mode === 'agent'
  switch (state.phase) {
    case 'preparing':
      return (
        <div className="flex items-center gap-2">
          <SiriWave variant="fluid-dots" size={20} renderScale={1.5} className="rounded-full shrink-0" />
          <span className="text-[12px] font-medium text-ink-2">Opening mic…</span>
        </div>
      )
    case 'recording':
      return (
        <div className="flex items-center gap-2">
          {/* WebGL GLSL SiriWave reactive waveform */}
          <SiriWave
            variant={isAgent ? 'fluid-dots' : 'wave'}
            size={24}
            renderScale={1.5}
            className="rounded-full shrink-0 shadow-[0_0_10px_rgba(56,189,248,0.35)]"
          />
          {/* Animated speaking text */}
          <span className="flex items-center gap-1.5 text-[12px] font-semibold text-ink">
            <span>{isAgent ? 'Listening for command' : 'Listening'}</span>
            <span className="inline-flex gap-0.5">
              <span className="size-1 rounded-full bg-accent animate-bounce [animation-delay:0ms]" />
              <span className="size-1 rounded-full bg-accent animate-bounce [animation-delay:150ms]" />
              <span className="size-1 rounded-full bg-accent animate-bounce [animation-delay:300ms]" />
            </span>
          </span>
        </div>
      )
    case 'processing':
      return (
        <div className="flex items-center gap-2">
          <SiriWave variant="wave" size={20} renderScale={1.5} className="rounded-full shrink-0" />
          <span className="text-[12px] font-medium text-ink-2">
            {isAgent ? 'Running command…' : 'Formatting text…'}
          </span>
        </div>
      )
    case 'result':
      return (
        <div className="flex min-w-0 items-center gap-2">
          <span className="size-2 shrink-0 rounded-full bg-green" />
          <StreamText
            text={state.message}
            caret={false}
            className="truncate text-[12px] font-medium text-ink"
          />
        </div>
      )
    case 'notice':
      return (
        <div className="flex min-w-0 items-center gap-2">
          <span className="size-2 shrink-0 rounded-full bg-orange" />
          <span className="truncate text-[12px] text-ink-2">{state.message}</span>
        </div>
      )
    case 'idle':
    default:
      return (
        <div className="flex items-center gap-2">
          <span className="size-1.5 rounded-full bg-line-strong" />
          <span className="text-[11px] text-ink-3">Murmur Standby</span>
        </div>
      )
  }
}

/** A disruptive command waits here for a yes or no. Skipping counts as no. */
function Confirm({ summary, onAnswer }: { readonly summary: string; readonly onAnswer: (approved: boolean) => void }) {
  return (
    <div className="w-full max-w-sm rounded-2xl border border-line-strong bg-surface p-1 shadow-overlay">
      <ApprovalCard
        questions={[{ q: `${summary}?`, type: 'radio', options: CONFIRM_OPTIONS }]}
        labels={{ skip: 'Cancel', continue: 'Confirm & Run' }}
        resettable={false}
        onSubmitted={(answers) => onAnswer(answers[0]?.[0] === 0)}
      />
    </div>
  )
}

export function Pill() {
  const [state, setState] = useState<MurmurPillState>(IDLE)
  const [pendingSummary, setPendingSummary] = useState<string | null>(null)
  const autoHideTimerRef = useRef<number | null>(null)

  useEffect(() => {
    const offState = window.murmur.onPillState((next) => {
      setState(next)

      if (autoHideTimerRef.current) {
        clearTimeout(autoHideTimerRef.current)
        autoHideTimerRef.current = null
      }

      if (next.phase === 'result' || next.phase === 'notice') {
        setPendingSummary(null)
        // Client safety auto-hide fallback if nothing captured or after result
        autoHideTimerRef.current = window.setTimeout(() => {
          setState(IDLE)
        }, next.phase === 'notice' ? 2200 : 2600)
      } else if (next.phase === 'idle') {
        setPendingSummary(null)
      }
    })

    const offConfirm = window.murmur.onConfirm((request) => setPendingSummary(request.summary))
    return () => {
      offState()
      offConfirm()
      if (autoHideTimerRef.current) clearTimeout(autoHideTimerRef.current)
    }
  }, [])

  const answer = (approved: boolean): void => {
    setPendingSummary(null)
    window.murmur.confirm(approved)
  }

  const handleDismiss = () => {
    if (state.phase === 'result' || state.phase === 'notice') {
      setState(IDLE)
    }
  }

  const isRecording = state.phase === 'recording'
  const isAgent = state.mode === 'agent'

  return (
    <main role="status" aria-live="polite" className="p-1">
      {pendingSummary !== null ? (
        <Confirm summary={pendingSummary} onAnswer={answer} />
      ) : (
        <div
          onClick={handleDismiss}
          className={`relative flex h-9 w-full max-w-[270px] cursor-default items-center justify-between gap-2.5 rounded-full border border-line bg-surface/95 px-3 py-1 shadow-overlay backdrop-blur-md transition-all duration-150 ${
            isRecording
              ? isAgent
                ? 'ring-1.5 ring-orange/60'
                : 'ring-1.5 ring-accent/60'
              : ''
          }`}
        >
          {/* Progress message and circular Siri animation */}
          <div className="min-w-0 flex-1">
            <Progress state={state} />
          </div>

          {/* Mode Pill Tag */}
          <div className="flex shrink-0 items-center">
            <span
              className={`rounded-full px-2 py-0.5 text-[10px] font-semibold tracking-wide uppercase ${
                isAgent
                  ? 'bg-orange-tint text-orange'
                  : 'bg-field text-ink-3'
              }`}
            >
              {isAgent ? 'Cmd' : 'Dictate'}
            </span>
          </div>
        </div>
      )}
    </main>
  )
}
