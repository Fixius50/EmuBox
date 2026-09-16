import assert from 'node:assert/strict';
import { mkdtemp, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { matches, readLogs } from '../scripts/logs.mjs';

const root = await mkdtemp(path.join(tmpdir(), 'emubox-log-reader-'));
try {
  const file = path.join(root, 'events.jsonl');
  const first = { timestampMs: 100, level: 'info', source: 'ui.input', event: 'input.click', sessionId: 'first' };
  const second = { timestampMs: 200, level: 'warn', source: 'os.kernel', event: 'journal.message', sessionId: 'second' };
  const last = { timestampMs: 300, level: 'error', source: 'startup', event: 'operation.result', sessionId: 'second' };
  await writeFile(`${file}.1`, `${JSON.stringify(first)}\n`);
  await writeFile(file, `${JSON.stringify(second)}\n${JSON.stringify(last)}\n{"partial":`);
  assert.deepEqual((await readLogs(file, { limit: 2 })).entries, [second, last]);
  assert.equal((await readLogs(file)).malformed, 1);
  assert.deepEqual((await readLogs(file, { source: 'ui' })).entries, [first]);
  assert.deepEqual((await readLogs(file, { level: 'warn', session: 'second', since: 250 })).entries, [last]);
  assert.equal(matches(first, { event: 'incident.mark' }), false);
  await assert.rejects(readLogs(file, { limit: 0 }));
  await assert.rejects(readLogs(file, { level: 'invalid' }));
  await assert.rejects(readLogs(path.join(root, 'absent')));
} finally { await rm(root, { recursive: true, force: true }); }
console.log('Telemetry reader: rotations, bounded selection, source/level/session/time filters and incomplete lines: OK');