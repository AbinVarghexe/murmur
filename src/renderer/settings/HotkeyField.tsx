import { useEffect, useState } from 'react'
import { Button } from '@/components/base/buttons/button'
import { Kbd } from '@/components/base/kbd/kbd'
import { acceleratorFromKeys, acceleratorLabels } from './accelerator'

interface HotkeyFieldProps {
  readonly label: string
  readonly description: string
  readonly value: string
  readonly onChange: (accelerator: string) => void
}

/** One hotkey row: shows the current combo and records a new one on request. */
export function HotkeyField({ label, description, value, onChange }: HotkeyFieldProps) {
  const [listening, setListening] = useState(false)

  useEffect(() => {
    if (!listening) return
    const onKeyDown = (event: KeyboardEvent): void => {
      event.preventDefault()
      if (event.key === 'Escape') {
        setListening(false)
        return
      }
      const accelerator = acceleratorFromKeys(event)
      if (!accelerator) return
      onChange(accelerator)
      setListening(false)
    }
    window.addEventListener('keydown', onKeyDown, true)
    return () => window.removeEventListener('keydown', onKeyDown, true)
  }, [listening, onChange])

  return (
    <div className="flex flex-wrap items-center justify-between gap-3 rounded-2xl bg-background-secondary-default px-4 py-3">
      <div className="min-w-0">
        <p className="text-body-medium text-text-primary">{label}</p>
        <p className="text-body-regular text-text-secondary">{description}</p>
      </div>
      <div className="flex items-center gap-3">
        {listening ? (
          <span className="text-body-medium text-text-secondary" aria-live="polite">
            Press your key… <span className="text-text-tertiary">(Esc to cancel)</span>
          </span>
        ) : (
          <span className="flex items-center gap-1" aria-label={`Current shortcut ${acceleratorLabels(value).join(' plus ')}`}>
            {acceleratorLabels(value).map((part) => (
              <Kbd key={part}>{part}</Kbd>
            ))}
          </span>
        )}
        <Button variant="secondary" size="small" onClick={() => setListening((current) => !current)}>
          {listening ? 'Cancel' : 'Change'}
        </Button>
      </div>
    </div>
  )
}
