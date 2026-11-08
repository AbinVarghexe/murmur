import { RiKeyboardBoxLine } from '@remixicon/react'
import { SegmentedControl, SegmentedControlItem } from '@/components/base/segmented-control/segmented-control'
import { Card } from './Card'
import { HotkeyField } from './HotkeyField'
import type { SettingsController } from './useSettings'

/** Seconds of quiet that end a recording. 0 turns auto-finish off. */
const AUTO_STOP_CHOICES = [0, 2, 3, 4, 5] as const

export function HotkeysCard({ settings }: { readonly settings: SettingsController }) {
  const draft = settings.draft
  if (!draft) return null
  return (
    <Card title="Hotkeys" description="Hold to talk, or tap to start and tap again to finish." icon={RiKeyboardBoxLine}>
      <div className="flex flex-col gap-3">
        <HotkeyField
          label="Dictation"
          description="Types what you say into the app you are using."
          value={draft.dictationHotkey}
          onChange={(accelerator) => settings.update({ dictationHotkey: accelerator })}
        />
        <HotkeyField
          label="Voice command"
          description="Opens apps, searches, controls media and more."
          value={draft.agentHotkey}
          onChange={(accelerator) => settings.update({ agentHotkey: accelerator })}
        />
      </div>
      <div className="flex flex-wrap items-center justify-between gap-3 rounded-2xl bg-background-secondary-default px-4 py-3">
        <div className="min-w-0">
          <p className="text-body-medium text-text-primary">Finish automatically</p>
          <p className="text-body-regular text-text-secondary">
            Transcribes after this much quiet, so you never need to tap again.
          </p>
        </div>
        <SegmentedControl
          aria-label="Finish automatically after silence"
          selectedKeys={[String(draft.autoStopSeconds)]}
          onSelectionChange={(keys) => {
            const seconds = Number([...keys][0])
            if (Number.isFinite(seconds)) settings.update({ autoStopSeconds: seconds })
          }}
        >
          {AUTO_STOP_CHOICES.map((seconds) => (
            <SegmentedControlItem key={seconds} id={String(seconds)}>
              {seconds === 0 ? 'Off' : `${seconds}s`}
            </SegmentedControlItem>
          ))}
        </SegmentedControl>
      </div>
      <p className="text-body-regular text-text-secondary">
        Press <strong className="font-medium text-text-primary">Change</strong>, then press the key you want. A single key
        works too: Right Alt, Right Ctrl, Right Shift, Caps Lock, Scroll Lock or F13–F24. Hold it to talk and let go to
        finish, or tap it once to start. Either way it also finishes by itself after a pause.
      </p>
      <p className="text-body-regular text-text-tertiary">
        The Fn key can’t be used: laptops handle it in firmware and never report it to Windows.
      </p>
    </Card>
  )
}
