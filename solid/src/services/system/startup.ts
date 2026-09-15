import type { StartupReport } from '@contracts/startup.types';

export function startupErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  if (error && typeof error === 'object') {
    for (const key of ['message', 'details']) {
      if (key in error && typeof (error as Record<string, unknown>)[key] === 'string') {
        return (error as Record<string, string>)[key];
      }
    }
  }
  return 'No se pudo completar la preparacion de EmuBox';
}

export function startupMessage(report: StartupReport): string {
  if (report.phase === 'error') return report.error || 'No se pudo preparar EmuBox';
  if (report.phase !== 'preparing') return 'Preparando biblioteca...';
  const task = report.tasks.find(entry => entry.state === 'running');
  return task ? {
    library: 'Preparando biblioteca guardada...',
    hardware: 'Comprobando el entorno grafico...',
    services: 'Comprobando servicios de sesion...',
    emulators: 'Preparando emuladores...',
  }[task.id] : 'Preparando EmuBox...';
}

export async function waitForStartup(
  read: () => Promise<StartupReport>,
  subscribe: (listener: (report: StartupReport) => void) => Promise<() => void>,
  onStatus: (report: StartupReport) => void,
  signal: AbortSignal,
  timeoutMs = 65000,
): Promise<StartupReport> {
  return new Promise((resolve, reject) => {
    let stopped = false;
    let unlisten: (() => void) | undefined;
    const finish = (error?: Error, report?: StartupReport) => {
      if (stopped) return;
      stopped = true;
      clearTimeout(timer);
      signal.removeEventListener('abort', aborted);
      unlisten?.();
      if (error) reject(error);
      else resolve(report!);
    };
    const aborted = () => finish(new Error('Preparacion cancelada'));
    const accept = (report: StartupReport) => {
      if (stopped) return;
      onStatus(report);
      if (report.phase === 'error') finish(new Error(startupMessage(report)));
      else if (report.phase !== 'preparing') finish(undefined, report);
    };
    const timer = setTimeout(() => finish(new Error('Se agoto el tiempo de preparacion nativa')), timeoutMs);
    signal.addEventListener('abort', aborted, { once: true });
    if (signal.aborted) { aborted(); return; }
    subscribe(accept).then(async cleanup => {
      unlisten = cleanup;
      if (stopped) { cleanup(); return; }
      accept(await read());
    }).catch(error => finish(new Error(startupErrorMessage(error))));
  });
}