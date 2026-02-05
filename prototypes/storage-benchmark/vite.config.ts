import { defineConfig } from 'vite';

export default defineConfig({
  root: './public',
  publicDir: '../public',
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    target: 'es2022',
  },
  server: {
    port: 3000,
    open: true,
  },
  optimizeDeps: {
    include: ['@automerge/automerge'],
  },
  resolve: {
    extensions: ['.ts', '.js'],
  },
});
