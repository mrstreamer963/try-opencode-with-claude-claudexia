import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import wasm from 'vite-plugin-wasm'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    wasm(),
  ],
  worker: {
    format: 'es',
    plugins: () => [
      wasm(),
    ],
  },
  optimizeDeps: {
    exclude: ['game-core'],
  },
})
