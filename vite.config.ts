import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue(), vueDevTools()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  // Prevent Vite from clearing the Rust console
  clearScreen: false,
  server: {
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ['**/crates/**', '**/backend/**'],
    },
  },
})
