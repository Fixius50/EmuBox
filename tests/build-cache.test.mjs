import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, rmSync, mkdirSync, utimesSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fingerprint } from '../scripts/build-cache.mjs';

const root = mkdtempSync(path.join(tmpdir(), 'emubox-build-cache-'));
try {
  mkdirSync(path.join(root, 'src'));
  const source = path.join(root, 'src/main.ts');
  writeFileSync(source, 'export const value = 1;');
  const first = fingerprint(root, ['src', 'missing'], 'node-version');
  utimesSync(source, new Date(0), new Date(0));
  assert.equal(fingerprint(root, ['src', 'missing'], 'node-version'), first);
  writeFileSync(source, 'export const value = 2;');
  assert.notEqual(fingerprint(root, ['src', 'missing'], 'node-version'), first);
  writeFileSync(source, 'export const value = 1;');
  assert.notEqual(fingerprint(root, ['src', 'missing'], 'different-node'), first);
  writeFileSync(path.join(root, 'src/added.ts'), 'export {};');
  assert.notEqual(fingerprint(root, ['src', 'missing'], 'node-version'), first);
  rmSync(path.join(root, 'src/added.ts'));
  assert.equal(fingerprint(root, ['src', 'missing'], 'node-version'), first);
} finally { rmSync(root, { recursive: true, force: true }); }
console.log('Build cache: content/environment/additions invalidate; timestamps do not.');