import { defineConfig } from 'vite';
import solid from 'vite-plugin-solid';

export default defineConfig({
  plugins: [solid()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./test/setup.ts'],
    // solid-js must be resolved through its browser condition or the testing
    // library renders against the server build and nothing mounts.
    server: { deps: { inline: [/solid-js/, /@dschz\/solid-flow/] } },
  },
  resolve: { conditions: ['development', 'browser'] },
});
