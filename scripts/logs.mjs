import { createReadStream } from 'node:fs';
import { createInterface } from 'node:readline';
import { parseArgs } from 'node:util';
import { pathToFileURL } from 'node:url';

const levels = { trace: 0, debug: 1, info: 2, warn: 3, error: 4 };

export function matches(entry, filter) {
  return (levels[entry.level] ?? -1) >= levels[filter.level ?? 'trace']
    && (!filter.source || entry.source?.startsWith(filter.source))
    && (!filter.event || entry.event === filter.event)
    && (!filter.session || entry.sessionId === filter.session)
    && (!filter.since || entry.timestampMs >= filter.since);
}

export async function readLogs(file, filter = {}) {
  const limit = filter.limit ?? 100;
  if (!Number.isInteger(limit) || limit < 1 || limit > 10000) throw new Error('limit debe estar entre 1 y 10000');
  if (!((filter.level ?? 'trace') in levels)) throw new Error('Nivel de registro invalido');
  const ring = new Array(limit);
  let count = 0;
  let malformed = 0;
  let filesRead = 0;
  for (const path of [...Array.from({ length: 8 }, (_, index) => `${file}.${8 - index}`), file]) {
    const input = createReadStream(path, { encoding: 'utf8', highWaterMark: 32768 });
    const lines = createInterface({ input, crlfDelay: Infinity });
    try {
      for await (const line of lines) {
        if (!line.trim()) continue;
        try {
          const entry = JSON.parse(line);
          if (entry && matches(entry, filter)) { ring[count % limit] = entry; count++; }
        } catch { malformed++; }
      }
      filesRead++;
    } catch (error) { if (error.code !== 'ENOENT') throw error; }
    finally { lines.close(); input.destroy(); }
  }
  if (!filesRead) throw new Error('No hay archivo de eventos; inicia una sesion con el controlador de registros');
  const length = Math.min(count, limit);
  const entries = Array.from({ length }, (_, index) => ring[(count - length + index) % limit]);
  return { entries, matched: count, malformed };
}

async function main() {
  const { values } = parseArgs({ options: {
    file: { type: 'string', default: '/var/log/emubox/events.jsonl' },
    source: { type: 'string' }, event: { type: 'string' }, session: { type: 'string' },
    since: { type: 'string' }, minutes: { type: 'string' }, level: { type: 'string', default: 'trace' },
    limit: { type: 'string', default: '100' }, json: { type: 'boolean' }, help: { type: 'boolean' },
  } });
  if (values.help) {
    console.log('node scripts/logs.mjs [--minutes 10 | --since ISO-8601] [--level warn] [--source ui] [--event incident.mark] [--session ID] [--limit 100] [--json]');
    return;
  }
  if (!(values.level in levels)) throw new Error('Niveles: trace, debug, info, warn, error');
  const since = values.since ? Date.parse(values.since) : values.minutes ? Date.now() - Number(values.minutes) * 60000 : undefined;
  if (since !== undefined && !Number.isFinite(since)) throw new Error('Fecha o minutos invalidos');
  const result = await readLogs(values.file, { ...values, since, limit: Number(values.limit) });
  for (const entry of result.entries) {
    console.log(values.json ? JSON.stringify(entry)
      : `${entry.timestamp} +${entry.elapsedMs}ms [${entry.level}] ${entry.source} ${entry.event} pid=${entry.pid ?? '?'} ${entry.message ?? ''} ${JSON.stringify(entry.data ?? {})}`);
  }
  console.error(`Eventos coincidentes: ${result.matched}; mostrados: ${result.entries.length}; lineas incompletas/invalidas: ${result.malformed}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch(error => { console.error(error.message); process.exitCode = 1; });
}