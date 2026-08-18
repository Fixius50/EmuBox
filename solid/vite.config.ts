import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';
import path from 'path';

export default defineConfig({
  plugins: [solidPlugin()],
  server: {
    port: 3001,
    strictPort: true
  },
  resolve: {
    alias: {
      '@contracts': path.resolve(__dirname, 'src/types'),
      '@services': path.resolve(__dirname, 'src/services'),
      '@stores': path.resolve(__dirname, 'src/stores'),
      '@hooks': path.resolve(__dirname, 'src/hooks'),
      '@components': path.resolve(__dirname, 'src/components'),
      '@animations': path.resolve(__dirname, 'src/animations'),
      '@styles': path.resolve(__dirname, 'src/styles'),
      '@data': path.resolve(__dirname, '../data')
    }
  },
  build: {
    target: 'esnext'
  }
});
