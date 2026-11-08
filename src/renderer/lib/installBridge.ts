/**
 * Connect `window.murmur` to the real app, or to a stand-in when the page is opened in a plain
 * browser during development. The stand-in is stripped from production builds.
 */
export async function installBridge(): Promise<void> {
  if ('__TAURI_INTERNALS__' in window) {
    const { installTauriBridge } = await import('./bridge')
    installTauriBridge()
  } else if (import.meta.env.DEV) {
    const { installDevBridge } = await import('./devBridge')
    installDevBridge()
  }
}
