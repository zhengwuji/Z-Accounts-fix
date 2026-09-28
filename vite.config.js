import { defineConfig } from 'vite';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
const srcRoot = resolve(root, 'src');

export default defineConfig({
  root: srcRoot,
  base: './',
  build: {
    outDir: resolve(root, 'dist'),
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: resolve(srcRoot, 'index.html'),
        settings: resolve(srcRoot, 'settings.html'),
        captcha: resolve(srcRoot, 'captcha.html'),
      },
    },
  },
  server: {
    host: '127.0.0.1',
    port: 5173,
    strictPort: true,
  },
  clearScreen: false,
});
