import { useState } from 'react'
import {
  RiMicFill,
  RiCheckLine,
  RiMagicLine,
  RiCodeSSlashLine,
  RiCpuLine,
  RiKeyboardBoxLine,
  RiCommandLine,
  RiSparkling2Line,
  RiShieldCheckLine,
  RiPulseLine,
  RiApps2Line,
  RiBrainLine,
} from '@remixicon/react'
import { Chip } from '@/components/base/badges/chip'
import { Button } from '@/components/base/buttons/button'
import { ThemeToggle } from '@/components/application/theme/theme-toggle'
import { VoiceStudio } from '@/components/studio/VoiceStudio'
import { TextFormatterCard } from '@/components/formatter/TextFormatterCard'
import { CodeReporter } from '@/components/reporter/CodeReporter'
import { CustomAppsCard } from './CustomAppsCard'
import { EngineCard } from './EngineCard'
import { HotkeysCard } from './HotkeysCard'
import { CleanupCard } from './CleanupCard'
import { CommandsCard } from './CommandsCard'
import { MemoryCard } from './MemoryCard'
import { useModel } from './useModel'
import { useSettings } from './useSettings'

type ActiveTab = 'studio' | 'voiceport' | 'formatter' | 'reporter' | 'apps' | 'engine' | 'hotkeys' | 'commands'

