import { defineConfig } from 'vitest/config'
import { resolve } from 'path'

export default defineConfig({
  test: {
    environment: 'node',
    exclude: ['node_modules/**', 'tests/playwright/**']
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      '@utils': resolve(__dirname, 'src/script/utils'),
      '@type': resolve(__dirname, 'src/type')
    }
  }
})
