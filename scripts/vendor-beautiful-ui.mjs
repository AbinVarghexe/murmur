// Copies the Beautiful UI components Murmur uses from a downloaded registry folder into the app,
// applying the few changes the app needs. Usage:
//   node scripts/vendor-beautiful-ui.mjs <folder with the registry "files" tree>
//
// Changes made to the library source, and why:
//  - LoadingState: the "Surfer" variant streams a video from a remote server. It is removed so the
//    app never makes a network request and stays fully offline.
//  - foundation.css: the published file ends a block it never opened (a leftover sidebar media-query
//    fragment), which Tailwind rejects. The fragment is removed.
//  - foundation.css: Tailwind is told exactly where to look for classes, so the pill's stylesheet
//    does not also generate every class used by the settings window (a different design system).
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import path from 'node:path'

const source = process.argv[2]
if (!source) throw new Error('Pass the folder that contains components/ and app/')
const dest = path.resolve('src/renderer')

const FILES = [
  ['components/atoms/Button.tsx', 'components/atoms/Button.tsx'],
  ['components/atoms/StreamText.tsx', 'components/atoms/StreamText.tsx'],
  ['components/atoms/ValuePill.tsx', 'components/atoms/ValuePill.tsx'],
  ['components/primitives/LoadingState.tsx', 'components/primitives/LoadingState.tsx'],
  ['components/primitives/ApprovalCard.tsx', 'components/primitives/ApprovalCard.tsx'],
  ['components/primitives/GlideMenu.tsx', 'components/primitives/GlideMenu.tsx'],
  ['app/beautifui/foundation.css', 'styles/beautifui/foundation.css'],
]

function removeSurfer(code) {
  let out = code
  // Drop the video URL default and the Surfer branch.
  out = out.replace(/\/\*\* the meme feed[\s\S]*?\*\/\s*videoSrc = "[^"]*",\s*/m, '')
  out = out.replace(/\s*videoSrc\?: string;/, '')
  out = out.replace(/\s*videoSrc,/, '')
  out = out.replace(/\n\s*const surfer = variant === "Surfer";/, '')
  out = out.replace(/\n\s*const \[videoOk, setVideoOk\] = useState\(true\);/, '')
  out = out.replace(/\(surfer \? "Subway surfing" : "Churning"\)/, '"Churning"')
  out = out.replace(/\n\s*if \(surfer\) \{[\s\S]*?\n  \}\n\n  return \(/, '\n\n  return (')
  out = out.replace(/ \*   Surfer — the Drive loader paired with a meme video below\n/, '')
  return out
}

const DANGLING_FRAGMENT = `/* The sidebar is one persistent 224px tree clipped by a 52px shell.
 * Every rail glyph remains centered at x=26 in both states. */
  .records-footer-hint {
    display: none;
  }
}

`

function scopeTailwind(css) {
  return css.replace(
    '@import "tailwindcss";',
    [
      '@import "tailwindcss" source(none);',
      '@source "../../pill";',
      '@source "../../components/atoms";',
      '@source "../../components/primitives";',
    ].join('\n'),
  )
}

for (const [from, to] of FILES) {
  let content = readFileSync(path.join(source, from), 'utf8')
  if (from.endsWith('LoadingState.tsx')) content = removeSurfer(content)
  if (from.endsWith('foundation.css')) content = scopeTailwind(content.replace(DANGLING_FRAGMENT, ''))
  const target = path.join(dest, to)
  mkdirSync(path.dirname(target), { recursive: true })
  writeFileSync(target, content)
  console.log(`vendored ${to}`)
}
