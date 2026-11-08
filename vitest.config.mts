import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vitest/config'

const projectDir = fileURLToPath(new URL('.', import.meta.url))

// Kept separate from vite.config.mts, whose root is the renderer folder.
export default defineConfig({
  resolve: { alias: { '@': resolve(projectDir, 'src/renderer') } },
  test: { include: ['src/**/*.test.ts'] },
})