export function App() {
  const [activeTab, setActiveTab] = useState<ActiveTab>('studio')
  const model = useModel()
  const settings = useSettings()

  const tabs: { id: ActiveTab; label: string; icon: typeof RiMicFill; badge?: string }[] = [
    { id: 'studio', label: 'Voice Studio', icon: RiMicFill, badge: 'Preview' },
    { id: 'voiceport', label: 'Voice Memory', icon: RiBrainLine, badge: 'Local' },
    { id: 'apps', label: 'App Launchers', icon: RiApps2Line, badge: 'ChatGPT' },
    { id: 'formatter', label: 'AI Formatter', icon: RiMagicLine, badge: 'Neural' },
    { id: 'reporter', label: 'Code Reporter', icon: RiCodeSSlashLine },
    { id: 'engine', label: 'Speech Engine', icon: RiCpuLine },
    { id: 'hotkeys', label: 'Hotkeys', icon: RiKeyboardBoxLine },
    { id: 'commands', label: 'Command Reference', icon: RiCommandLine },
  ]

  return (
    <div className="min-h-screen bg-canvas text-ink antialiased">
      {/* IDE Top Navigation Bar */}
      <header className="sticky top-0 z-40 border-b border-line bg-surface/90 backdrop-blur-md">
        <div className="mx-auto flex max-w-6xl items-center justify-between gap-4 px-4 py-2.5 sm:px-6">
          {/* Brand & Workspace Info */}
          <div className="flex items-center gap-3">
            <span className="relative grid size-9 place-items-center rounded-xl bg-accent text-white shadow-md shadow-accent/20">
              <RiMicFill className="size-5" />
              <span className="absolute -bottom-0.5 -end-0.5 size-2.5 rounded-full bg-green ring-2 ring-surface" />
            </span>
            <div>
              <div className="flex items-center gap-2">
                <span className="text-[15px] font-bold tracking-tight text-ink">Murmur</span>
                <span className="rounded-md border border-accent/30 bg-accent-tint/40 px-1.5 py-0.5 font-mono text-[10px] font-semibold text-accent-ink">
                  WhisperFlow · v2.0
                </span>
                <span className="hidden items-center gap-1 rounded-md border border-line-soft bg-field px-1.5 py-0.5 font-mono text-[10px] text-ink-3 md:inline-flex">
                  <RiBrainLine className="size-3 text-accent" />
                  Personal Lexicon Active
                </span>
              </div>
              <p className="hidden text-[11px] text-ink-3 sm:block">On-Device Continuous Voice Learning & Dictation IDE</p>
            </div>
          </div>

          {/* Center Tabs Navigation */}
          <nav className="flex items-center gap-1 overflow-x-auto rounded-xl border border-line-soft bg-inset p-1">
            {tabs.map((tab) => {
              const Icon = tab.icon
              const isSelected = activeTab === tab.id
              return (
                <button
                  key={tab.id}
                  type="button"
                  onClick={() => setActiveTab(tab.id)}
                  className={`flex items-center gap-1.5 whitespace-nowrap rounded-lg px-2.5 py-1.5 text-[12px] font-medium transition-all ${
                    isSelected
                      ? 'bg-surface text-ink shadow-xs ring-1 ring-line-strong'
                      : 'text-ink-2 hover:bg-hover hover:text-ink'
                  }`}
                >
                  <Icon className={`size-3.5 ${isSelected ? 'text-accent' : 'text-ink-3'}`} />
                  <span>{tab.label}</span>
                  {tab.badge ? (
                    <span
                      className={`rounded-full px-1.5 py-0.2 text-[9px] font-semibold uppercase tracking-wider ${
                        isSelected ? 'bg-accent-tint text-accent-ink' : 'bg-field text-ink-3'
                      }`}
                    >
                      {tab.badge}
                    </span>
                  ) : null}
                </button>
              )
            })}
          </nav>

          {/* Right Tools */}
          <div className="flex items-center gap-2.5">
            <ThemeToggle appearance="segmented" size="small" />
          </div>
        </div>
      </header>

      {/* Main Workspace Body */}
      <main className="mx-auto max-w-6xl px-4 pb-28 pt-6 sm:px-6">
        {/* Tab 1: Voice Studio & Live VO Interaction */}
        {activeTab === 'studio' && (
          <div className="flex flex-col gap-6">
            <VoiceStudio />
          </div>
        )}

        {/* Tab 2: Personal Voice Port & Whisper Flow Learning */}
        {activeTab === 'voiceport' && (
          <div className="flex flex-col gap-6">
            <MemoryCard />
          </div>
        )}

        {/* Tab 3: Custom Application Launchers & Typo Workarounds */}
        {activeTab === 'apps' && (
          <div className="flex flex-col gap-6">
            <CustomAppsCard />
          </div>
        )}

        {/* Tab 4: Local AI Text Formatter */}
        {activeTab === 'formatter' && (
          <div className="flex flex-col gap-6">
            <TextFormatterCard />
          </div>
        )}

        {/* Tab 3: Code Reporter & AST Inspector */}
        {activeTab === 'reporter' && (
          <div className="flex flex-col gap-6">
            <CodeReporter />
          </div>
        )}

        {/* Tab 4: Engine & Offline Weights */}
        {activeTab === 'engine' && (
          <div className="flex flex-col gap-6">
            <EngineCard model={model} />
            <div className="grid gap-5 md:grid-cols-2">
              <div className="rounded-2xl border border-line-soft bg-surface p-5 shadow-sm">
                <h3 className="mb-2 text-[14px] font-semibold text-ink">On-Device Guarantees</h3>
                <ul className="flex flex-col gap-2 text-[12px] text-ink-2">
                  <li className="flex items-center gap-2">
                    <RiShieldCheckLine className="size-4 text-green" />
                    Audio never traverses the network or leaves RAM.
                  </li>
                  <li className="flex items-center gap-2">
                    <RiShieldCheckLine className="size-4 text-green" />
                    Local Sherpa-ONNX runtime with INT8 quantization.
                  </li>
                  <li className="flex items-center gap-2">
                    <RiShieldCheckLine className="size-4 text-green" />
                    Works 100% offline with zero cloud dependency.
                  </li>
                </ul>
              </div>

              <div className="rounded-2xl border border-line-soft bg-surface p-5 shadow-sm">
                <h3 className="mb-2 text-[14px] font-semibold text-ink">Acoustic Specs</h3>
                <div className="grid grid-cols-2 gap-3 font-mono text-[12px]">
                  <div>
                    <span className="text-[10px] uppercase text-ink-3">Sampling Rate</span>
                    <p className="font-semibold text-ink">16,000 Hz Mono</p>
                  </div>
                  <div>
                    <span className="text-[10px] uppercase text-ink-3">Latency Floor</span>
                    <p className="font-semibold text-accent-ink">&lt; 150 ms</p>
                  </div>
                  <div>
                    <span className="text-[10px] uppercase text-ink-3">Model Size</span>
                    <p className="font-semibold text-ink">660 MB weights</p>
                  </div>
                  <div>
                    <span className="text-[10px] uppercase text-ink-3">Inference Target</span>
                    <p className="font-semibold text-ink">CPU / WinML Direct</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Tab 5: Hotkeys & Trigger Settings */}
        {activeTab === 'hotkeys' && (
          <div className="flex flex-col gap-6">
            <div className="grid items-start gap-6 md:grid-cols-2">
              <HotkeysCard settings={settings} />
              <CleanupCard settings={settings} />
            </div>
          </div>
        )}

        {/* Tab 6: Voice Command Catalog */}
        {activeTab === 'commands' && (
          <div className="flex flex-col gap-6">
            <CommandsCard />
          </div>
        )}
      </main>

      {/* IDE Bottom Status Bar */}
      <footer className="fixed inset-x-0 bottom-0 z-40 border-t border-line bg-surface/95 px-4 py-2 backdrop-blur-md">
        <div className="mx-auto flex max-w-6xl items-center justify-between gap-4">
          {/* Status Telemetry */}
          <div className="flex items-center gap-3 text-[11px] text-ink-3">
            <div className="flex items-center gap-1.5">
              <span className={`size-2 rounded-full ${model.ready ? 'bg-green' : 'bg-orange animate-pulse'}`} />
              <span className="font-mono text-ink-2">
                {model.ready ? 'Parakeet TDT 0.6B (Active)' : 'Engine Not Loaded'}
              </span>
            </div>
            <span className="hidden sm:inline">|</span>
            <span className="hidden font-mono sm:inline">Local AI Formatter: Online</span>
            <span className="hidden sm:inline">|</span>
            <span className="hidden sm:inline">Zero Cloud Leaks</span>
          </div>

          {/* Dirty changes & Save button */}
          <div className="flex items-center gap-3">
            <div className="min-h-5" role="status" aria-live="polite">
              {settings.message ? (
                <Chip color={settings.message.tone === 'ok' ? 'lime' : 'rose'} variant="subtle">
                  {settings.message.text}
                </Chip>
              ) : settings.dirty ? (
                <span className="text-[12px] text-orange">Unsaved changes pending</span>
              ) : null}
            </div>

            <Button
              leadingIcon={RiCheckLine}
              disabled={!settings.dirty || settings.saving}
              onClick={() => void settings.save()}
            >
              {settings.saving ? 'Saving…' : 'Save Changes'}
            </Button>
          </div>
        </div>
      </footer>
    </div>
  )
}
