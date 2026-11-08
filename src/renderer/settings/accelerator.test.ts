import { describe, expect, it } from 'vitest'
import { acceleratorFromKeys, acceleratorLabels } from './accelerator'

const press = (code: string, mods: Partial<{ ctrlKey: boolean; altKey: boolean; shiftKey: boolean }> = {}) => ({
  code, ctrlKey: false, altKey: false, shiftKey: false, ...mods,
})

describe('acceleratorFromKeys', () => {
  it('builds an accelerator from modifiers plus a key', () => {
    expect(acceleratorFromKeys(press('Space', { ctrlKey: true, altKey: true }))).toBe('Control+Alt+Space')
    expect(acceleratorFromKeys(press('KeyD', { ctrlKey: true, shiftKey: true }))).toBe('Control+Shift+D')
    expect(acceleratorFromKeys(press('F9', { altKey: true }))).toBe('Alt+F9')
  })

  it('accepts single keys on their own', () => {
    expect(acceleratorFromKeys(press('AltRight', { altKey: true }))).toBe('RightAlt')
    expect(acceleratorFromKeys(press('ControlRight', { ctrlKey: true }))).toBe('RightControl')
    expect(acceleratorFromKeys(press('CapsLock'))).toBe('CapsLock')
    expect(acceleratorFromKeys(press('F13'))).toBe('F13')
  })

  it('does not treat the left Alt or Ctrl keys as single-key hotkeys', () => {
    expect(acceleratorFromKeys(press('AltLeft', { altKey: true }))).toBeNull()
    expect(acceleratorFromKeys(press('ControlLeft', { ctrlKey: true }))).toBeNull()
  })

  it('rejects a key without a modifier', () => {
    expect(acceleratorFromKeys(press('KeyA'))).toBeNull()
  })

  it('rejects unsupported keys and bare modifiers', () => {
    expect(acceleratorFromKeys(press('ControlLeft', { ctrlKey: true }))).toBeNull()
    expect(acceleratorFromKeys(press('Semicolon', { ctrlKey: true }))).toBeNull()
  })
})

describe('acceleratorLabels', () => {
  it('uses friendly names', () => {
    expect(acceleratorLabels('Control+Alt+Return')).toEqual(['Ctrl', 'Alt', 'Enter'])
  })

  it('labels single keys as one key', () => {
    expect(acceleratorLabels('RightAlt')).toEqual(['Right Alt'])
    expect(acceleratorLabels('CapsLock')).toEqual(['Caps Lock'])
  })
})
