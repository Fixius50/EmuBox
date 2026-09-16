import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';
import { TauriBackendService } from '../solid/src/services/backend/tauri-backend.service';
import { startupErrorMessage, startupMessage, waitForStartup } from '../solid/src/services/system/startup';
import type { StartupReport } from '../solid/src/types/startup.types';
import { createTelemetryBuffer, type FrontendLogEvent } from '../solid/src/services/system/telemetry';

const tauriConfig = JSON.parse(readFileSync(new URL('../src-tauri/tauri.conf.json', import.meta.url), 'utf8'));
const eventCapability = JSON.parse(readFileSync(new URL('../src-tauri/capabilities/main-events.json', import.meta.url), 'utf8'));
assert.equal(tauriConfig.app.windows[0].label, 'main');
assert.equal(eventCapability.identifier, 'main-events');
assert.equal(eventCapability.local, true);
assert.equal(eventCapability.remote, undefined);
assert.deepEqual(eventCapability.windows, [tauriConfig.app.windows[0].label]);
assert.deepEqual(eventCapability.permissions, ['core:event:allow-listen', 'core:event:allow-unlisten']);
const selectedCapabilities = tauriConfig.app.security?.capabilities;
assert.ok(selectedCapabilities === undefined || selectedCapabilities.includes(eventCapability.identifier),
  'The main event capability must be selected or discovered automatically');
console.log('Native event ACL: local main window may subscribe/unsubscribe; no remote or emit grant.');

const backend = new TauriBackendService();
assert.equal(backend.isTauriEnvironment, false);
for (const operation of [
  () => backend.getGames(),
  () => backend.getHardwareInfo(),
  () => backend.getSettings(),
  () => backend.getStartupStatus(),
  () => backend.getStartupData(),
  () => backend.startupFrontendReady(),
  () => backend.getDownloadJobs(),
  () => backend.getDownloadCandidates('unavailable'),
  () => backend.selectDownloadCandidate('unavailable', 'disc.iso'),
  () => backend.launchGame('unavailable'),
  () => backend.recordFrontendEvents([]),
]) {
  await assert.rejects(operation, /runtime nativo Tauri/);
}
console.log('Native IPC: absent runtime rejects operations without fabricated results.');
const source = ts.createSourceFile('backend.ts', readFileSync(new URL('../solid/src/services/backend/tauri-backend.service.ts', import.meta.url), 'utf8'), ts.ScriptTarget.Latest, true);
const registered = new Set([...readFileSync(new URL('../src-tauri/src/lib.rs', import.meta.url), 'utf8').matchAll(/commands::\w+::(\w+)/g)].map(match => match[1]));
assert.equal(registered.has('execute_command'), false, 'Generic shell execution must not be exposed over IPC');
const nativeStartup = readFileSync(new URL('../src-tauri/src/services/runtime/startup.rs', import.meta.url), 'utf8');
assert.doesNotMatch(nativeStartup, /game_database::(?:ensure_local_index|sync_all)/,
  'Canonical catalog reconciliation must not block cached library startup');
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

const logBatches: FrontendLogEvent[][] = [];
let finishBatch!: () => void;
const telemetry = createTelemetryBuffer(async entries => {
  logBatches.push(entries);
  await new Promise<void>(resolve => { finishBatch = resolve; });
});
const logEntry: FrontendLogEvent = { event: 'input.click', level: 'info', message: '', timestampMs: 100, elapsedMs: 1, data: {} };
for (let index = 0; index < 70; index++) telemetry.push(logEntry);
const sending = telemetry.flush();
await telemetry.flush();
assert.equal(logBatches.length, 1);
assert.equal(logBatches[0].length, 32);
assert.equal(logBatches[0][0].data.droppedEvents, 6);
finishBatch();
await sending;
telemetry.close();
await telemetry.flush();
assert.equal(logBatches.length, 1);
console.log('Telemetry: bounded batches, dropped-event accounting, no concurrent IPC and cleanup verified.');

const prepared: StartupReport = {
  phase: 'prepared', elapsedMs: 10, warnings: [], error: null, tasks: [],
  resources: { cpuAvailable: 2, memoryAvailableMb: 512, maxConcurrentTasks: 1, ioSlots: 1,
    graphicsSlots: 1, graphicsState: 'indeterminate', graphicsBackend: 'auto', virtualMachine: true, storageAvailable: true },
};
assert.equal(startupErrorMessage({ details: 'Disk unavailable' }), 'Disk unavailable');
assert.equal(startupErrorMessage(new Error('Disk unavailable')), 'Disk unavailable');
assert.ok(startupErrorMessage(null).includes('preparacion'));
let unsubscribed = 0;
const order: string[] = [];
const fromSnapshot = await waitForStartup(async () => { order.push('read'); return prepared; },
  async () => { order.push('subscribe'); return () => { unsubscribed++; }; }, () => {}, new AbortController().signal);
assert.deepEqual(order, ['subscribe', 'read']);
assert.equal(fromSnapshot.phase, 'prepared');
assert.equal(unsubscribed, 1);
let readAfterEvent = false;
await waitForStartup(async () => { readAfterEvent = true; return prepared; }, async accept => {
  accept(prepared);
  return () => { unsubscribed++; };
}, () => {}, new AbortController().signal);
await Promise.resolve();
assert.equal(readAfterEvent, false);
assert.equal(unsubscribed, 2);
await assert.rejects(waitForStartup(async () => ({ ...prepared, phase: 'error', error: 'SQLite unreadable' }),
  async () => () => {}, () => {}, new AbortController().signal), /SQLite unreadable/);
const cancelled = new AbortController();
cancelled.abort();
await assert.rejects(waitForStartup(async () => prepared, async () => () => {}, () => {}, cancelled.signal), /cancelada/);
await assert.rejects(waitForStartup(async () => ({ ...prepared, phase: 'preparing' }),
  async () => () => {}, () => {}, new AbortController().signal, 5), /tiempo/);
let releaseSubscription!: (cleanup: () => void) => void;
const duringSubscribe = new AbortController();
const late = waitForStartup(async () => prepared, () => new Promise(resolve => { releaseSubscription = resolve; }),
  () => {}, duringSubscribe.signal);
duringSubscribe.abort();
await assert.rejects(late, /cancelada/);
releaseSubscription(() => { unsubscribed++; });
await Promise.resolve();
assert.equal(unsubscribed, 3);
assert.equal(startupMessage({ ...prepared, phase: 'preparing', tasks: [{ id: 'emulators', state: 'running', elapsedMs: 0 }] }), 'Preparando emuladores...');
console.log('Startup: snapshot/event ordering, native failure, cancellation, timeout and late listener cleanup verified.');