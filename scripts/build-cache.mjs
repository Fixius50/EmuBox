import { createHash } from 'node:crypto';
import { existsSync, readFileSync, readdirSync, realpathSync, statSync, mkdirSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

export function fingerprint(root, entries, environment = '') {
  const digest = createHash('sha256').update(environment);
  const visited = new Set();
  const visit = relative => {
    const location = path.join(root, relative);
    digest.update(JSON.stringify(relative));
    if (!existsSync(location)) { digest.update('absent'); return; }
    const info = statSync(location);
    if (info.isDirectory()) {
      const real = realpathSync(location);
      if (visited.has(real)) return;
      visited.add(real);
      for (const entry of readdirSync(location).sort()) visit(path.join(relative, entry));
    } else if (info.isFile()) {
      digest.update(`${info.size}:`).update(readFileSync(location));
    }
  };
  for (const entry of [...entries].sort()) visit(entry);
  return digest.digest('hex');
}

function run(program, args) {
  const result = spawnSync(program, args, { stdio: 'inherit', shell: false });
  if (result.error || result.status !== 0) throw result.error || new Error(`${program} fallo (${result.status})`);
}

function dependencies(root) {
  const stamp = path.join(root, 'node_modules/.emubox-dependencies.json');
  const environment = `${process.version}:${process.platform}:${process.arch}`;
  const inputs = () => fingerprint(root, ['package.json', 'package-lock.json'], environment);
  let saved;
  try { saved = JSON.parse(readFileSync(stamp, 'utf8')); } catch { saved = null; }
  const reusable = process.env.EMUBOX_REINSTALL_DEPS !== '1' && saved?.inputs === inputs()
    && spawnSync('npm', ['ls', '--depth=0', '--json'], { stdio: 'ignore' }).status === 0;
  if (!reusable) run('npm', [existsSync(path.join(root, 'package-lock.json')) ? 'ci' : 'install', '--no-audit', '--no-fund']);
  const require = createRequire(path.join(root, 'package.json'));
  try { require('esbuild').transformSync('const ready = true;'); }
  catch { run('npm', ['rebuild', 'esbuild']); require('esbuild').transformSync('const ready = true;'); }
  writeFileSync(stamp, JSON.stringify({ inputs: inputs() }));
  console.log(reusable ? 'Dependencias verificadas y reutilizadas.' : 'Dependencias instaladas y verificadas.');
}

function frontend(root) {
  const stamp = path.join(root, 'node_modules/.cache/emubox/frontend.json');
  const environment = JSON.stringify(Object.entries(process.env).filter(([key]) => key.startsWith('VITE_') || key === 'NODE_ENV').sort());
  const envFiles = [root, path.join(root, 'solid')].flatMap(directory => readdirSync(directory)
    .filter(name => name.startsWith('.env')).map(name => path.relative(root, path.join(directory, name))));
  const inputs = fingerprint(root, ['solid/src', 'solid/public', 'solid/index.html', 'solid/vite.config.ts',
    'data', 'tsconfig.json', 'package.json', 'package-lock.json', 'scripts/build-cache.mjs', ...envFiles], `${process.version}:${process.arch}:${environment}`);
  let saved;
  try { saved = JSON.parse(readFileSync(stamp, 'utf8')); } catch { saved = null; }
  if (process.env.EMUBOX_REBUILD_FRONTEND !== '1' && saved?.inputs === inputs
    && existsSync(path.join(root, 'solid/dist/index.html')) && saved.outputs === fingerprint(root, ['solid/dist'])) {
    console.log('Frontend sin cambios: artefactos verificados y reutilizados.');
    return;
  }
  run('npm', ['run', 'build']);
  mkdirSync(path.dirname(stamp), { recursive: true });
  writeFileSync(stamp, JSON.stringify({ inputs, outputs: fingerprint(root, ['solid/dist']) }));
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  try {
    process.chdir(root);
    switch (process.argv[2]) {
      case 'dependencies': dependencies(root); break;
      case 'frontend': frontend(root); break;
      default: throw new Error('Uso: build-cache.mjs dependencies|frontend');
    }
  } catch (error) { console.error(error); process.exitCode = 1; }
}