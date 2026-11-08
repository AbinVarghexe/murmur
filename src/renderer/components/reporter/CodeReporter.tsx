import { useState } from 'react'
import {
  RiCommandLine,
  RiFileCopyLine,
  RiCheckLine,
  RiShieldCheckLine,
  RiAlertLine,
  RiPlayCircleLine,
  RiCodeSSlashLine,
  RiTerminalBoxLine,
} from '@remixicon/react'

export interface ParsedCommandAst {
  intent: string
  actionCategory: 'Open' | 'Search' | 'Media' | 'Keys' | 'Messages' | 'Windows' | 'Custom'
  entity: string
  disruptive: boolean
  confidence: number
  generatedCode: {
    rustSnippet: string
    powershellSnippet: string
    virtualKeyCode?: string
  }
}

export function parseCommandToAst(phrase: string): ParsedCommandAst {
  const p = phrase.trim().toLowerCase()

  if (p.includes('close window') || p.includes('quit') || p.includes('exit')) {
    return {
      intent: 'WindowClose',
      actionCategory: 'Windows',
      entity: 'ActiveWindow',
      disruptive: true,
      confidence: 0.98,
      generatedCode: {
        rustSnippet: `// Rust Planner: Disruptive Window Termination\nmurmur_core::planner::Plan::Confirm {\n    summary: "Close the active window".to_string(),\n    action: Box::new(murmur_core::planner::Plan::SendKeys(vec![\n        Key::Alt,\n        Key::F4,\n    ])),\n}`,
        powershellSnippet: `Add-Type @"\n  using System;\n  using System.Runtime.InteropServices;\n  public class Win {\n    [DllImport("user32.dll")]\n    public static extern IntPtr GetForegroundWindow();\n    [DllImport("user32.dll")]\n    public static extern bool PostMessage(IntPtr hWnd, uint Msg, int wParam, int lParam);\n  }\n"@\n$hwnd = [Win]::GetForegroundWindow()\n[Win]::PostMessage($hwnd, 0x0010, 0, 0) # WM_CLOSE`,
        virtualKeyCode: 'VK_MENU + VK_F4',
      },
    }
  }

  if (p.startsWith('open ') || p.startsWith('go to ')) {
    const target = p.replace(/^(open|go to)\s+/, '')
    const isUrl = target.includes('.com') || target.includes('.org') || target.includes('http') || target.includes('github')
    return {
      intent: isUrl ? 'OpenUrl' : 'LaunchApplication',
      actionCategory: 'Open',
      entity: target,
      disruptive: false,
      confidence: 0.95,
      generatedCode: {
        rustSnippet: `// Rust Planner: Open Target\nmurmur_core::urls::open_or_search("${target}")\n    .map_err(|e| format!("Failed to open target: {e}"))?;`,
        powershellSnippet: isUrl
          ? `Start-Process "https://${target.replace(/^https?:\/\//, '')}"`
          : `Start-Process -FilePath "${target}"`,
      },
    }
  }

  if (p.startsWith('search ')) {
    const query = p.replace(/^search\s+(for\s+)?/, '')
    return {
      intent: 'WebSearch',
      actionCategory: 'Search',
      entity: query,
      disruptive: false,
      confidence: 0.94,
      generatedCode: {
        rustSnippet: `// Rust Planner: Web Search with URI encoding\nlet query = murmur_core::urls::encode_search_query("${query}");\nopen::that(format!("https://www.google.com/search?q={query}"))?;`,
        powershellSnippet: `Start-Process "https://www.google.com/search?q=${encodeURIComponent(query)}"`,
      },
    }
  }

  if (p.includes('volume') || p.includes('mute') || p.includes('pause') || p.includes('next track')) {
    return {
      intent: 'MediaControl',
      actionCategory: 'Media',
      entity: p,
      disruptive: false,
      confidence: 0.99,
      generatedCode: {
        rustSnippet: `// Rust Planner: Multimedia Key Event\nmurmur_core::keys::simulate_media_key(murmur_core::keys::MediaAction::${
          p.includes('mute')
            ? 'ToggleMute'
            : p.includes('up')
              ? 'VolumeUp'
              : p.includes('pause')
                ? 'PlayPause'
                : 'NextTrack'
        });`,
        powershellSnippet: `$wscript = New-Object -ComObject Wscript.Shell\n$wscript.SendKeys([char]174) # Media Key Dispatch`,
        virtualKeyCode: p.includes('up') ? 'VK_VOLUME_UP' : p.includes('down') ? 'VK_VOLUME_DOWN' : 'VK_MEDIA_PLAY_PAUSE',
      },
    }
  }

  if (p.startsWith('press ') || p.includes('select all') || p.includes('copy that')) {
    return {
      intent: 'SendShortcut',
      actionCategory: 'Keys',
      entity: p,
      disruptive: false,
      confidence: 0.96,
      generatedCode: {
        rustSnippet: `// Rust WinAPI: Synthesize Keyboard Event\nlet chord = murmur_core::keys::parse_chord("${p}")?;\nmurmur_core::keys::send_input_chord(&chord)?;`,
        powershellSnippet: `$wshell = New-Object -ComObject wscript.shell\n$wshell.SendKeys("^c")`,
        virtualKeyCode: 'VK_CONTROL + VK_KEY_C',
      },
    }
  }

  // Default to General Dictation / Freeform Voice Typing
  return {
    intent: 'TypeInputText',
    actionCategory: 'Custom',
    entity: phrase,
    disruptive: false,
    confidence: 0.91,
    generatedCode: {
      rustSnippet: `// Rust Core: Send Input String via enigo / SendInput\nlet formatted_text = murmur_core::format::format_transcript(\n    "${phrase}",\n    format_options,\n);\nmurmur_core::keys::type_string(&formatted_text);`,
      powershellSnippet: `Add-Type -AssemblyName System.Windows.Forms\n[System.Windows.Forms.SendKeys]::SendWait("${phrase}")`,
    },
  }
}

