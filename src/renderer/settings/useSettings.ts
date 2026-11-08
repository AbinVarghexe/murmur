import { useCallback, useEffect, useMemo, useState } from 'react'

export interface StatusMessage {
  readonly tone: 'ok' | 'error'
  readonly text: string
}

export interface SettingsController {
  readonly draft: MurmurSettings | null
  readonly dirty: boolean
  readonly saving: boolean
  readonly message: StatusMessage | null
  readonly update: (patch: Partial<MurmurSettings>) => void
  readonly updateCleanup: (patch: Partial<MurmurCleanup>) => void
  readonly save: () => Promise<void>
}

/** Loads settings, tracks unsaved edits, and saves through the main process. */
export function useSettings(): SettingsController {
  const [saved, setSaved] = useState<MurmurSettings | null>(null)
  const [draft, setDraft] = useState<MurmurSettings | null>(null)
  const [saving, setSaving] = useState(false)
  const [message, setMessage] = useState<StatusMessage | null>(null)

  useEffect(() => {
    let alive = true
    void window.murmur.getSettings().then((settings) => {
      if (!alive || !settings) return
      setSaved(settings)
      setDraft(settings)
    })
    return () => {
      alive = false
    }
  }, [])

  const dirty = useMemo(() => saved !== null && draft !== null && JSON.stringify(saved) !== JSON.stringify(draft), [saved, draft])

  const update = useCallback((patch: Partial<MurmurSettings>) => {
    setMessage(null)
    setDraft((current) => (current ? { ...current, ...patch } : current))
  }, [])

  const updateCleanup = useCallback((patch: Partial<MurmurCleanup>) => {
    setMessage(null)
    setDraft((current) => (current ? { ...current, cleanup: { ...current.cleanup, ...patch } } : current))
  }, [])

  const save = useCallback(async () => {
    if (!draft) return
    setSaving(true)
    setMessage(null)
    try {
      const result = await window.murmur.saveSettings(draft)
      if (result.ok) {
        setSaved(draft)
        setMessage({ tone: 'ok', text: 'Saved' })
      } else {
        setMessage({ tone: 'error', text: result.error })
      }
    } catch {
      setMessage({ tone: 'error', text: 'Could not save settings.' })
    } finally {
      setSaving(false)
    }
  }, [draft])

  return { draft, dirty, saving, message, update, updateCleanup, save }
}
