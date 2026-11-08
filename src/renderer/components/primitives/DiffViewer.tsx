import type { DiffToken } from '@/lib/localTextModel'

interface DiffViewerProps {
  diff: DiffToken[]
  raw: string
  formatted: string
  mode?: 'inline' | 'split'
}

/**
 * Beautiful UI styled Diff Inspector.
 * Uses soft OKLCH tints and hairline borders to cleanly illustrate
 * what the Local AI model corrected and structured.
 */
export function DiffViewer({ diff, raw, formatted, mode = 'inline' }: DiffViewerProps) {
  if (diff.length === 0) {
    return (
      <div className="rounded-xl border border-line-soft bg-surface p-4 text-center text-body-regular text-ink-3">
        No differences detected. Text is already optimal.
      </div>
    )
  }

  if (mode === 'split') {
    return (
      <div className="grid gap-3 sm:grid-cols-2">
        <div className="flex flex-col gap-1.5 rounded-xl border border-line-soft bg-inset p-3.5">
          <div className="flex items-center justify-between">
            <span className="text-caption-1-semibold uppercase tracking-wider text-ink-3">Raw Transcript</span>
            <span className="rounded-full bg-field px-2 py-0.5 text-[11px] font-medium text-ink-3">Input</span>
          </div>
          <p className="whitespace-pre-wrap font-mono text-[13px] leading-relaxed text-ink-2">{raw}</p>
        </div>

        <div className="flex flex-col gap-1.5 rounded-xl border border-line-soft bg-surface p-3.5 shadow-sm">
          <div className="flex items-center justify-between">
            <span className="text-caption-1-semibold uppercase tracking-wider text-accent-ink">Local AI Formatted</span>
            <span className="rounded-full bg-green-tint px-2 py-0.5 text-[11px] font-medium text-green">Enhanced</span>
          </div>
          <p className="whitespace-pre-wrap font-mono text-[13px] leading-relaxed text-ink">{formatted}</p>
        </div>
      </div>
    )
  }

  return (
    <div className="rounded-xl border border-line-soft bg-surface p-3.5 shadow-sm">
      <div className="mb-2 flex items-center justify-between border-b border-line-soft pb-2">
        <span className="text-caption-1-semibold uppercase tracking-wider text-ink-3">Diff Stream</span>
        <div className="flex items-center gap-2 text-[11px]">
          <span className="inline-flex items-center gap-1 text-red">
            <span className="size-2 rounded-full bg-red" /> Removed
          </span>
          <span className="inline-flex items-center gap-1 text-green">
            <span className="size-2 rounded-full bg-green" /> Enhanced
          </span>
        </div>
      </div>

      <div className="whitespace-pre-wrap font-mono text-[13px] leading-relaxed text-ink">
        {diff.map((token, index) => {
          if (token.type === 'delete') {
            return (
              <del
                key={index}
                className="mx-0.5 rounded bg-red-tint px-1 py-0.5 text-red line-through decoration-red/60"
              >
                {token.value}
              </del>
            )
          }
          if (token.type === 'insert') {
            return (
              <ins
                key={index}
                className="mx-0.5 rounded bg-green-tint px-1 py-0.5 text-green no-underline font-medium"
              >
                {token.value}
              </ins>
            )
          }
          return <span key={index}>{token.value}</span>
        })}
      </div>
    </div>
  )
}
