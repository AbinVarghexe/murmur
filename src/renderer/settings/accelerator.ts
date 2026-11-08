import { isSingleKeyHotkey, SINGLE_KEY_LABELS, singleKeyFromCode } from '../../shared/hotkeys'

const KEY_NAMES: Readonly<Record<string, string>> = {
  Space: 'Space', ArrowUp: 'Up', ArrowDown: 'Down', ArrowLeft: 'Left', ArrowRight: 'Right',
  Enter: 'Return', Tab: 'Tab', Backspace: 'Backspace', Delete: 'Delete', Home: 'Home', End: 'End',
}

const DISPLAY_NAMES: Readonly<Record<string, string>> = { Control: 'Ctrl', Return: 'Enter', Super: 'Win' }

export interface KeyInput {
  readonly code: string
  readonly ctrlKey: boolean
  readonly altKey: boolean
  readonly shiftKey: boolean
}

function keyFromCode(code: string): string | null {
  const named = KEY_NAMES[code]
  if (named) return named
  if (/^Key[A-Z]$/.test(code)) return code.slice(3)
  if (/^Digit[0-9]$/.test(code)) return code.slice(5)
  if (/^F(?:[1-9]|1[0-9]|2[0-4])$/.test(code)) return code
  return null
}

/**
 * Turn a key press into a hotkey: a single key such as "RightAlt", or an Electron
 * accelerator such as "Control+Alt+Space". Returns null unless the press is a
 * supported single key, or a supported key plus at least one modifier.
 */
export function acceleratorFromKeys(input: KeyInput): string | null {
  const single = singleKeyFromCode(input.code)
  if (single) return single
  const key = keyFromCode(input.code)
  if (!key || !(input.ctrlKey || input.altKey || input.shiftKey)) return null
  const parts = [input.ctrlKey ? 'Control' : '', input.altKey ? 'Alt' : '', input.shiftKey ? 'Shift' : '', key]
  return parts.filter(Boolean).join('+')
}

/** Split an accelerator into friendly labels for display, e.g. ["Ctrl", "Alt", "Space"]. */
export function acceleratorLabels(accelerator: string): string[] {
  if (isSingleKeyHotkey(accelerator)) return [SINGLE_KEY_LABELS[accelerator] ?? accelerator]
  return accelerator.split('+').filter(Boolean).map((part) => DISPLAY_NAMES[part] ?? part)
}
