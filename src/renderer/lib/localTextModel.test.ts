import { describe, expect, it } from 'vitest'
import { formatWithLocalModel, computeTextDiff } from './localTextModel'
import {
  learnFromCorrection,
  applyVoiceProfileMemory,
  addOrUpdatePersonalWord,
} from './voiceProfileMemory'

describe('localTextModel', () => {
  it('removes fillers and stutters while capitalizing', () => {
    const raw = 'um uh the the report is basically ready'
    const res = formatWithLocalModel(raw)
    expect(res.formatted).toBe('The report is ready')
  })

  it('normalizes technical proper nouns', () => {
    const raw = 'check the pr on get hub for anti gravity with vs code and tailwind css'
    const res = formatWithLocalModel(raw)
    expect(res.formatted).toContain('GitHub')
    expect(res.formatted).toContain('Antigravity')
    expect(res.formatted).toContain('VS Code')
    expect(res.formatted).toContain('Tailwind CSS')
  })

  it('adapts user persona name epinogees and personal tools', () => {
    const raw = 'hello this is epic geese designing components in fig ma with chat gpt'
    const res = formatWithLocalModel(raw)
    expect(res.formatted).toContain('epinogees')
    expect(res.formatted).toContain('Figma')
    expect(res.formatted).toContain('ChatGPT')
    expect(res.detectedTags).toContain('Voice Profile Adapted')
  })

  it('continuously learns new corrections from user edits (Whisper Flow)', () => {
    // Model initially transcribed 'k vo studio'
    const misheard = 'welcome to k vo studio'
    const corrected = 'welcome to Kaevo Studio'

    // User corrects the word
    const learned = learnFromCorrection(misheard, corrected, 'branding update')
    expect(learned.length).toBeGreaterThan(0)

    // Next time the user speaks the misheard phrase, it is automatically corrected
    const nextSpoken = 'we love working at k vo studio'
    const res = formatWithLocalModel(nextSpoken)
    expect(res.formatted).toContain('Kaevo Studio')
  })

  it('transforms spoken code identifiers to camelCase and snake_case', () => {
    const raw = 'define camel case getUserProfile and snake case account_status'
    const res = formatWithLocalModel(raw)
    expect(res.formatted).toContain('`getUserProfile`')
    expect(res.formatted).toContain('`account_status`')
  })

  it('transforms spoken operators', () => {
    const raw = 'param fat arrow triple equals null'
    const res = formatWithLocalModel(raw)
    expect(res.formatted).toContain('=>')
    expect(res.formatted).toContain('===')
  })

  it('injects frequently used author name Abin Varghese instead of random guess', () => {
    const raw = 'this commit was pushed by abin vargis on github'
    const res = formatWithLocalModel(raw)
    expect(res.formatted).toContain('Abin Varghese')
  })

  it('converts spoken punctuation and newlines', () => {
    const raw = 'hello comma world period new line next line'
    const res = formatWithLocalModel(raw)
    expect(res.formatted).toBe('Hello, world.\nNext line')
  })

  it('computes accurate diff tokens', () => {
    const original = 'um hello'
    const formatted = 'Hello.'
    const diff = computeTextDiff(original, formatted)
    expect(diff.some((t) => t.type === 'delete' && t.value.includes('um'))).toBe(true)
    expect(diff.some((t) => t.type === 'insert')).toBe(true)
  })
})
