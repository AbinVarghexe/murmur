import { useState, useEffect } from 'react'
import {
  RiApps2Line,
  RiAddLine,
  RiDeleteBinLine,
  RiPlayFill,
  RiShieldCheckLine,
  RiSparkling2Line,
  RiCheckLine,
  RiInformationLine,
} from '@remixicon/react'
import { Button } from '@/components/base/buttons/button'
import { Chip } from '@/components/base/badges/chip'

export function CustomAppsCard() {
  const [apps, setApps] = useState<CustomAppItem[]>([])
  const [loading, setLoading] = useState(true)
  const [name, setName] = useState('')
  const [aliases, setAliases] = useState('')
  const [target, setTarget] = useState('')
  const [statusMessage, setStatusMessage] = useState<string | null>(null)

  useEffect(() => {
    void (async () => {
      try {
        const loaded = await window.murmur.getCustomApps()
        setApps(loaded)
      } catch (e) {
        console.error('Failed to load custom apps:', e)
      } finally {
        setLoading(false)
      }
    })()
  }, [])

  const handleAddApp = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!name.trim() || !target.trim()) return

    const parsedAliases = aliases
      .split(',')
      .map((a) => a.trim().toLowerCase())
      .filter(Boolean)

    if (!parsedAliases.includes(name.trim().toLowerCase())) {
      parsedAliases.push(name.trim().toLowerCase())
    }

    const newApp: CustomAppItem = {
      id: `app-${Date.now()}`,
      name: name.trim(),
      aliases: parsedAliases,
      target: target.trim(),
    }

    const updated = [...apps, newApp]
    setApps(updated)
    setName('')
    setAliases('')
    setTarget('')

    try {
      await window.murmur.saveCustomApps(updated)
      setStatusMessage(`Added "${newApp.name}" successfully!`)
      setTimeout(() => setStatusMessage(null), 3000)
    } catch (err) {
      console.error(err)
    }
  }

  const handleDelete = async (id: string) => {
    const updated = apps.filter((a) => a.id !== id)
    setApps(updated)
    try {
      await window.murmur.saveCustomApps(updated)
    } catch (err) {
      console.error(err)
    }
  }

  const handleTestLaunch = async (targetCmd: string) => {
    try {
      await window.murmur.launchCustomApp(targetCmd)
      setStatusMessage(`Launched: ${targetCmd}`)
      setTimeout(() => setStatusMessage(null), 2500)
    } catch (err: any) {
      setStatusMessage(`Error launching: ${err}`)
      setTimeout(() => setStatusMessage(null), 3500)
    }
  }

  return (
    <div className="flex flex-col gap-6">
      {/* Top Banner */}
      <div className="rounded-2xl border border-line bg-surface p-5 shadow-xs sm:p-6">
        <div className="flex flex-col justify-between gap-4 md:flex-row md:items-center">
          <div className="flex items-start gap-3.5">
            <span className="grid size-12 shrink-0 place-items-center rounded-2xl bg-gradient-to-tr from-orange to-amber-500 text-white shadow-sm">
              <RiApps2Line className="size-6" />
            </span>
            <div>
              <div className="flex flex-wrap items-center gap-2">
                <h2 className="text-[17px] font-bold tracking-tight text-ink">
                  Application Launchers & Typo Workarounds
                </h2>
                <Chip color="cyan" variant="bold">
                  Fuzzy & Typo Tolerant
                </Chip>
              </div>
              <p className="mt-1 text-[13px] text-ink-2 leading-relaxed">
                Register installed apps (such as <strong className="text-ink">ChatGPT</strong>,{' '}
                <strong className="text-ink">Figma</strong>, or <strong className="text-ink">Antigravity</strong>)
                with spoken variations and typos. When you say &quot;open chat gpt&quot;, &quot;launch chatgpt&quot;,
                or even with a typo like &quot;chat gbt&quot;, Murmur accurately resolves and launches your tool.
              </p>
            </div>
          </div>

          {statusMessage && (
            <div className="flex items-center gap-1.5 rounded-xl border border-accent/30 bg-accent-tint/50 px-3 py-1.5 text-[12px] font-medium text-accent-ink animate-pulse">
              <RiSparkling2Line className="size-3.5" />
              <span>{statusMessage}</span>
            </div>
          )}
        </div>
      </div>

      {/* Add Application Form */}
      <div className="rounded-2xl border border-line bg-surface p-5 shadow-xs">
        <h3 className="flex items-center gap-2 text-[14px] font-bold text-ink">
          <RiAddLine className="size-4 text-accent" />
          <span>Add Installed Tool or Custom Launcher</span>
        </h3>
        <p className="mt-1 text-[12px] text-ink-2">
          Specify the app name, spoken aliases (comma-separated), and the executable name, full path, or protocol.
        </p>

        <form onSubmit={handleAddApp} className="mt-4 grid gap-3 sm:grid-cols-3">
          <div>
            <label className="text-[11px] font-semibold uppercase tracking-wider text-ink-3">
              Application Name
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. ChatGPT, Cursor, Slack"
              className="mt-1 w-full rounded-xl border border-line bg-field px-3 py-2 text-[13px] text-ink placeholder:text-ink-3 focus:border-accent focus:outline-none"
            />
          </div>

          <div>
            <label className="text-[11px] font-semibold uppercase tracking-wider text-ink-3">
              Spoken Aliases & Typos
            </label>
            <input
              type="text"
              value={aliases}
              onChange={(e) => setAliases(e.target.value)}
              placeholder="e.g. chat gpt, chatgpt, gpt, chat gbt"
              className="mt-1 w-full rounded-xl border border-line bg-field px-3 py-2 text-[13px] text-ink placeholder:text-ink-3 focus:border-accent focus:outline-none"
            />
          </div>

          <div>
            <label className="text-[11px] font-semibold uppercase tracking-wider text-ink-3">
              Target (Exe, Path, or Protocol)
            </label>
            <div className="mt-1 flex items-center gap-2">
              <input
                type="text"
                value={target}
                onChange={(e) => setTarget(e.target.value)}
                placeholder="e.g. chatgpt-desktop.exe or chatgpt:"
                className="w-full rounded-xl border border-line bg-field px-3 py-2 text-[13px] text-ink placeholder:text-ink-3 focus:border-accent focus:outline-none"
              />
              <Button
                variant="primary"
                size="small"
                leadingIcon={RiAddLine}
                onClick={() => handleAddApp({ preventDefault: () => {} } as any)}
                disabled={!name.trim() || !target.trim()}
              >
                Add
              </Button>
            </div>
          </div>
        </form>
      </div>

      {/* Applications List */}
      <div className="rounded-2xl border border-line bg-surface p-5 shadow-xs">
        <div className="mb-4 flex items-center justify-between border-b border-line-soft pb-3">
          <div>
            <h3 className="text-[14px] font-bold text-ink">Active Application Matchers</h3>
            <span className="text-[11px] text-ink-3">{apps.length} tools registered for voice launch</span>
          </div>
        </div>

        {loading ? (
          <div className="p-4 text-center text-[12px] text-ink-3">Loading application database…</div>
        ) : (
          <div className="space-y-2.5">
            {apps.map((app) => (
              <div
                key={app.id}
                className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-line-soft bg-inset/40 p-3.5 transition-colors hover:border-line-strong hover:bg-inset"
              >
                <div className="flex flex-col gap-1">
                  <div className="flex items-center gap-2">
                    <span className="font-semibold text-[14px] text-ink">{app.name}</span>
                    <span className="font-mono text-[11px] text-ink-3">({app.target})</span>
                  </div>
                  <div className="flex flex-wrap items-center gap-1">
                    <span className="text-[11px] text-ink-3">Spoken:</span>
                    {app.aliases.map((alias) => (
                      <span
                        key={alias}
                        className="rounded-md border border-line-soft bg-field px-1.5 py-0.5 font-mono text-[11px] text-ink-2"
                      >
                        &quot;{alias}&quot;
                      </span>
                    ))}
                  </div>
                </div>

                <div className="flex items-center gap-2">
                  <Button
                    variant="secondary"
                    size="small"
                    leadingIcon={RiPlayFill}
                    onClick={() => handleTestLaunch(app.target)}
                  >
                    Test Launch
                  </Button>
                  <button
                    type="button"
                    onClick={() => handleDelete(app.id)}
                    className="grid size-8 place-items-center rounded-lg text-ink-3 transition-colors hover:bg-red-tint hover:text-red"
                    title="Delete application matcher"
                  >
                    <RiDeleteBinLine className="size-4" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
