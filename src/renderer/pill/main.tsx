import '@fontsource-variable/inter'
import '@/styles/pill.css'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { installBridge } from '@/lib/installBridge'
import { Pill } from './Pill'

// Follow the system light/dark setting; the pill sits over other apps, so it adapts to them.
const dark = window.matchMedia('(prefers-color-scheme: dark)')
const applyScheme = (): void => {
  document.documentElement.classList.toggle('dark', dark.matches)
}
applyScheme()
dark.addEventListener('change', applyScheme)

await installBridge()

const container = document.getElementById('root')
if (container) {
  createRoot(container).render(
    <StrictMode>
      <Pill />
    </StrictMode>,
  )
}
