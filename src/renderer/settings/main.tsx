import '@fontsource-variable/inter'
import '@/styles/app.css'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { applyTheme, THEME_STORAGE_KEY, type ThemeMode } from '@/components/application/theme/theme-toggle'
import { installBridge } from '@/lib/installBridge'
import { App } from './App'

/** Dark unless the user has chosen otherwise. The BoardUI toggle is manual, not system-driven. */
function initialTheme(): ThemeMode {
  try {
    const stored = window.localStorage.getItem(THEME_STORAGE_KEY)
    if (stored === 'light' || stored === 'dark') return stored
  } catch {
    // Storage can be unavailable; use the default.
  }
  return 'dark'
}

await installBridge()

applyTheme(initialTheme(), { persist: true })

const container = document.getElementById('root')
if (container) {
  createRoot(container).render(
    <StrictMode>
      <App />
    </StrictMode>,
  )
}
