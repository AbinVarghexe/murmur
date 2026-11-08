import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

const projectDir = fileURLToPath(new URL('.', import.meta.url))
const root = resolve(projectDir, 'src/renderer')

export default defineConfig({
  root,
  base: './',
  publicDir: resolve(root, 'public'),
  plugins: [react(), tailwindcss()],
  resolve: { alias: { '@': root } },
  build: {
    outDir: resolve(projectDir, 'dist/renderer'),
    emptyOutDir: true,
    target: 'chrome140',
    rollupOptions: {
      input: { settings: resolve(root, 'settings.html'), pill: resolve(root, 'pill.html') },
    },
  },
})
