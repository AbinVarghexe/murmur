import { useEffect, useRef, useState } from 'react'
import { RiMicFill, RiMicOffFill, RiPulseLine } from '@remixicon/react'

export interface VoAudioVisualizerProps {
  isRecording?: boolean
  mode?: 'dictation' | 'agent'
  statusText?: string
  onToggleMic?: () => void
  interactive?: boolean
}

const BAR_COUNT = 28

/**
 * VO Audio Reactive Visualizer & Sound Wave Effects
 *
 * Implements fluid voice animation, real-time waveform bars,
 * acoustic energy halos, and peak decibel meters for high-end interactive voice studio feedback.
 */
export function VoAudioVisualizer({
  isRecording = false,
  mode = 'dictation',
  statusText,
  onToggleMic,
  interactive = true,
}: VoAudioVisualizerProps) {
  const [liveDb, setLiveDb] = useState(-48)
  const [barHeights, setBarHeights] = useState<number[]>(() => Array.from({ length: BAR_COUNT }, () => 15))
  const [isLiveMicActive, setIsLiveMicActive] = useState(false)
  const [micError, setMicError] = useState<string | null>(null)

  const audioCtxRef = useRef<AudioContext | null>(null)
  const analyserRef = useRef<AnalyserNode | null>(null)
  const animFrameRef = useRef<number | null>(null)
  const streamRef = useRef<MediaStream | null>(null)

  // Start / Stop real mic analysis if requested
  const toggleRealMic = async () => {
    if (isLiveMicActive) {
      if (streamRef.current) {
        streamRef.current.getTracks().forEach((track) => track.stop())
        streamRef.current = null
      }
      if (audioCtxRef.current) {
        void audioCtxRef.current.close()
        audioCtxRef.current = null
      }
      setIsLiveMicActive(false)
      return
    }

    try {
      setMicError(null)
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
      streamRef.current = stream
      const ctx = new (window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext)()
      const analyser = ctx.createAnalyser()
      analyser.fftSize = 64
      const source = ctx.createMediaStreamSource(stream)
      source.connect(analyser)

      audioCtxRef.current = ctx
      analyserRef.current = analyser
      setIsLiveMicActive(true)
    } catch {
      setMicError('Microphone access was not permitted. Using neural synthetic wave.')
      setIsLiveMicActive(false)
    }
  }

  // Animation loop for both real microphone audio and synthetic voice wave
  useEffect(() => {
    let phase = 0

    const updateFrame = () => {
      phase += 0.08
      const active = isRecording || isLiveMicActive

      if (isLiveMicActive && analyserRef.current) {
        const data = new Uint8Array(analyserRef.current.frequencyBinCount)
        analyserRef.current.getByteFrequencyData(data)

        let sum = 0
        const newBars: number[] = []
        for (let i = 0; i < BAR_COUNT; i += 1) {
          const dataIdx = Math.floor((i / BAR_COUNT) * data.length)
          const val = data[dataIdx] || 0
          sum += val
          // Scale 0-255 to 10-100% height
          const height = Math.max(12, Math.min(100, (val / 255) * 100))
          newBars.push(height)
        }
        setBarHeights(newBars)

        const avg = sum / (data.length || 1)
        const db = Math.round((avg / 255) * 50 - 50)
        setLiveDb(db)
      } else if (active) {
        // High fidelity synthetic voice wave simulation
        const newBars: number[] = []
        let energySum = 0
        for (let i = 0; i < BAR_COUNT; i += 1) {
          const distFromCenter = Math.abs(i - BAR_COUNT / 2) / (BAR_COUNT / 2)
          const baseHeight = 15 + (1 - distFromCenter) * 45
          const wave1 = Math.sin(phase * 1.5 + i * 0.45) * 22
          const wave2 = Math.cos(phase * 0.9 + i * 0.25) * 18
          const noise = ((i * 17) % 7) * 2.5
          const height = Math.max(10, Math.min(95, baseHeight + wave1 + wave2 + noise))
          newBars.push(height)
          energySum += height
        }
        setBarHeights(newBars)
        const avg = energySum / BAR_COUNT
        setLiveDb(Math.round(avg * 0.4 - 45))
      } else {
        // Idle gentle breathing ripple
        const idleBars = Array.from({ length: BAR_COUNT }, (_, i) => {
          const sine = Math.sin(phase * 0.4 + i * 0.2) * 6
          return 14 + sine
        })
        setBarHeights(idleBars)
        setLiveDb(-54)
      }

      animFrameRef.current = requestAnimationFrame(updateFrame)
    }

    animFrameRef.current = requestAnimationFrame(updateFrame)

    return () => {
      if (animFrameRef.current) cancelAnimationFrame(animFrameRef.current)
    }
  }, [isRecording, isLiveMicActive])

  // Cleanup media streams on unmount
  useEffect(() => {
    return () => {
      if (streamRef.current) {
        streamRef.current.getTracks().forEach((track) => track.stop())
      }
      if (audioCtxRef.current) {
        void audioCtxRef.current.close()
      }
    }
  }, [])

  const active = isRecording || isLiveMicActive

  return (
    <div className="relative overflow-hidden rounded-2xl border border-line-soft bg-surface p-5 shadow-sm transition-all duration-300">
      {/* Background ambient voice glow aura */}
      <div
        aria-hidden
        className={`pointer-events-none absolute -top-12 start-1/2 size-48 -translate-x-1/2 rounded-full blur-3xl transition-opacity duration-700 ${
          active
            ? mode === 'agent'
              ? 'bg-orange/25 opacity-100'
              : 'bg-accent/25 opacity-100'
            : 'bg-accent-tint/40 opacity-40'
        }`}
      />

      <div className="relative flex flex-col gap-4">
        {/* Header telemetry row */}
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex items-center gap-2.5">
            <span
              className={`relative grid size-8 place-items-center rounded-xl transition-all ${
                active
                  ? mode === 'agent'
                    ? 'bg-orange text-white shadow-md ring-4 ring-orange-tint'
                    : 'bg-accent text-white shadow-md ring-4 ring-accent-tint'
                  : 'bg-field text-ink-2'
              }`}
            >
              {active ? <RiPulseLine className="size-4 animate-spin" /> : <RiMicFill className="size-4" />}
            </span>
            <div>
              <div className="flex items-center gap-1.5">
                <span className="text-[13px] font-semibold text-ink">
                  {active ? (mode === 'agent' ? 'Command Listening' : 'Dictation Live') : 'Acoustic Standby'}
                </span>
                <span
                  className={`inline-block size-2 rounded-full ${
                    active ? (mode === 'agent' ? 'bg-orange animate-ping' : 'bg-green animate-ping') : 'bg-ink-3'
                  }`}
                />
              </div>
              <p className="text-[11px] text-ink-3">
                {statusText || (active ? 'Acoustic energy capturing in 16kHz mono' : 'Press hotkey or toggle mic')}
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            {/* Decibel readout */}
            <div className="flex items-center gap-1.5 rounded-lg border border-line-soft bg-inset px-2.5 py-1 font-mono text-[11px] text-ink-2">
              <span className="text-ink-3">GAIN:</span>
              <span className={liveDb > -20 ? 'text-orange font-semibold' : 'text-ink'}>{liveDb} dBFS</span>
            </div>

            {interactive ? (
              <button
                type="button"
                onClick={() => {
                  void toggleRealMic()
                  onToggleMic?.()
                }}
                className={`flex items-center gap-1.5 rounded-lg px-2.5 py-1 text-[12px] font-medium transition-colors ${
                  isLiveMicActive
                    ? 'bg-red-tint text-red ring-1 ring-red/30'
                    : 'bg-field text-ink-2 hover:bg-hover hover:text-ink'
                }`}
              >
                {isLiveMicActive ? (
                  <>
                    <RiMicOffFill className="size-3.5" />
                    <span>Stop Mic</span>
                  </>
                ) : (
                  <>
                    <RiMicFill className="size-3.5" />
                    <span>Test Mic</span>
                  </>
                )}
              </button>
            ) : null}
          </div>
        </div>

        {/* Real-time soundwave bar visualizer */}
        <div className="flex h-16 w-full items-center justify-between gap-[3px] rounded-xl border border-line-soft bg-inset px-3 py-2">
          {barHeights.map((height, i) => {
            const isCenter = Math.abs(i - BAR_COUNT / 2) < 4
            return (
              <div key={i} className="flex h-full flex-1 items-center justify-center">
                <div
                  className={`w-full rounded-full transition-all duration-75 ${
                    active
                      ? mode === 'agent'
                        ? isCenter
                          ? 'bg-orange'
                          : 'bg-orange/70'
                        : isCenter
                          ? 'bg-accent'
                          : 'bg-accent/70'
                      : 'bg-line-strong'
                  }`}
                  style={{
                    height: `${height}%`,
                    minHeight: '4px',
                  }}
                />
              </div>
            )
          })}
        </div>

        {micError ? (
          <p className="text-[11px] text-orange">{micError}</p>
        ) : (
          <div className="flex items-center justify-between text-[11px] text-ink-3">
            <span>Neural VAD · 0.6B Parakeet Engine</span>
            <span>Zero cloud transfer · 100% on-device</span>
          </div>
        )}
      </div>
    </div>
  )
}
