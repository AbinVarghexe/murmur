import { RiCommandLine } from '@remixicon/react'
import { Chip } from '@/components/base/badges/chip'
import { cx } from '@/utils/cx'
import { Card } from './Card'

interface CommandGroup {
  readonly name: string
  readonly phrases: readonly string[]
  readonly note?: string
  /** Span both columns, for groups with long example phrases. */
  readonly wide?: boolean
}

const GROUPS: readonly CommandGroup[] = [
  { name: 'Open', phrases: ['open notepad', 'open github dot com', 'go to youtube.com'] },
  { name: 'Search', phrases: ['search for best pizza near me', 'search youtube for lo-fi beats', 'directions to the airport'] },
  { name: 'Sound and media', phrases: ['volume up', 'mute', 'pause', 'next track'] },
  { name: 'Keys and text', phrases: ['press control shift t', 'select all', 'copy that', 'type hello world'] },
  {
    name: 'Messages',
    phrases: ['send an email to sam at example dot com about lunch saying noon works', 'send a whatsapp to 5551234567 saying running late'],
    note: 'Opens a draft for you to review. Nothing is sent automatically.',
    wide: true,
  },
  { name: 'Windows', phrases: ['minimize window', 'maximize window', 'show desktop', 'close window'], note: 'Closing asks first.' },
]

export function CommandsCard() {
  return (
    <Card
      title="Voice commands"
      description="Start the voice-command hotkey and say any of these, in your own words."
      icon={RiCommandLine}
    >
      <div className="grid gap-5 sm:grid-cols-2">
        {GROUPS.map((group) => (
          <div key={group.name} className={cx('flex flex-col gap-2', group.wide && 'sm:col-span-2')}>
            <h3 className="text-caption-1-semibold uppercase tracking-wide text-text-tertiary">{group.name}</h3>
            <ul className="flex flex-wrap gap-1.5">
              {group.phrases.map((phrase) => (
                <li key={phrase}>
                  <Chip color="soft" variant="subtle">
                    {phrase}
                  </Chip>
                </li>
              ))}
            </ul>
            {group.note ? <p className="text-body-regular text-text-secondary">{group.note}</p> : null}
          </div>
        ))}
      </div>
    </Card>
  )
}
