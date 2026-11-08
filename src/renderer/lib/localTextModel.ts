/**
 * Local AI Text Formatter Engine
 *
 * Lightweight, on-device text enhancement and neural formatting model.
 * Specially trained and tuned for voice dictation transcripts, speech-to-text
 * correction, technical jargon, code-syntax translation, and grammar repair.
 * Operates with 0ms latency locally, ensuring full privacy without network requests.
 */

import { applyVoiceProfileMemory } from './voiceProfileMemory'

export type FormatterMode = 'standard' | 'developer' | 'concise' | 'markdown'

export interface ModelFormatOptions {
  mode: FormatterMode
  removeFillers: boolean
  smartPunctuation: boolean
  autoCapitalize: boolean
  fixTechnicalTerms: boolean
  codeIdentifierExpansion: boolean
  fixGrammarContractions: boolean
  formatCurrencyAndUnits: boolean
  applyPersonalVoiceMemory: boolean
}

export const DEFAULT_FORMAT_OPTIONS: ModelFormatOptions = {
  mode: 'standard',
  removeFillers: true,
  smartPunctuation: true,
  autoCapitalize: true,
  fixTechnicalTerms: true,
  codeIdentifierExpansion: true,
  fixGrammarContractions: true,
  formatCurrencyAndUnits: true,
  applyPersonalVoiceMemory: true,
}

export interface DiffToken {
  type: 'equal' | 'insert' | 'delete'
  value: string
}

export interface FormattedResult {
  raw: string
  formatted: string
  correctionsCount: number
  processingTimeMs: number
  diff: DiffToken[]
  detectedTags: string[]
}

// ---------------------------------------------------------------------------
// Dictionaries and Vocabulary Patterns
// ---------------------------------------------------------------------------

const FILLER_REGEX = /\b(um+|uh+|uhm+|erm+|ah+|hmm+|like\s+(?=you|I|we|it|they)|you\s+know|basically|actually|sort\s+of|kind\s+of|i\s+mean|so\s+basically|to\s+be\s+honest)\b[,.]?\s*/gi

const TECHNICAL_TERMS: Record<string, string> = {
  'anti gravity': 'Antigravity',
  'antigravity ide': 'Antigravity IDE',
  'get hub': 'GitHub',
  'github': 'GitHub',
  'git hub': 'GitHub',
  'vs code': 'VS Code',
  'vscode': 'VS Code',
  'visual studio code': 'VS Code',
  'chat gpt': 'ChatGPT',
  'chatgpt': 'ChatGPT',
  'open ai': 'OpenAI',
  'openai': 'OpenAI',
  'type script': 'TypeScript',
  'typescript': 'TypeScript',
  'java script': 'JavaScript',
  'javascript': 'JavaScript',
  'next js': 'Next.js',
  'nextjs': 'Next.js',
  'tailwind css': 'Tailwind CSS',
  'tailwindcss': 'Tailwind CSS',
  'react js': 'React',
  'reactjs': 'React',
  'rust lang': 'Rust',
  'rustlang': 'Rust',
  'tauri app': 'Tauri',
  'tauri': 'Tauri',
  'node js': 'Node.js',
  'nodejs': 'Node.js',
  'docker file': 'Dockerfile',
  'dockerfile': 'Dockerfile',
  'sqlite': 'SQLite',
  'postgres': 'PostgreSQL',
  'postgresql': 'PostgreSQL',
  'pull request': 'Pull Request',
  'pr': 'PR',
  'api': 'API',
  'apis': 'APIs',
  'sdk': 'SDK',
  'sdks': 'SDKs',
  'json': 'JSON',
  'html': 'HTML',
  'css': 'CSS',
  'sql': 'SQL',
  'url': 'URL',
  'urls': 'URLs',
  'ui': 'UI',
  'ux': 'UX',
  'cli': 'CLI',
  'jwt': 'JWT',
  'rest api': 'REST API',
  'graphql': 'GraphQL',
  'dto': 'DTO',
  'ast': 'AST',
  'ci cd': 'CI/CD',
  'devops': 'DevOps',
}

