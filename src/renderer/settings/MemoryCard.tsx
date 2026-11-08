import { useCallback, useEffect, useState, type FormEvent } from 'react'
import {
  RiAddLine,
  RiBrainLine,
  RiCheckLine,
  RiDeleteBinLine,
  RiEditLine,
  RiHistoryLine,
  RiMicFill,
  RiRefreshLine,
  RiSparkling2Line,
  RiStopFill,
} from '@remixicon/react'
import { Button } from '@/components/base/buttons/button'
import { Chip } from '@/components/base/badges/chip'
import { Input } from '@/components/base/input/input'
import { Card } from './Card'

type MemoryKind = VoiceMemoryEntry['kind']

const EMPTY_MEMORY: MurmurVoiceMemory = {
  schemaVersion: 2,
  entries: [],
  corrections: [],
  recentTranscripts: [],
  learningExamples: [],
}

const KIND_LABELS: Record<MemoryKind, string> = {
  person: 'Person',
  vocabulary: 'Vocabulary',
  phrase: 'Phrase',
}

function confidenceLabel(value: number): string {
  if (value >= 0.9) return 'High'
  if (value >= 0.75) return 'Medium'
  return 'Learning'
}

export function MemoryCard() {
  const [memory, setMemory] = useState<MurmurVoiceMemory>(EMPTY_MEMORY)
  const [loading, setLoading] = useState(true)
  const [status, setStatus] = useState<string | null>(null)
  const [canonical, setCanonical] = useState('')
  const [aliases, setAliases] = useState('')
  const [kind, setKind] = useState<MemoryKind>('person')
  const [editingId, setEditingId] = useState<string | undefined>()
  const [original, setOriginal] = useState('')
  const [corrected, setCorrected] = useState('')
  const [context, setContext] = useState('')
  const [trainingStatus, setTrainingStatus] = useState<VoiceTrainingEvent['status'] | 'idle'>('idle')
  const [trainingMessage, setTrainingMessage] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    setLoading(true)
    try {
      setMemory((await window.murmur.getVoiceMemory()) ?? EMPTY_MEMORY)
    } catch {
      setStatus('Could not load voice memory.')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void refresh()
  }, [refresh])

  useEffect(() => window.murmur.onVoiceTraining((event) => {
    setTrainingStatus(event.status)
    setTrainingMessage(event.message ?? null)
    if (event.status === 'completed') {
      setOriginal(event.personalized || event.raw || '')
      void refresh()
    }
  }), [refresh])

  useEffect(() => window.murmur.onLearningUpdate(() => {
    void refresh()
  }), [refresh])

  const toggleVoiceTraining = async () => {
    if (trainingStatus === 'processing') return
    if (trainingStatus === 'recording') {
      try {
        await window.murmur.stopVoiceTraining()
      } catch (error) {
        setTrainingStatus('error')
        setTrainingMessage(error instanceof Error ? error.message : 'Could not stop the teaching recording.')
      }
      return
    }

    setTrainingMessage(null)
    try {
      await window.murmur.startVoiceTraining()
      setTrainingStatus('recording')
    } catch (error) {
      setTrainingStatus('error')
      setTrainingMessage(error instanceof Error ? error.message : 'Could not start the teaching recording.')
    }
  }

  const saveEntry = async (event: FormEvent) => {
    event.preventDefault()
    const cleanCanonical = canonical.trim()
    if (!cleanCanonical) return
    try {
      await window.murmur.addVoiceMemoryEntry({
        canonical: cleanCanonical,
        aliases: aliases.split(',').map((item) => item.trim()).filter(Boolean),
        kind,
        id: editingId,
      })
      setCanonical('')
      setAliases('')
      setEditingId(undefined)
      setStatus(editingId ? 'Memory entry updated.' : 'Memory entry added.')
      await refresh()
    } catch {
      setStatus('Could not save this memory entry.')
    }
  }

  const learnCorrection = async (event: FormEvent) => {
    event.preventDefault()
    if (!original.trim() || !corrected.trim()) return
    try {
      await window.murmur.learnVoiceCorrection(original.trim(), corrected.trim(), context.trim())
      setOriginal('')
      setCorrected('')
      setContext('')
      setStatus('Correction learned locally. Exact matches are active immediately; fuzzy variants build confidence over time.')
      await refresh()
    } catch {
      setStatus('Could not learn this correction.')
    }
  }

  const editEntry = (entry: VoiceMemoryEntry) => {
    setEditingId(entry.id)
    setCanonical(entry.canonical)
    setAliases(entry.aliases.join(', '))
    setKind(entry.kind)
  }

  const deleteEntry = async (id: string) => {
    await window.murmur.deleteVoiceMemoryEntry(id)
    if (editingId === id) {
      setEditingId(undefined)
      setCanonical('')
      setAliases('')
    }
    await refresh()
  }

  const deleteCorrection = async (id: string) => {
    await window.murmur.deleteVoiceCorrection(id)
    await refresh()
  }

  const clearMemory = async () => {
    if (!window.confirm('Clear all learned voice memory from this device?')) return
    await window.murmur.clearVoiceMemory()
    setStatus('Voice memory cleared from this device.')
    await refresh()
  }

  return (
    <div className="flex flex-col gap-6">
      <Card
        title="Personal voice memory"
        description="Murmur adapts locally around the ASR model. Explicit terms apply immediately; uncertain fuzzy variants require stronger evidence."
        icon={RiBrainLine}
      >
        <div className="flex flex-wrap items-center justify-between gap-3 rounded-2xl bg-background-secondary-default p-4">
          <div className="flex items-center gap-2 text-body-regular text-text-secondary">
            <RiSparkling2Line className="size-4 text-foreground-icon-secondary" aria-hidden />
            <span>
              {memory.entries.length} entries · {memory.corrections.length} learned corrections · {memory.learningExamples.length} learning examples
            </span>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="secondary" size="small" leadingIcon={RiRefreshLine} onClick={() => void refresh()}>
              Refresh
            </Button>
            <Button variant="danger" size="small" leadingIcon={RiDeleteBinLine} onClick={() => void clearMemory()}>
              Clear memory
            </Button>
          </div>
        </div>
        {status ? <p className="text-body-regular text-text-secondary" role="status">{status}</p> : null}
      </Card>

      <div className="grid gap-6 lg:grid-cols-2">
        <Card
          title={editingId ? 'Edit memory entry' : 'Add a known term'}
          description="Add a canonical name or term with the ways Parakeet commonly hears it."
          icon={RiAddLine}
        >
          <form className="flex flex-col gap-3" onSubmit={saveEntry}>
            <Input
              label="Canonical term"
              value={canonical}
              onChange={setCanonical}
              placeholder="Abin Varghese"
            />
            <Input
              label="Aliases"
              hint="Separate variants with commas."
              value={aliases}
              onChange={setAliases}
              placeholder="Eben Vargis, Abin Vargis"
            />
            <div className="flex flex-wrap gap-2" aria-label="Memory category">
              {(Object.keys(KIND_LABELS) as MemoryKind[]).map((option) => (
                <Button
                  key={option}
                  type="button"
                  size="small"
                  variant={kind === option ? 'primary' : 'secondary'}
                  onClick={() => setKind(option)}
                >
                  {KIND_LABELS[option]}
                </Button>
              ))}
            </div>
            <div className="flex flex-wrap gap-2">
              <Button leadingIcon={editingId ? RiCheckLine : RiAddLine} type="submit">
                {editingId ? 'Save entry' : 'Add entry'}
              </Button>
              {editingId ? (
                <Button
                  variant="secondary"
                  type="button"
                  onClick={() => {
                    setEditingId(undefined)
                    setCanonical('')
                    setAliases('')
                  }}
                >
                  Cancel edit
                </Button>
              ) : null}
            </div>
          </form>
        </Card>

        <Card
          title="Teach a correction"
          description="Speak an example, review what Murmur wrote, then enter what you meant."
          icon={RiSparkling2Line}
        >
          <div className="flex flex-col gap-3 rounded-2xl bg-background-secondary-default p-4">
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div className="min-w-0">
                <p className="text-body-medium text-text-primary">Teach by voice</p>
                <p className="text-body-regular text-text-secondary">
                  Record one sentence containing the name, word, or phrase Murmur gets wrong. Audio stays on this device.
                </p>
              </div>
              <Button
                leadingIcon={trainingStatus === 'recording' ? RiStopFill : RiMicFill}
                variant={trainingStatus === 'recording' ? 'danger' : 'primary'}
                disabled={trainingStatus === 'processing'}
                onClick={() => void toggleVoiceTraining()}
              >
                {trainingStatus === 'recording'
                  ? 'Stop & transcribe'
                  : trainingStatus === 'processing'
                    ? 'Transcribing…'
                    : 'Record my voice'}
              </Button>
            </div>
            {trainingMessage ? (
              <p className="text-caption-1-regular text-text-secondary" role="status">
                {trainingMessage}
              </p>
            ) : null}
            <p className="text-caption-1-regular text-text-tertiary">
              This teaches local voice memory around the ASR model; it does not change the model weights.
            </p>
          </div>
          <form className="flex flex-col gap-3" onSubmit={learnCorrection}>
            <Input
              label="What Murmur wrote"
              value={original}
              onChange={setOriginal}
              placeholder="Speak above to fill this automatically"
            />
            <Input
              label="What you meant"
              value={corrected}
              onChange={setCorrected}
              placeholder="I sent it to Abin Varghese"
            />
            <Input
              label="Context (optional)"
              value={context}
              onChange={setContext}
              placeholder="Send the project report"
            />
            <Button leadingIcon={RiSparkling2Line} type="submit">
              Teach this correction
            </Button>
          </form>
        </Card>
      </div>

      <Card title="Known terms" description="Entries are stored in the local Murmur data directory." icon={RiBrainLine}>
        {loading ? (
          <p className="text-body-regular text-text-secondary">Loading voice memory…</p>
        ) : memory.entries.length === 0 ? (
          <p className="text-body-regular text-text-secondary">No personal terms have been learned yet.</p>
        ) : (
          <div className="flex flex-col gap-2">
            {memory.entries.map((entry) => (
              <div key={entry.id} className="flex flex-wrap items-center justify-between gap-3 rounded-2xl bg-background-secondary-default p-4">
                <div className="min-w-0">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="text-body-medium text-text-primary">{entry.canonical}</span>
                    <Chip color="soft" variant="caption">{KIND_LABELS[entry.kind]}</Chip>
                    <Chip color={entry.confidence >= 0.8 ? 'lime' : 'yellow'} variant="caption">
                      {confidenceLabel(entry.confidence)} · {Math.round(entry.confidence * 100)}%
                    </Chip>
                  </div>
                  <p className="mt-1 text-body-regular text-text-secondary">
                    {entry.aliases.length > 0 ? `Aliases: ${entry.aliases.join(', ')}` : 'No aliases'} · {entry.frequency} uses
                  </p>
                </div>
                <div className="flex shrink-0 gap-2">
                  <Button variant="secondary" size="small" leadingIcon={RiEditLine} onClick={() => editEntry(entry)}>
                    Edit
                  </Button>
                  <Button variant="danger" size="small" leadingIcon={RiDeleteBinLine} onClick={() => void deleteEntry(entry.id)}>
                    Delete
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}
      </Card>

      <Card title="Learned corrections" description="Conflicting corrections are retained and ranked by confidence, frequency, and context." icon={RiHistoryLine}>
        {memory.corrections.length === 0 ? (
          <p className="text-body-regular text-text-secondary">No corrections have been recorded yet.</p>
        ) : (
          <div className="flex flex-col gap-2">
            {memory.corrections.map((correction) => (
              <div key={correction.id} className="flex flex-wrap items-center justify-between gap-3 rounded-2xl bg-background-secondary-default p-4">
                <div className="min-w-0 text-body-regular">
                  <p className="text-text-secondary">
                    <span className="text-text-tertiary">“{correction.original}”</span>{' '}
                    <span aria-hidden>→</span>{' '}
                    <span className="text-text-primary">“{correction.corrected}”</span>
                  </p>
                  <p className="mt-1 text-caption-1-regular text-text-tertiary">
                    {correction.frequency} confirmations · {correction.successes} accepted · {correction.failures} edited again · {Math.round(correction.confidence * 100)}% confidence
                    {correction.context ? ` · ${correction.context}` : ''}
                  </p>
                  {correction.promoted ? (
                    <Chip color="lime" variant="caption">Promoted personal rule</Chip>
                  ) : (
                    <Chip color="soft" variant="caption">Learning reliability</Chip>
                  )}
                </div>
                <Button variant="danger" size="small" leadingIcon={RiDeleteBinLine} onClick={() => void deleteCorrection(correction.id)}>
                  Delete
                </Button>
              </div>
            ))}
          </div>
        )}
      </Card>

      <Card title="Recent transcript history" description="Use history to teach the exact output that the production voice pipeline produced." icon={RiHistoryLine}>
        {memory.recentTranscripts.length === 0 ? (
          <p className="text-body-regular text-text-secondary">Your production dictations will appear here after the first recording.</p>
        ) : (
          <div className="flex flex-col gap-2">
            {memory.recentTranscripts.slice(0, 10).map((record) => (
              <div key={record.id} className="flex flex-wrap items-center justify-between gap-3 rounded-2xl bg-background-secondary-default p-4">
                <div className="min-w-0">
                  <div className="mb-1 flex flex-wrap items-center gap-2">
                    <Chip color="soft" variant="caption">{record.mode || 'voice'}</Chip>
                    <span className="text-caption-1-regular text-text-tertiary">{record.timestamp}</span>
                  </div>
                  <p className="truncate text-body-regular text-text-primary">{record.personalized || record.raw}</p>
                  {record.personalized !== record.raw ? <p className="truncate text-caption-1-regular text-text-tertiary">Raw: {record.raw}</p> : null}
                </div>
                <Button
                  variant="secondary"
                  size="small"
                  leadingIcon={RiSparkling2Line}
                  onClick={() => {
                    setOriginal(record.personalized || record.raw)
                    setCorrected(record.corrected ?? '')
                    window.scrollTo({ top: 0, behavior: 'smooth' })
                  }}
                >
                  Teach this
                </Button>
              </div>
            ))}
          </div>
        )}
      </Card>

      <Card title="Automatic learning activity" description="Murmur keeps the original audio and compares the text it inserted with later edits in the focused editor." icon={RiBrainLine}>
        {memory.learningExamples.length === 0 ? (
          <p className="text-body-regular text-text-secondary">Automatic examples will appear after you dictate into a supported text editor.</p>
        ) : (
          <div className="flex flex-col gap-2">
            {memory.learningExamples.slice(0, 10).map((example) => (
              <div key={example.id} className="flex flex-wrap items-center justify-between gap-3 rounded-2xl bg-background-secondary-default p-4">
                <div className="min-w-0">
                  <div className="mb-1 flex flex-wrap items-center gap-2">
                    <Chip color={example.status === 'learned' || example.status === 'unchanged' ? 'lime' : example.status === 'edited' || example.status === 'failed' ? 'yellow' : 'soft'} variant="caption">
                      {example.status}
                    </Chip>
                    <span className="text-caption-1-regular text-text-tertiary">
                      {example.successes} accepted · {example.failures} failed
                    </span>
                  </div>
                  <p className="truncate text-body-regular text-text-primary">Inserted: {example.inserted}</p>
                  {example.corrected ? <p className="truncate text-caption-1-regular text-text-secondary">Edited to: {example.corrected}</p> : null}
                </div>
              </div>
            ))}
          </div>
        )}
      </Card>
    </div>
  )
}
