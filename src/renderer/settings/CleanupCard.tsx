import { RiMagicLine } from '@remixicon/react'
import { SwitchCard } from '@/components/base/switch/switch-card'
import { Card } from './Card'
import type { SettingsController } from './useSettings'

export function CleanupCard({ settings }: { readonly settings: SettingsController }) {
  const draft = settings.draft
  if (!draft) return null
  const { cleanup } = draft
  return (
    <Card title="Text cleanup" description="Tidy the transcript before it is typed." icon={RiMagicLine}>
      <div className="flex flex-col gap-2">
        <SwitchCard
          title="Remove filler words"
          description="Drops “um”, “uh” and similar."
          isSelected={cleanup.removeFillers}
          onChange={(value) => settings.updateCleanup({ removeFillers: value })}
        />
        <SwitchCard
          title="Smart formatting"
          description="Fixes spacing, “I”, and repeated words."
          isSelected={cleanup.smartFormat}
          onChange={(value) => settings.updateCleanup({ smartFormat: value })}
        />
        <SwitchCard
          title="Capitalize sentences"
          description="Capital letter to start each sentence."
          isSelected={cleanup.autoCapitalize}
          onChange={(value) => settings.updateCleanup({ autoCapitalize: value })}
        />
        <SwitchCard
          title="Spoken punctuation"
          description="Say “new line” or “comma”."
          isSelected={cleanup.spokenPunctuation}
          onChange={(value) => settings.updateCleanup({ spokenPunctuation: value })}
        />
        <SwitchCard
          title="Load engine at startup"
          description="First dictation starts instantly."
          isSelected={draft.warmOnStartup}
          onChange={(value) => settings.update({ warmOnStartup: value })}
        />
      </div>
    </Card>
  )
}