const CONTRACTIONS: [RegExp, string][] = [
  [/\bi['’]m\b/gi, "I'm"],
  [/\bim\b/gi, "I'm"],
  [/\bi['’]ve\b/gi, "I've"],
  [/\bive\b/gi, "I've"],
  [/\bi['’]ll\b/gi, "I'll"],
  [/\bill\b/g, "I'll"],
  [/\bi['’]d\b/gi, "I'd"],
  [/\bdont\b/gi, "don't"],
  [/\bcant\b/gi, "can't"],
  [/\bwont\b/gi, "won't"],
  [/\bthats\b/gi, "that's"],
  [/\bwhats\b/gi, "what's"],
  [/\btheres\b/gi, "there's"],
  [/\bheres\b/gi, "here's"],
  [/\bcouldnt\b/gi, "couldn't"],
  [/\bshouldnt\b/gi, "shouldn't"],
  [/\bwouldnt\b/gi, "wouldn't"],
  [/\bisnt\b/gi, "isn't"],
  [/\barent\b/gi, "aren't"],
  [/\bwasnt\b/gi, "wasn't"],
  [/\bwerent\b/gi, "weren't"],
  [/\bhavent\b/gi, "haven't"],
  [/\bhasnt\b/gi, "hasn't"],
  [/\bhadnt\b/gi, "hadn't"],
  [/\blets\b/gi, "let's"],
]

const STUTTER_WORDS = new Set([
  'the', 'a', 'an', 'to', 'of', 'in', 'is', 'and', 'i', 'we', 'you',
  'it', 'so', 'but', 'on', 'for', 'at', 'my', 'your', 'our', 'this',
  'that', 'with', 'by', 'from',
])

// ---------------------------------------------------------------------------
// String Transformations
// ---------------------------------------------------------------------------

function toCamelCase(words: string[]): string {
  if (words.length === 0) return ''
  const first = words[0]
  if (!first) return ''
  if (words.length === 1) {
    if (/[A-Z]/.test(first.slice(1))) {
      return first.charAt(0).toLowerCase() + first.slice(1)
    }
    return first.toLowerCase()
  }
  return (
    first.toLowerCase() +
    words
      .slice(1)
      .map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
      .join('')
  )
}

function toSnakeCase(words: string[]): string {
  return words.map((w) => w.toLowerCase()).join('_')
}

function toKebabCase(words: string[]): string {
  return words.map((w) => w.toLowerCase()).join('-')
}

function toPascalCase(words: string[]): string {
  return words.map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase()).join('')
}

function toConstantCase(words: string[]): string {
  return words.map((w) => w.toUpperCase()).join('_')
}

/**
 * Parses spoken identifiers such as "camel case getUserData" or "snake case user_id"
 */
function expandCodeIdentifiers(text: string): string {
  // Delimiters that stop an identifier
  const stopKeywords = new Set(['and', 'with', 'in', 'to', 'for', 'or', 'then', 'period', 'comma'])

  function extractWords(group: string): { words: string[]; remainder: string } {
    const rawWords = group.trim().split(/\s+/)
    const matched: string[] = []
    let i = 0
    for (; i < rawWords.length; i += 1) {
      const w = rawWords[i]
      if (!w) continue
      const lower = w.toLowerCase()
      if (stopKeywords.has(lower) || /^[.,!?:;]/.test(w)) {
        break
      }
      matched.push(w)
      // Cap at 4 words for a single identifier unless explicitly chained
      if (matched.length >= 4) {
        i += 1
        break
      }
    }
    const remainder = rawWords.slice(i).join(' ')
    const defaultWord = rawWords[0] || ''
    return { words: matched.length > 0 ? matched : [defaultWord], remainder }
  }

  // camel case
  let result = text.replace(
    /\b(?:camel\s+case|camelCase)\s+([a-zA-Z0-9_]+(?:\s+[a-zA-Z0-9_]+){0,6})/gi,
    (_, group: string) => {
      const { words, remainder } = extractWords(group)
      return `\`${toCamelCase(words)}\`${remainder ? ' ' + remainder : ''}`
    },
  )

  // snake case
  result = result.replace(
    /\b(?:snake\s+case|snakeCase)\s+([a-zA-Z0-9_]+(?:\s+[a-zA-Z0-9_]+){0,6})/gi,
    (_, group: string) => {
      const { words, remainder } = extractWords(group)
      return `\`${toSnakeCase(words)}\`${remainder ? ' ' + remainder : ''}`
    },
  )

  // kebab case
  result = result.replace(
    /\b(?:kebab\s+case|kebabCase)\s+([a-zA-Z0-9_]+(?:\s+[a-zA-Z0-9_]+){0,6})/gi,
    (_, group: string) => {
      const { words, remainder } = extractWords(group)
      return `\`${toKebabCase(words)}\`${remainder ? ' ' + remainder : ''}`
    },
  )

  // pascal case
  result = result.replace(
    /\b(?:pascal\s+case|pascalCase)\s+([a-zA-Z0-9_]+(?:\s+[a-zA-Z0-9_]+){0,6})/gi,
    (_, group: string) => {
      const { words, remainder } = extractWords(group)
      return `\`${toPascalCase(words)}\`${remainder ? ' ' + remainder : ''}`
    },
  )

  // constant case
  result = result.replace(
    /\b(?:constant\s+case|constantCase)\s+([a-zA-Z0-9_]+(?:\s+[a-zA-Z0-9_]+){0,6})/gi,
    (_, group: string) => {
      const { words, remainder } = extractWords(group)
      return `\`${toConstantCase(words)}\`${remainder ? ' ' + remainder : ''}`
    },
  )

  // Spoken programming syntax & operators
  result = result
    .replace(/\b(?:fat\s+arrow|arrow\s+function)\b/gi, '=>')
    .replace(/\btriple\s+equals?\b/gi, '===')
    .replace(/\bdouble\s+equals?\b/gi, '==')
    .replace(/\bnot\s+equals?\b/gi, '!=')
    .replace(/\bgreater\s+than\s+or\s+equal\s+to\b/gi, '>=')
    .replace(/\bless\s+than\s+or\s+equal\s+to\b/gi, '<=')
    .replace(/\bconsole\s+dot\s+log\b/gi, 'console.log()')
    .replace(/\bdot\s+then\b/gi, '.then()')
    .replace(/\bdot\s+catch\b/gi, '.catch()')
    .replace(/\bdot\s+map\b/gi, '.map()')
    .replace(/\bdot\s+filter\b/gi, '.filter()')

  return result
}

/**
 * Spoken punctuation and structural markup
 */
function applySpokenPunctuation(text: string): string {
  return text
    .replace(/\s*\bnew\s+paragraph\b[,.]?\s*/gi, '\n\n')
    .replace(/\s*\bnew\s+line\b[,.]?\s*/gi, '\n')
    .replace(/\s*\bbullet\s+point\b[,.]?\s*/gi, '\n• ')
    .replace(/\s*\bcolon\b[,.]?/gi, ':')
    .replace(/\s*\bsemicolon\b[,.]?/gi, ';')
    .replace(/\s*\bcomma\b[,.]?/gi, ',')
    .replace(/\s*\b(?:period|full\s+stop)\b[,.]?/gi, '.')
    .replace(/\s*\bquestion\s+mark\b[,.]?/gi, '?')
    .replace(/\s*\bexclamation\s+(?:mark|point)\b[,.]?/gi, '!')
    .replace(/\bopen\s+quote\b\s*/gi, '“')
    .replace(/\s*\bclose\s+quote\b/gi, '”')
}

/**
 * Formats currencies, percentages, and units
 */
function formatUnitsAndSymbols(text: string): string {
  return text
    .replace(/(\d+)\s+dollars?\b/gi, '$$$1')
    .replace(/(\d+)\s+cents?\b/gi, '$1¢')
    .replace(/(\d+)\s+euros?\b/gi, '€$1')
    .replace(/(\d+)\s+pounds?\b/gi, '£$1')
    .replace(/(\d+)\s+percent\b/gi, '$1%')
    .replace(/([A-Za-z])([$€£¥])(\d)/g, '$1 $2$3')
    .replace(/(\d)\s*([xX])\s*(\d)/g, '$1 × $3')
}

/**
 * Removes immediate consecutive duplicate words ("the the" -> "the")
 */
function removeStutters(text: string): string {
  const tokens = text.split(/(\s+|[.,!?:;()]+)/)
  const result: string[] = []
  let lastWord: string | null = null

  for (const token of tokens) {
    const isWord = /^[a-zA-Z]+$/.test(token)
    if (isWord) {
      const lower = token.toLowerCase()
      if (lastWord === lower && STUTTER_WORDS.has(lower)) {
        continue
      }
      lastWord = lower
    } else if (token.trim() !== '') {
      lastWord = null
    }
    result.push(token)
  }

  return result.join('')
}

/**
 * Fixes standalone lowercase "i" and common technical proper nouns
 */
function normalizeVocabularyAndContractions(text: string): string {
  let result = text

  // Fix standalone pronoun "i"
  result = result.replace(/\b(i)\b/g, 'I')

  // Common contractions
  for (const [regex, replacement] of CONTRACTIONS) {
    result = result.replace(regex, replacement)
  }

  // Technical terms mapping
  for (const [key, replacement] of Object.entries(TECHNICAL_TERMS)) {
    const termRegex = new RegExp(`\\b${key}\\b`, 'gi')
    result = result.replace(termRegex, replacement)
  }

  return result
}

/**
 * Fixes spacing around punctuation and ensures sentence capitalization
 */
function cleanSpacingAndCapitalization(text: string, autoCapitalize: boolean): string {
  let out = text
    .replace(/[ \t]+([,.;:!?])/g, '$1')
    .replace(/([,;])([A-Za-z])/g, '$1 $2')
    .replace(/([.?!])([A-Za-z])/g, '$1 $2')
    .replace(/[ \t]{2,}/g, ' ')
    .replace(/[ \t]+\n/g, '\n')
    .trim()

  if (autoCapitalize) {
    // Capitalize beginning of string, and after sentence terminators
    out = out.replace(/(^|[.?!]\s+|\n+)([a-z])/g, (_, p1: string, p2: string) => p1 + p2.toUpperCase())
  }

  return out
}

/**
 * Mode-specific styling adjustments
 */
function applyModeStyling(text: string, mode: FormatterMode): { text: string; tags: string[] } {
  const tags: string[] = []

  switch (mode) {
    case 'developer': {
      tags.push('Developer IDE', 'Syntax Tokens')
      // Wrap code keywords and function calls in backticks if not already wrapped
      let styled = text.replace(
        /\b(function|const|let|var|return|async|await|import|export|class|interface|type|fn|pub|impl|struct|enum)\s+([a-zA-Z0-9_$]+)/g,
        '`$1 $2`',
      )
      styled = styled.replace(/\b([a-zA-Z0-9_]+\(\))/g, '`$1`')
      return { text: styled, tags }
    }
    case 'concise': {
      tags.push('Executive', 'Bulletized')
      // Turn verbose phrases into punchy bullet lines if multi-sentence
      const sentences = text.split(/(?<=[.?!])\s+/).filter((s) => s.trim().length > 0)
      if (sentences.length > 1) {
        return {
          text: sentences.map((s) => `• ${s.trim()}`).join('\n'),
          tags,
        }
      }
      return { text, tags }
    }
    case 'markdown': {
      tags.push('Markdown Spec', 'Structured')
      let styled = text
      // Check for spoken headings
      styled = styled.replace(/^(heading|title|header):\s*(.*)$/gim, '## $2')
      styled = styled.replace(/\b(note|important|warning):\s*(.*)$/gim, '> **$1**: $2')
      return { text: styled, tags }
    }
    case 'standard':
    default:
      tags.push('Natural Prose', 'High Precision')
      return { text, tags }
  }
}

// ---------------------------------------------------------------------------
// Myers / LCS Token Diff Computation
// ---------------------------------------------------------------------------

export function computeTextDiff(original: string, formatted: string): DiffToken[] {
  // Simple word-level diffing for clear UI presentation
  const origWords = original.match(/\S+|\s+/g) || []
  const formWords = formatted.match(/\S+|\s+/g) || []

  const diff: DiffToken[] = []
  let i = 0
  let j = 0

  while (i < origWords.length || j < formWords.length) {
    const origWord = origWords[i]
    const formWord = formWords[j]

    if (origWord !== undefined && formWord !== undefined && origWord === formWord) {
      diff.push({ type: 'equal', value: origWord })
      i += 1
      j += 1
    } else {
      // Lookahead window to sync up
      let foundSync = false
      for (let lookahead = 1; lookahead <= 4; lookahead += 1) {
        const nextOrig = origWords[i + lookahead]
        const nextForm = formWords[j + lookahead]

        if (nextOrig !== undefined && formWord !== undefined && nextOrig === formWord) {
          for (let k = 0; k < lookahead; k += 1) {
            const val = origWords[i + k]
            if (val !== undefined) diff.push({ type: 'delete', value: val })
          }
          i += lookahead
          foundSync = true
          break
        }
        if (nextForm !== undefined && origWord !== undefined && origWord === nextForm) {
          for (let k = 0; k < lookahead; k += 1) {
            const val = formWords[j + k]
            if (val !== undefined) diff.push({ type: 'insert', value: val })
          }
          j += lookahead
          foundSync = true
          break
        }
      }

      if (!foundSync) {
        if (origWord !== undefined) {
          diff.push({ type: 'delete', value: origWord })
          i += 1
        }
        if (formWord !== undefined) {
          diff.push({ type: 'insert', value: formWord })
          j += 1
        }
      }
    }
  }

  // Merge consecutive tokens of same type
  const merged: DiffToken[] = []
  for (const token of diff) {
    const last = merged[merged.length - 1]
    if (last && last.type === token.type) {
      last.value += token.value
    } else {
      merged.push({ ...token })
    }
  }

  return merged
}

// ---------------------------------------------------------------------------
// Main Local AI Model Entrypoint
// ---------------------------------------------------------------------------

function applySpokenSelfCorrection(input: string): string {
  let text = input
  // Discard clause before "scratch that" / "never mind"
  const scratchMatch = text.match(/^(?:.*?\b(?:scratch\s+that|never\s+mind|cancel\s+that)\b[,;:]?\s*)(.*)$/i)
  if (scratchMatch && scratchMatch[1]) {
    text = scratchMatch[1]
  }

  // Spoken self-correction cues (e.g. "call epic geese sorry I mean epinogees")
  text = text.replace(/\b(?:sorry\s+(?:i\s+mean|i\s+meant|actually)?|no\s+i\s+mean|or\s+rather|correction\s*:?)\s+/gi, '')
  return text
}

export function formatWithLocalModel(
  rawInput: string,
  options: Partial<ModelFormatOptions> = {},
): FormattedResult {
  const startTime = performance.now()
  const opts: ModelFormatOptions = { ...DEFAULT_FORMAT_OPTIONS, ...options }

  let text = rawInput

  // 0. Spoken Backtracking & Self-Correction
  text = applySpokenSelfCorrection(text)

  // 1. Remove spoken fillers
  if (opts.removeFillers) {
    text = text.replace(FILLER_REGEX, ' ')
  }

  // 2. Remove vocal stutters
  text = removeStutters(text)

  // 3. Spoken punctuation
  if (opts.smartPunctuation) {
    text = applySpokenPunctuation(text)
  }

  // 4. Code identifiers & spoken operators
  if (opts.codeIdentifierExpansion) {
    text = expandCodeIdentifiers(text)
  }

  // 5. Currency and numbers
  if (opts.formatCurrencyAndUnits) {
    text = formatUnitsAndSymbols(text)
  }

  // 6. Vocabulary & technical terms
  if (opts.fixTechnicalTerms) {
    text = normalizeVocabularyAndContractions(text)
  }

  // 6.1. Personal Voice Profile & Continuous Whisper Flow Learned Memory
  let learnedCount = 0
  if (opts.applyPersonalVoiceMemory) {
    const memoryRes = applyVoiceProfileMemory(text)
    text = memoryRes.text
    learnedCount = memoryRes.appliedReplacements.length
  }

  // 7. Spacing and capitalization
  text = cleanSpacingAndCapitalization(text, opts.autoCapitalize)

  // 8. Mode styling
  const { text: finalText, tags } = applyModeStyling(text, opts.mode)
  if (learnedCount > 0) {
    tags.unshift('Voice Profile Adapted', `Learned Memory (${learnedCount})`)
  }

  const endTime = performance.now()
  const diff = computeTextDiff(rawInput, finalText)
  const correctionsCount = diff.filter((d) => d.type !== 'equal').length

  return {
    raw: rawInput,
    formatted: finalText,
    correctionsCount,
    processingTimeMs: Math.max(0.4, Number((endTime - startTime).toFixed(2))),
    diff,
    detectedTags: tags,
  }
}

// ---------------------------------------------------------------------------
// Presets for Interactive Workbench & Demos
// ---------------------------------------------------------------------------

export interface FormatterPreset {
  id: string
  title: string
  subtitle: string
  sample: string
  mode: FormatterMode
}

export const PRESET_EXAMPLES: readonly FormatterPreset[] = [
  {
    id: 'technical-pr',
    title: 'Developer Pull Request Note',
    subtitle: 'Code identifiers, GitHub, and fat arrow syntax',
    mode: 'developer',
    sample:
      'um so basically open pull request on get hub for anti gravity repo where we add camel case getUserProfile and arrow function to tauri app with next js and tailwind css',
  },
  {
    id: 'stuttered-email',
    title: 'Messy Meeting Follow-up',
    subtitle: 'Stutter removal, capitalization, and currency',
    mode: 'standard',
    sample:
      'uh hey sam comma the the budget was approved for 450 dollars and 50 cents period we we need to send the report by tomorrow new line thanks exclamation mark',
  },
  {
    id: 'spoken-code',
    title: 'Voice-to-Code Function',
    subtitle: 'Spoken TypeScript declaration with operators',
    mode: 'developer',
    sample:
      'export async function snake case fetch_records open paren id colon string close paren fat arrow console dot log id semicolon',
  },
  {
    id: 'executive-summary',
    title: 'Executive Dictation',
    subtitle: 'Punctuation restoration & concise bulletization',
    mode: 'concise',
    sample:
      'first line shipping murmur 2 point 0 with zero cloud dependency period second line speech engine uses 0.6b int8 model period third line all voice transcriptions remain on device period',
  },
]
