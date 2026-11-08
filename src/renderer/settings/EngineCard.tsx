import { RiDownloadCloud2Line, RiMicFill } from '@remixicon/react'
import { Chip } from '@/components/base/badges/chip'
import { StatusDot } from '@/components/base/badges/status-dot'
import { Button } from '@/components/base/buttons/button'
import { ProgressBar } from '@/components/base/progress/progress'
import type { ModelController } from './useModel'

const MB = 1024 * 1024
const toMb = (bytes: number): number => Math.round(bytes / MB)

function headline(model: ModelController): { title: string; body: string } {
  if (model.ready) {
    return { title: 'Ready to listen', body: 'Speech recognition runs on this computer. No account, no quota, no audio leaves the device.' }
  }
  if (model.downloading) {
    return { title: 'Installing the speech engine', body: 'Keep this window open. The download is verified before it is used.' }
  }
  return { title: 'One-time setup', body: 'Download the speech model once and Murmur works offline, with no usage limits.' }
}

/** Hero card: engine state at a glance, with the one action that matters. */
export function EngineCard({ model }: { readonly model: ModelController }) {
  const { title, body } = headline(model)
  const percent = model.totalBytes > 0 ? Math.round((model.receivedBytes / model.totalBytes) * 100) : 0

  return (
    <section
      aria-label="Speech engine"
      className="relative overflow-hidden rounded-3xl border border-border-button-default bg-background-secondary-default p-6"
    >
      <div aria-hidden className="pointer-events-none absolute -end-16 -top-20 size-64 rounded-full bg-accent-500/15 blur-3xl" />
      <div className="relative flex flex-col gap-5">
        <div className="flex flex-wrap items-center gap-2">
          <StatusDot color={model.ready ? 'green' : model.downloading ? 'indigo' : 'yellow'} />
          <Chip color={model.ready ? 'lime' : model.downloading ? 'blue' : 'yellow'} variant="caption">
            {model.ready ? 'Installed · offline' : model.downloading ? 'Downloading' : 'Setup needed'}
          </Chip>
          <span className="text-caption-1-semibold text-text-tertiary">NVIDIA Parakeet TDT 0.6B · int8</span>
        </div>

        <div className="flex flex-col gap-1.5">
          <h2 className="text-title-1-medium text-text-primary">{title}</h2>
          <p className="max-w-xl text-body-regular text-text-secondary">{body}</p>
        </div>

        {model.downloading ? (
          <ProgressBar
            label="Downloading speech model"
            value={percent}
            description={`${toMb(model.receivedBytes)} of ${toMb(model.totalBytes)} MB`}
          />
        ) : null}

        {!model.ready && !model.downloading && model.loaded ? (
          <div>
            <Button leadingIcon={RiDownloadCloud2Line} onClick={() => void model.download()}>
              Download model ({toMb(model.totalBytes)} MB)
            </Button>
          </div>
        ) : null}

        {model.ready ? (
          <p className="flex items-center gap-2 text-body-regular text-text-secondary">
            <RiMicFill className="size-4 text-foreground-icon-tertiary" aria-hidden />
            Press your dictation hotkey anywhere to start.
          </p>
        ) : null}

        {model.error ? (
          <p role="alert" className="text-body-regular text-text-error-primary">
            {model.error}
          </p>
        ) : null}
      </div>
    </section>
  )
}
