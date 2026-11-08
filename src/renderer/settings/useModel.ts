import { useCallback, useEffect, useState } from 'react'

export interface ModelController {
  readonly ready: boolean
  readonly loaded: boolean
  readonly downloading: boolean
  readonly receivedBytes: number
  readonly totalBytes: number
  readonly error: string
  readonly download: () => Promise<void>
}

const DOWNLOAD_FAILED = 'Download failed. Check your connection and try again.'

/** Tracks the speech model: installed or not, and live download progress. */
export function useModel(): ModelController {
  const [status, setStatus] = useState<{ ready: boolean; totalBytes: number; downloading: boolean } | null>(null)
  const [receivedBytes, setReceivedBytes] = useState(0)
  const [error, setError] = useState('')

  const refresh = useCallback(async () => {
    setStatus(await window.murmur.getModelStatus())
  }, [])

  useEffect(() => {
    void refresh()
    return window.murmur.onModelProgress((progress) => {
      setReceivedBytes(progress.receivedBytes)
      setStatus((current) => (current ? { ...current, totalBytes: progress.totalBytes, downloading: true } : current))
    })
  }, [refresh])

  const download = useCallback(async () => {
    setError('')
    setReceivedBytes(0)
    setStatus((current) => (current ? { ...current, downloading: true } : current))
    try {
      const result = await window.murmur.downloadModel()
      if (!result.ok) setError(result.error ?? DOWNLOAD_FAILED)
    } catch {
      setError(DOWNLOAD_FAILED)
    }
    await refresh()
  }, [refresh])

  return {
    ready: status?.ready ?? false,
    loaded: status !== null,
    downloading: status?.downloading ?? false,
    receivedBytes,
    totalBytes: status?.totalBytes ?? 0,
    error,
    download,
  }
}
