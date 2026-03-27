/// <reference types="vitest/config" />
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

const host = process.env['TAURI_DEV_HOST']

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': '/src',
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host != null && host !== '' ? host : false,
    ...(host != null && host !== ''
      ? {
          hmr: {
            protocol: 'ws' as const,
            host,
            port: 1421,
          },
        }
      : {}),
    watch: {
      ignored: ['**/backend/**'],
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test-setup.ts'],
  },
})
