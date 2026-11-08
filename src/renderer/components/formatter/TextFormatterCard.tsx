import { useState, useTransition } from 'react'
import {
  RiMagicLine,
  RiFileCopyLine,
  RiCheckLine,
  RiSparkling2Line,
  RiSpeedUpLine,
  RiShieldCheckLine,
  RiCodeLine,
  RiListCheck2,
  RiMarkdownLine,
  RiChat1Line,
} from '@remixicon/react'
import {
  formatWithLocalModel,
  PRESET_EXAMPLES,
  type FormatterMode,
  type ModelFormatOptions,
  DEFAULT_FORMAT_OPTIONS,
} from '@/lib/localTextModel'
import { DiffViewer } from '@/components/primitives/DiffViewer'

export function TextFormatterCard() {
  const [inputText, setInputText] = useState(PRESET_EXAMPLES[0]?.sample ?? '')
  const [options, setOptions] = useState<ModelFormatOptions>(DEFAULT_FORMAT_OPTIONS)
  const [diffMode, setDiffMode] = useState<'inline' | 'split'>('inline')
  const [copied, setCopied] = useState(false)
  const [, startTransition] = useTransition()

  const formattedResult = formatWithLocalModel(inputText, options)

  const handleSelectPreset = (sample: string, mode: FormatterMode) => {
    startTransition(() => {
      setInputText(sample)
      setOptions((prev) => ({ ...prev, mode }))
    })
  }

  const handleCopy = () => {
    void navigator.clipboard.writeText(formattedResult.formatted)
    setCopied(true)
    setTimeout(() => setCopied(false), 1800)
  }

  const modes: { id: FormatterMode; label: string; icon: typeof RiChat1Line }[] = [
    { id: 'standard', label: 'Prose', icon: RiChat1Line },
    { id: 'developer', label: 'Developer', icon: RiCodeLine },
    { id: 'concise', label: 'Concise', icon: RiListCheck2 },
    { id: 'markdown', label: 'Markdown', icon: RiMarkdownLine },
  ]

  return (
    <section className="flex flex-col gap-5 rounded-3xl border border-line-strong bg-surface p-6 shadow-card transition-all">
      {/* Header */}
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-line-soft pb-4">
        <div className="flex items-center gap-3">
          <span className="grid size-10 place-items-center rounded-2xl bg-accent text-white shadow-sm">
            <RiMagicLine className="size-5" />
          </span>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-[16px] font-semibold text-ink">Local AI Text Formatter</h2>
              <span className="rounded-full bg-accent-tint px-2 py-0.5 text-[11px] font-medium text-accent-ink">
                On-Device Neural Model
              </span>
            </div>
            <p className="text-[12px] text-ink-3">
              Trained lightweight model that repairs speech disfluencies, technical wordings, and code syntax in 0ms
            </p>
          </div>
        </div>

        {/* Telemetry pill */}
        <div className="flex items-center gap-2">
          <div className="flex items-center gap-1.5 rounded-full border border-line-soft bg-field px-3 py-1 font-mono text-[11px] text-ink-2">
            <RiSpeedUpLine className="size-3.5 text-accent" />
            <span>{formattedResult.processingTimeMs} ms</span>
          </div>

          <div className="flex items-center gap-1.5 rounded-full border border-line-soft bg-green-tint px-3 py-1 text-[11px] font-medium text-green">
            <RiShieldCheckLine className="size-3.5" />
            <span>Private · Offline</span>
          </div>
        </div>
      </div>

      {/* Preset selection bar */}
      <div className="flex flex-col gap-2">
        <span className="text-caption-1-semibold uppercase tracking-wider text-ink-3">Dictation Presets</span>
        <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-4">
          {PRESET_EXAMPLES.map((preset) => (
            <button
              key={preset.id}
              type="button"
              onClick={() => handleSelectPreset(preset.sample, preset.mode)}
              className="flex flex-col items-start gap-1 rounded-xl border border-line-soft bg-inset p-3 text-start transition-all hover:border-line-strong hover:bg-hover active:scale-[0.99]"
            >
              <div className="flex w-full items-center justify-between">
                <span className="text-[12px] font-semibold text-ink">{preset.title}</span>
                <span className="rounded bg-field px-1.5 py-0.5 text-[10px] uppercase font-mono text-ink-3">
                  {preset.mode}
                </span>
              </div>
              <p className="line-clamp-2 text-[11px] text-ink-3">{preset.subtitle}</p>
            </button>
          ))}
        </div>
      </div>

      {/* Options bar */}
      <div className="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-line-soft bg-inset p-3">
        {/* Mode selector */}
        <div className="flex items-center gap-1 rounded-xl border border-line-soft bg-surface p-1 shadow-xs">
          {modes.map((m) => {
            const Icon = m.icon
            const isSelected = options.mode === m.id
            return (
              <button
                key={m.id}
                type="button"
                onClick={() => setOptions((prev) => ({ ...prev, mode: m.id }))}
                className={`flex items-center gap-1.5 rounded-lg px-2.5 py-1 text-[12px] font-medium transition-all ${
                  isSelected ? 'bg-accent text-white shadow-xs' : 'text-ink-2 hover:bg-hover hover:text-ink'
                }`}
              >
                <Icon className="size-3.5" />
                <span>{m.label}</span>
              </button>
            )
          })}
        </div>

        {/* Feature toggles */}
        <div className="flex flex-wrap items-center gap-1.5">
          <label className="flex cursor-pointer items-center gap-1.5 rounded-lg bg-field px-2.5 py-1 text-[11px] font-medium text-ink-2 transition-colors hover:bg-hover">
            <input
              type="checkbox"
              checked={options.removeFillers}
              onChange={(e) => setOptions((prev) => ({ ...prev, removeFillers: e.target.checked }))}
              className="rounded accent-accent"
            />
            <span>Strip Fillers</span>
          </label>

          <label className="flex cursor-pointer items-center gap-1.5 rounded-lg bg-field px-2.5 py-1 text-[11px] font-medium text-ink-2 transition-colors hover:bg-hover">
            <input
              type="checkbox"
              checked={options.fixTechnicalTerms}
              onChange={(e) => setOptions((prev) => ({ ...prev, fixTechnicalTerms: e.target.checked }))}
              className="rounded accent-accent"
            />
            <span>Fix Tech Wording</span>
          </label>

          <label className="flex cursor-pointer items-center gap-1.5 rounded-lg bg-field px-2.5 py-1 text-[11px] font-medium text-ink-2 transition-colors hover:bg-hover">
            <input
              type="checkbox"
              checked={options.codeIdentifierExpansion}
              onChange={(e) => setOptions((prev) => ({ ...prev, codeIdentifierExpansion: e.target.checked }))}
              className="rounded accent-accent"
            />
            <span>Code Identifiers</span>
          </label>

          <label className="flex cursor-pointer items-center gap-1.5 rounded-lg bg-field px-2.5 py-1 text-[11px] font-medium text-ink-2 transition-colors hover:bg-hover">
            <input
              type="checkbox"
              checked={options.smartPunctuation}
              onChange={(e) => setOptions((prev) => ({ ...prev, smartPunctuation: e.target.checked }))}
              className="rounded accent-accent"
            />
            <span>Smart Punctuation</span>
          </label>
        </div>
      </div>

      {/* Text input area */}
      <div className="flex flex-col gap-2">
        <div className="flex items-center justify-between">
          <label htmlFor="raw-transcript-input" className="text-[12px] font-medium text-ink-2">
            Raw Speech Transcript
          </label>
          <span className="text-[11px] text-ink-3">Live re-formatted on keystroke</span>
        </div>
        <textarea
          id="raw-transcript-input"
          rows={3}
          value={inputText}
          onChange={(e) => setInputText(e.target.value)}
          placeholder="Speak or paste any unformatted transcription here…"
          className="w-full rounded-2xl border border-line bg-inset p-3.5 font-mono text-[13px] leading-relaxed text-ink outline-none transition-all focus:border-accent focus:ring-2 focus:ring-accent-tint"
        />
      </div>

      {/* Formatted output card */}
      <div className="flex flex-col gap-3 rounded-2xl border border-line-soft bg-inset p-4 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-2 border-b border-line-soft pb-3">
          <div className="flex items-center gap-2">
            <RiSparkling2Line className="size-4 text-accent" />
            <span className="text-[13px] font-semibold text-ink">Formatted Result</span>
            <span className="rounded-full bg-field px-2 py-0.5 text-[11px] font-medium text-ink-3">
              {formattedResult.correctionsCount} repairs made
            </span>
          </div>

          <div className="flex items-center gap-2">
            <div className="flex items-center gap-1 rounded-lg border border-line-soft bg-surface p-0.5 text-[11px]">
              <button
                type="button"
                onClick={() => setDiffMode('inline')}
                className={`rounded px-2 py-0.5 font-medium transition-colors ${
                  diffMode === 'inline' ? 'bg-field text-ink' : 'text-ink-3 hover:text-ink'
                }`}
              >
                Inline Diff
              </button>
              <button
                type="button"
                onClick={() => setDiffMode('split')}
                className={`rounded px-2 py-0.5 font-medium transition-colors ${
                  diffMode === 'split' ? 'bg-field text-ink' : 'text-ink-3 hover:text-ink'
                }`}
              >
                Split View
              </button>
            </div>

            <button
              type="button"
              onClick={handleCopy}
              className="flex items-center gap-1.5 rounded-lg bg-surface px-3 py-1 text-[12px] font-medium text-ink shadow-xs transition-all hover:bg-hover active:scale-[0.98]"
            >
              {copied ? <RiCheckLine className="size-3.5 text-green" /> : <RiFileCopyLine className="size-3.5" />}
              <span>{copied ? 'Copied' : 'Copy Text'}</span>
            </button>
          </div>
        </div>

        {/* Visual Diff View */}
        <DiffViewer
          diff={formattedResult.diff}
          raw={formattedResult.raw}
          formatted={formattedResult.formatted}
          mode={diffMode}
        />

        {/* Clean Output Textarea */}
        <div className="mt-1 flex flex-col gap-1.5">
          <span className="text-[11px] font-medium text-ink-3">Clean Render:</span>
          <div className="rounded-xl border border-line-soft bg-surface p-3 font-mono text-[13px] leading-relaxed text-ink shadow-xs">
            {formattedResult.formatted}
          </div>
        </div>
      </div>
    </section>
  )
}
