import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';
import { TauriBackendService } from '../solid/src/services/backend/tauri-backend.service';

const backend = new TauriBackendService();
assert.equal(backend.isTauriEnvironment, false);
for (const operation of [
  () => backend.getGames(),
  () => backend.getHardwareInfo(),
  () => backend.getSettings(),
  () => backend.getDownloadJobs(),
  () => backend.getDownloadCandidates('unavailable'),
  () => backend.selectDownloadCandidate('unavailable', 'disc.iso'),
  () => backend.launchGame('unavailable'),
  () => backend.executeCommand('true'),
]) {
  await assert.rejects(operation, /runtime nativo Tauri/);
}
console.log('Native IPC: absent runtime rejects operations without fabricated results.');
const source = ts.createSourceFile('backend.ts', readFileSync(new URL('../solid/src/services/backend/tauri-backend.service.ts', import.meta.url), 'utf8'), ts.ScriptTarget.Latest, true);
const registered = new Set([...readFileSync(new URL('../src-tauri/src/lib.rs', import.meta.url), 'utf8').matchAll(/commands::\w+::(\w+)/g)].map(match => match[1]));
let checked = 0;
function visit(node: ts.Node) {
  if (ts.isCallExpression(node) && ts.isPropertyAccessExpression(node.expression) && node.expression.expression.kind === ts.SyntaxKind.ThisKeyword
    && node.expression.name.text === 'invoke' && node.arguments[0] && ts.isStringLiteral(node.arguments[0])) {
    assert.ok(registered.has(node.arguments[0].text), `Unregistered native command: ${node.arguments[0].text}`);
    checked++;
  }
  ts.forEachChild(node, visit);
}
visit(source);
assert.ok(checked > 40);
await assert.rejects(() => backend.checkForUpdates(), /no disponible/);
console.log(`IPC registration: ${checked} actual adapter commands verified against Rust.`);