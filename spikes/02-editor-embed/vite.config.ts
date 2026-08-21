import { defineConfig } from 'vite';
import solid from 'vite-plugin-solid';

export default defineConfig({
  plugins: [solid()],
  // Tauri expects a fixed port and no clearing of the terminal.
  clearScreen: false,
  server: { port: 1421, strictPort: true },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./test/setup.ts'],
    // rust-analyzer needs time to index before it answers anything.
    testTimeout: 120_000,
    hookTimeout: 120_000,
    server: { deps: { inline: [/solid-js/, /@codemirror/] } },
  },
  resolve: { conditions: ['development', 'browser'] },
});
