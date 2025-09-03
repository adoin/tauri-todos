import type { UserConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import UnoCSS from 'unocss/vite'
import { defineConfig } from 'vite'
import unoConfig from './uno.config'

// Get the host from environment variables
// @ts-expect-error process is available in Node.js environment
const host = process?.env?.TAURI_DEV_HOST

// https://vitejs.dev/config/
export default defineConfig(async (): Promise<UserConfig> => ({
  plugins: [
    vue(),
    UnoCSS(unoConfig),
  ],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
  // to access the Tauri environment variables set by the CLI with process.env.VARIABLE_NAME
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    // @ts-expect-error process is available in Node.js environment
    target: process?.env?.TAURI_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    // don't minify for debug builds
    // @ts-expect-error process is available in Node.js environment
    minify: !process?.env?.TAURI_DEBUG ? 'esbuild' : false,
    // produce sourcemaps for debug builds
    // @ts-expect-error process is available in Node.js environment
    sourcemap: !!process?.env?.TAURI_DEBUG,
  },
}))
