import { execSync } from 'child_process';
import os from 'os';

/**
 * Capture OS-level memory metrics for a given process PID or process name
 */
export function getProcessMemoryMetrics(pidOrName) {
  const isWindows = os.platform() === 'win32';

  if (isWindows) {
    try {
      const psCommand = `powershell -NoProfile -Command "Get-Process | Where-Object { $_.ProcessName -like '*${pidOrName}*' -or $_.Id -eq '${pidOrName}' } | Select-Object -Property Id, ProcessName, WorkingSet64, PrivateMemorySize64 | ConvertTo-Json"`;
      const stdout = execSync(psCommand, { encoding: 'utf-8' });
      if (!stdout || stdout.trim().length === 0) return null;

      const data = JSON.parse(stdout);
      const proc = Array.isArray(data) ? data[0] : data;
      if (!proc) return null;

      return {
        pid: proc.Id,
        processName: proc.ProcessName,
        rssMb: Math.round((proc.WorkingSet64 / (1024 * 1024)) * 10) / 10,
        privateMemoryMb: Math.round((proc.PrivateMemorySize64 / (1024 * 1024)) * 10) / 10
      };
    } catch (e) {
      return null;
    }
  } else {
    // Linux (Arch Linux / Debian / etc.)
    try {
      const cmd = `ps -o pid,comm,rss,vsz -C "${pidOrName}" --no-headers || ps -p ${pidOrName} -o pid,comm,rss,vsz --no-headers`;
      const stdout = execSync(cmd, { encoding: 'utf-8' });
      const lines = stdout.trim().split('\n');
      if (lines.length === 0 || !lines[0]) return null;

      const parts = lines[0].trim().split(/\s+/);
      const pid = parseInt(parts[0], 10);
      const name = parts[1];
      const rssKb = parseInt(parts[2], 10);
      const vszKb = parseInt(parts[3], 10);

      return {
        pid,
        processName: name,
        rssMb: Math.round((rssKb / 1024) * 10) / 10,
        privateMemoryMb: Math.round((vszKb / 1024) * 10) / 10
      };
    } catch (e) {
      return null;
    }
  }
}

// CLI test mode
if (process.argv[2]) {
  const target = process.argv[2];
  console.log(`Midiendo métricas de proceso OS para: ${target}`);
  const metrics = getProcessMemoryMetrics(target);
  console.log('Resultado:', metrics);
}
