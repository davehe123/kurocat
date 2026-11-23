import { defineConfig } from 'vite';

export default defineConfig({
  clearScreen: false,
  build: { rollupOptions: { input: { pet: 'index.html', control: 'control.html' } } },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
});
