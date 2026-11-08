/**
 * Hotkey vocabulary shared by the settings UI and the main process.
 *
 * A hotkey is either a chord ("Control+Alt+Space", registered as an OS shortcut)
 * or a single key on its own ("RightAlt", watched through a keyboard hook).
 * The Fn key is not offered: laptop firmware handles it and Windows never sees it.
 */

/** Single-key hotkeys: spec name -> label shown to the user. */
export const SINGLE_KEY_LABELS: Readonly<Record<string, string>> = {
  RightAlt: 'Right Alt',
  RightControl: 'Right Ctrl',
  RightShift: 'Right Shift',
  CapsLock: 'Caps Lock',
  ScrollLock: 'Scroll Lock',
  F13: 'F13', F14: 'F14', F15: 'F15', F16: 'F16', F17: 'F17', F18: 'F18',
  F19: 'F19', F20: 'F20', F21: 'F21', F22: 'F22', F23: 'F23', F24: 'F24',
}

/** KeyboardEvent.code values that map to a single-key hotkey. */
const CODE_TO_SINGLE_KEY: Readonly<Record<string, string>> = {
  AltRight: 'RightAlt',
  ControlRight: 'RightControl',
  ShiftRight: 'RightShift',
  CapsLock: 'CapsLock',
  ScrollLock: 'ScrollLock',
  F13: 'F13', F14: 'F14', F15: 'F15', F16: 'F16', F17: 'F17', F18: 'F18',
  F19: 'F19', F20: 'F20', F21: 'F21', F22: 'F22', F23: 'F23', F24: 'F24',
}

export function isSingleKeyHotkey(spec: string): boolean {
  return Object.hasOwn(SINGLE_KEY_LABELS, spec)
}

/** Map a pressed key to a single-key hotkey name, or null if it is not one of them. */
export function singleKeyFromCode(code: string): string | null {
  return Object.hasOwn(CODE_TO_SINGLE_KEY, code) ? (CODE_TO_SINGLE_KEY[code] ?? null) : null
}
