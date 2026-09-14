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
  'src-tauri/src/services/downloads/mod.rs',
  'src-tauri/src/services/downloads/service/catalog.rs',
  'src-tauri/src/services/downloads/service/repository.rs',
  'src-tauri/src/services/downloads/manager/transfer.rs',
  'src-tauri/src/services/downloads/manager/publication.rs',
  'src-tauri/src/services/downloads/installers/sandbox.rs',
  'src-tauri/src/services/downloads/installers/launch.rs',
  'src-tauri/src/services/graphics/mod.rs',
  'src-tauri/src/services/infrastructure/mod.rs',
  'src-tauri/src/services/library/games/scanner.rs',
  'src-tauri/src/services/library/games/repository.rs',
  'src-tauri/src/services/library/platforms.rs',
  'src-tauri/src/services/runtime/mod.rs',
  'src-tauri/src/services/system/mod.rs',
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
