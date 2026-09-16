export interface FrontendLogEvent {
  event: string;
  level: 'debug' | 'info' | 'warn' | 'error';
  timestampMs: number;
  elapsedMs: number;
  message: string;
  data: Record<string, string | number | boolean | null>;
}

type Send = (entries: FrontendLogEvent[]) => Promise<void>;
let report: ((entry: FrontendLogEvent) => void) | undefined;

export function recordUiEvent(event: string, data: FrontendLogEvent['data'] = {}, level: FrontendLogEvent['level'] = 'info', message = ''): void {
  report?.({ event, level, timestampMs: Date.now(), elapsedMs: performance.now(), message: message.slice(0, 2048), data });
}

export function createTelemetryBuffer(send: Send) {
  let pending: FrontendLogEvent[] = [];
  let dropped = 0;
  let busy = false;
  let closed = false;
  return {
    push(entry: FrontendLogEvent) {
      if (closed) return;
      if (pending.length >= 64) { dropped++; return; }
      pending.push(entry);
    },
    async flush() {
      if (closed || busy || !pending.length) return;
      busy = true;
      const entries = pending.splice(0, 32);
      if (dropped) {
        entries[0] = { ...entries[0], data: { ...entries[0].data, droppedEvents: dropped } };
        dropped = 0;
      }
      try { await send(entries); } catch { dropped += entries.length; }
      finally { busy = false; }
    },
    close() { closed = true; pending = []; },
  };
}

export function startFrontendTelemetry(send: Send): () => void {
  const buffer = createTelemetryBuffer(send);
  report = entry => buffer.push(entry);
  const original = { log: console.log, info: console.info, warn: console.warn, error: console.error, debug: console.debug };
  for (const method of ['log', 'info', 'warn', 'error', 'debug'] as const) {
    console[method] = (...args: unknown[]) => {
      original[method].apply(console, args);
      const text = args.filter(value => typeof value === 'string').join(' ').slice(0, 2048);
      recordUiEvent('console.message', { method }, method === 'log' ? 'info' : method, text || '[non-text console message]');
    };
  }
  const state = () => ({ hidden: document.hidden, focused: document.hasFocus(),
    pipeline: document.documentElement.getAttribute('data-render-pipeline'),
    blur: document.documentElement.getAttribute('data-blur-mode'),
    width: window.innerWidth, height: window.innerHeight });
  const click = (event: MouseEvent) => recordUiEvent('input.click', { ...state(), button: event.button,
    targetTag: event.target instanceof Element ? event.target.tagName : 'unknown', trusted: event.isTrusted });
  const focus = (event: Event) => recordUiEvent('window.focus', { ...state(), type: event.type });
  const visibility = () => recordUiEvent('window.visibility', state());
  const resize = () => recordUiEvent('window.resize', state());
  const failure = (event: ErrorEvent) => recordUiEvent('window.error', { line: event.lineno, column: event.colno }, 'error', event.message);
  const rejection = (event: PromiseRejectionEvent) => recordUiEvent('promise.rejection', {}, 'error',
    event.reason instanceof Error ? event.reason.message : typeof event.reason === 'string' ? event.reason : '[non-text rejection]');
  const contextLost = () => recordUiEvent('graphics.context-lost', state(), 'error');
  const mark = (event: KeyboardEvent) => {
    if (event.key === 'F8' && !event.repeat) recordUiEvent('incident.mark', state(), 'warn', 'Marca manual del fallo visual');
  };
  document.addEventListener('click', click, true);
  document.addEventListener('visibilitychange', visibility);
  document.addEventListener('webglcontextlost', contextLost, true);
  window.addEventListener('focus', focus);
  window.addEventListener('blur', focus);
  window.addEventListener('resize', resize);
  window.addEventListener('error', failure);
  window.addEventListener('unhandledrejection', rejection);
  window.addEventListener('keydown', mark);
  let frame = 0;
  let lastFrame = performance.now();
  let maxGapMs = 0;
  let frames = 0;
  const tick = (now: number) => {
    maxGapMs = Math.max(maxGapMs, now - lastFrame);
    lastFrame = now;
    frames++;
    frame = requestAnimationFrame(tick);
  };
  frame = requestAnimationFrame(tick);
  const heartbeat = setInterval(() => {
    recordUiEvent('render.sample', { ...state(), frames, maxGapMs: Math.round(maxGapMs), sinceFrameMs: Math.round(performance.now() - lastFrame) },
      !document.hidden && (maxGapMs > 1000 || performance.now() - lastFrame > 1000) ? 'warn' : 'info');
    frames = 0;
    maxGapMs = 0;
  }, 5000);
  const flush = setInterval(() => { void buffer.flush(); }, 1000);
  recordUiEvent('session.start', state());
  return () => {
    report = undefined;
    clearInterval(flush);
    clearInterval(heartbeat);
    cancelAnimationFrame(frame);
    document.removeEventListener('click', click, true);
    document.removeEventListener('visibilitychange', visibility);
    document.removeEventListener('webglcontextlost', contextLost, true);
    window.removeEventListener('focus', focus);
    window.removeEventListener('blur', focus);
    window.removeEventListener('resize', resize);
    window.removeEventListener('error', failure);
    window.removeEventListener('unhandledrejection', rejection);
    window.removeEventListener('keydown', mark);
    Object.assign(console, original);
    buffer.close();
  };
}