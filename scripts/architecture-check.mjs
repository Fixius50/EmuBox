import { existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const requiredPaths = [
  'package.json',
  'tsconfig.json',
  'solid/vite.config.ts',
  'solid/src/App.tsx',
  'src-tauri/Cargo.toml',
  'src-tauri/src/lib.rs',
  'src-tauri/src/services/emulators/mod.rs',
  'scripts/build.sh',
  'scripts/verify.mjs',
];
const missing = requiredPaths.filter((requiredPath) => !existsSync(path.join(root, requiredPath)));

if (missing.length > 0) {
  console.error(`Rutas requeridas ausentes: ${missing.join(', ')}`);
  process.exitCode = 1;
} else {
  console.log(`Arquitectura válida: ${requiredPaths.length} rutas críticas presentes.`);
}