interface CodeReporterProps {
  initialPhrase?: string
}

export function CodeReporter({ initialPhrase = 'open github.com' }: CodeReporterProps) {
  const [phrase, setPhrase] = useState(initialPhrase)
  const [activeTab, setActiveTab] = useState<'rust' | 'powershell' | 'ast'>('rust')
  const [copied, setCopied] = useState(false)
  const [simulatedStatus, setSimulatedStatus] = useState<string | null>(null)

  const ast = parseCommandToAst(phrase)

  const copyCode = (code: string) => {
    void navigator.clipboard.writeText(code)
    setCopied(true)
    setTimeout(() => setCopied(false), 1800)
  }

  const simulateExecution = () => {
    setSimulatedStatus('Dispatching command to local OS pipeline…')
    setTimeout(() => {
      setSimulatedStatus(
        ast.disruptive
          ? 'Requires Confirmation: Prompted ApprovalCard.'
          : 'Executed successfully with 0 errors.',
      )
    }, 450)
  }

  return (
    <div className="flex flex-col gap-4 rounded-2xl border border-line-soft bg-surface p-5 shadow-sm">
      {/* Header */}
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-line-soft pb-4">
        <div className="flex items-center gap-2.5">
          <span className="grid size-9 place-items-center rounded-xl bg-field text-ink">
            <RiCodeSSlashLine className="size-5" />
          </span>
          <div>
            <h3 className="text-[14px] font-semibold text-ink">Code Reporter & Command AST</h3>
            <p className="text-[12px] text-ink-3">Live AST breakdown and synthesized system code for voice input</p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {ast.disruptive ? (
            <span className="inline-flex items-center gap-1 rounded-full bg-orange-tint px-2.5 py-1 text-[11px] font-medium text-orange">
              <RiAlertLine className="size-3.5" />
              Confirmation Required
            </span>
          ) : (
            <span className="inline-flex items-center gap-1 rounded-full bg-green-tint px-2.5 py-1 text-[11px] font-medium text-green">
              <RiShieldCheckLine className="size-3.5" />
              Safe Autonomous Exec
            </span>
          )}

          <div className="rounded-full bg-field px-2.5 py-1 font-mono text-[11px] text-ink-2">
            Match: {Math.round(ast.confidence * 100)}%
          </div>
        </div>
      </div>

      {/* Input tester */}
      <div className="flex flex-col gap-2">
        <label htmlFor="code-reporter-input" className="text-[12px] font-medium text-ink-2">
          Test Command or Spoken Phrase
        </label>
        <div className="flex items-center gap-2">
          <div className="relative flex-1">
            <RiCommandLine className="pointer-events-none absolute start-3 top-1/2 size-4 -translate-y-1/2 text-ink-3" />
            <input
              id="code-reporter-input"
              type="text"
              value={phrase}
              onChange={(e) => setPhrase(e.target.value)}
              placeholder="e.g. open notepad, volume up, press control c"
              className="w-full rounded-xl border border-line bg-inset py-2 pe-3 ps-9 font-mono text-[13px] text-ink outline-none transition-all focus:border-accent focus:ring-2 focus:ring-accent-tint"
            />
          </div>

          <button
            type="button"
            onClick={simulateExecution}
            className="flex items-center gap-1.5 rounded-xl bg-accent px-3.5 py-2 text-[12px] font-medium text-white shadow-sm transition-all hover:opacity-90 active:scale-[0.98]"
          >
            <RiPlayCircleLine className="size-4" />
            <span>Simulate</span>
          </button>
        </div>

        {/* Quick sample chips */}
        <div className="flex flex-wrap items-center gap-1.5 pt-1">
          <span className="text-[11px] text-ink-3">Presets:</span>
          {['open github.com', 'close window', 'search best pizza near me', 'volume up', 'press control shift t'].map(
            (p) => (
              <button
                key={p}
                type="button"
                onClick={() => setPhrase(p)}
                className={`rounded-md px-2 py-0.5 text-[11px] transition-colors ${
                  phrase === p ? 'bg-accent-tint font-medium text-accent-ink' : 'bg-field text-ink-2 hover:bg-hover'
                }`}
              >
                {p}
              </button>
            ),
          )}
        </div>
      </div>

      {/* AST Metadata Grid */}
      <div className="grid grid-cols-2 gap-2 rounded-xl border border-line-soft bg-inset p-3 sm:grid-cols-4">
        <div>
          <span className="text-[10px] uppercase tracking-wider text-ink-3">Intent Node</span>
          <p className="font-mono text-[12px] font-semibold text-ink">{ast.intent}</p>
        </div>
        <div>
          <span className="text-[10px] uppercase tracking-wider text-ink-3">Action Category</span>
          <p className="font-mono text-[12px] font-medium text-ink-2">{ast.actionCategory}</p>
        </div>
        <div>
          <span className="text-[10px] uppercase tracking-wider text-ink-3">Target Entity</span>
          <p className="truncate font-mono text-[12px] font-medium text-ink-2">{ast.entity}</p>
        </div>
        <div>
          <span className="text-[10px] uppercase tracking-wider text-ink-3">Virtual Key Chord</span>
          <p className="font-mono text-[12px] font-medium text-accent-ink">{ast.generatedCode.virtualKeyCode || 'N/A'}</p>
        </div>
      </div>

      {/* Code Viewer */}
      <div className="overflow-hidden rounded-xl border border-line-soft bg-canvas">
        <div className="flex items-center justify-between border-b border-line-soft bg-inset px-3 py-1.5">
          <div className="flex items-center gap-1">
            <button
              type="button"
              onClick={() => setActiveTab('rust')}
              className={`flex items-center gap-1.5 rounded-lg px-2.5 py-1 text-[12px] font-medium transition-colors ${
                activeTab === 'rust' ? 'bg-surface text-ink shadow-xs' : 'text-ink-3 hover:text-ink'
              }`}
            >
              <RiCodeSSlashLine className="size-3.5" />
              Rust (murmur-core)
            </button>
            <button
              type="button"
              onClick={() => setActiveTab('powershell')}
              className={`flex items-center gap-1.5 rounded-lg px-2.5 py-1 text-[12px] font-medium transition-colors ${
                activeTab === 'powershell' ? 'bg-surface text-ink shadow-xs' : 'text-ink-3 hover:text-ink'
              }`}
            >
              <RiTerminalBoxLine className="size-3.5" />
              PowerShell / Win32
            </button>
            <button
              type="button"
              onClick={() => setActiveTab('ast')}
              className={`flex items-center gap-1.5 rounded-lg px-2.5 py-1 text-[12px] font-medium transition-colors ${
                activeTab === 'ast' ? 'bg-surface text-ink shadow-xs' : 'text-ink-3 hover:text-ink'
              }`}
            >
              JSON AST
            </button>
          </div>

          <button
            type="button"
            onClick={() => {
              const code =
                activeTab === 'rust'
                  ? ast.generatedCode.rustSnippet
                  : activeTab === 'powershell'
                    ? ast.generatedCode.powershellSnippet
                    : JSON.stringify(ast, null, 2)
              copyCode(code)
            }}
            className="flex items-center gap-1 rounded-md px-2 py-1 text-[11px] text-ink-3 hover:bg-hover hover:text-ink"
          >
            {copied ? <RiCheckLine className="size-3.5 text-green" /> : <RiFileCopyLine className="size-3.5" />}
            <span>{copied ? 'Copied' : 'Copy'}</span>
          </button>
        </div>

        <pre className="overflow-x-auto p-3.5 font-mono text-[12px] leading-relaxed text-ink">
          <code>
            {activeTab === 'rust' && ast.generatedCode.rustSnippet}
            {activeTab === 'powershell' && ast.generatedCode.powershellSnippet}
            {activeTab === 'ast' && JSON.stringify(ast, null, 2)}
          </code>
        </pre>
      </div>

      {simulatedStatus ? (
        <div className="flex items-center gap-2 rounded-xl bg-field px-3 py-2 text-[12px] text-ink-2">
          <span className="size-2 rounded-full bg-accent animate-pulse" />
          <span>{simulatedStatus}</span>
        </div>
      ) : null}
    </div>
  )
}
