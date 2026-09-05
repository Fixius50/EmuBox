import { accessSync, constants, existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const project = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function command(program, args = []) {
  const result = spawnSync(program, args, { encoding: 'utf8', timeout: 15000 });
  return { ok: !result.error && result.status === 0, text: result.stdout?.trim() ?? '' };
}

function readable(file) {
  try { return readFileSync(file, 'utf8'); } catch { return ''; }
}

function accessible(file, mode) {
  try { accessSync(file, mode); return true; } catch { return false; }
}

function devices(directory, pattern) {
  try {
    return readdirSync(directory).filter(name => pattern.test(name))
      .map(name => path.join(directory, name))
      .filter(file => statSync(file).isCharacterDevice());
  } catch { return []; }
}

export function collectApplianceFacts() {
  const architectureScript = path.join(project, 'installer/lib/architecture.sh');
  const account = command('getent', ['passwd', 'emubox']).text.split(':');
  const uid = account.length >= 7 ? Number(account[2]) : null;
  const runtimeDirectory = uid === null ? '' : `/run/user/${uid}`;
  const running = name => uid !== null && command('pgrep', ['-u', String(uid), '-x', name]).ok;
  const directories = ['/etc/emubox', '/var/lib/emubox', '/var/lib/emubox/games',
    '/var/lib/emubox/saves', '/var/lib/emubox/states', '/var/cache/emubox', '/var/log/emubox', '/run/emubox'];
  const directoryAccess = directories.map(directory => ({
    path: directory,
    writable: accessible(directory, constants.R_OK | constants.W_OK | constants.X_OK),
  }));
  const dri = devices('/dev/dri', /^(card\d+|renderD\d+)$/);
  const input = devices('/dev/input', /^event\d+$/);
  let waylandSocket;
  try {
    waylandSocket = readdirSync(runtimeDirectory).filter(name => /^wayland-\d+$/.test(name))
      .some(name => statSync(path.join(runtimeDirectory, name)).isSocket());
  } catch { waylandSocket = false; }
  const binary = path.join(project, 'bin/emubox');
  const libraries = command('ldconfig', ['-p']).text;
  const dbPath = '/var/lib/emubox/emubox.db';
  let databaseHeader = false;
  if (existsSync(dbPath)) {
    const header = command('head', ['-c', '16', dbPath]).text;
    databaseHeader = header.startsWith('SQLite format 3\0');
  }
  const autologin = command('systemctl', ['show', 'getty@tty1.service', '--property=ExecStart', '--value']).text;
  const auxiliary = command('systemctl', ['is-enabled', 'emubox.service']).text;
  return {
    architecture: command('bash', [architectureScript, '--host']).text || 'unsupported',
    distributionSupported: command('bash', [architectureScript, '--distribution']).ok,
    systemd: readable('/proc/1/comm').trim() === 'systemd',
    uid,
    inspectorUid: process.getuid?.() ?? null,
    binaryCompatible: accessible(binary, constants.X_OK) && command('bash', [architectureScript, '--binary', binary]).ok,
    librariesAvailable: ['libgtk-3.so.0', 'libwebkit2gtk-4.1.so.0', 'libjavascriptcoregtk-4.1.so.0']
      .every(library => libraries.includes(library)),
    directoryAccess,
    runtimePersistence: readable('/etc/tmpfiles.d/emubox.conf').split('\n')
      .some(line => /^d\s+\/run\/emubox\s+0755\s+emubox\s+emubox(?:\s|$)/.test(line)),
    autologin: /--autologin\s+emubox(?:\s|;|$)/.test(autologin),
    singleStartup: ['disabled', 'masked', 'not-found'].includes(auxiliary),
    ttyActive: command('systemctl', ['is-active', '--quiet', 'getty@tty1']).ok,
    launcherInstalled: readable('/usr/local/bin/emubox-session').includes('/opt/emubox/scripts/run.sh'),
    profileConfigured: readable(path.join(account[5] || '/nonexistent', '.bash_profile')).includes('/dev/tty1')
      && readable(path.join(account[5] || '/nonexistent', '.bash_profile')).includes('emubox-session'),
    drmAccessible: dri.some(file => accessible(file, constants.R_OK | constants.W_OK)),
    inputAccessible: input.some(file => accessible(file, constants.R_OK)),
    waylandSocket,
    sessionBus: runtimeDirectory !== '' && existsSync(path.join(runtimeDirectory, 'bus')),
    compositorRunning: running('cage') || running('gamescope'),
    runtimeRunning: running('emubox'),
    audioUnitsAvailable: ['pipewire.socket', 'pipewire-pulse.socket', 'wireplumber.service']
      .every(unit => existsSync(path.join('/usr/lib/systemd/user', unit))),
    audioRunning: running('pipewire') && running('wireplumber'),
    databaseHeader,
  };
}

export function evaluateAppliance(facts) {
  const checks = [];
  const check = (id, passed, detail, missing = 'fail') => checks.push({ id, status: passed ? 'pass' : missing, detail });
  check('architecture', ['x86_64', 'aarch64'].includes(facts.architecture), facts.architecture);
  check('distribution', facts.distributionSupported, 'Manual appliance base: Arch x86_64 or Arch Linux ARM aarch64');
  check('systemd', facts.systemd, 'PID 1 is systemd');
  check('user', Number.isInteger(facts.uid) && facts.uid > 0, 'Non-root emubox account');
  const correctInspector = facts.uid > 0 && facts.inspectorUid === facts.uid;
  check('inspector', correctInspector, 'Run this check as emubox, without sudo');
  check('native-runtime', facts.binaryCompatible, 'Executable ELF matches the host CPU');
  check('native-libraries', facts.librariesAvailable, 'GTK/WebKitGTK runtime libraries found in loader cache');
  for (const directory of facts.directoryAccess) {
    check(`filesystem:${directory.path}`, correctInspector && directory.writable, 'Access as emubox, not root');
  }
  check('runtime-persistence', facts.runtimePersistence, 'tmpfiles recreates /run/emubox at boot');
  check('autologin', facts.autologin, 'getty@tty1 logs in as emubox');
  check('single-startup', facts.singleStartup, 'No auxiliary system emubox.service enabled');
  check('session-launcher', facts.launcherInstalled && facts.profileConfigured, 'TTY1 profile delegates to the common launcher');
  check('tty-active', facts.ttyActive, 'TTY1 active', 'pending');
  check('drm-access', correctInspector && facts.drmAccessible, 'DRI device permissions', 'pending');
  check('input-access', correctInspector && facts.inputAccessible, 'Input device readable; controller behavior untested', 'pending');
  check('wayland', facts.waylandSocket && facts.compositorRunning, 'Wayland socket and compositor owned by emubox', 'pending');
  check('session-bus', facts.sessionBus, 'User session bus exists', 'pending');
  check('runtime-process', facts.runtimeRunning, 'EmuBox process owned by emubox; UI behavior untested', 'pending');
  check('audio-units', facts.audioUnitsAvailable, 'PipeWire sockets and WirePlumber user unit installed');
  check('audio-processes', facts.audioRunning, 'PipeWire and WirePlumber owned by emubox; sound untested', 'pending');
  check('sqlite-file', facts.databaseHeader, 'SQLite header present; no database opened or modified', 'pending');
  for (const [id, detail] of [
    ['cold-boot', 'Boot independently of SSH; verify one session'],
    ['ui-ipc', 'Visible responsive UI; real Tauri IPC, not browser mocks'],
    ['sqlite-persistence', 'Save an authorized setting/favorite, restart and verify persistence'],
    ['input-functional', 'Navigate with the physical controller and keyboard'],
    ['audio-functional', 'Verify audible output on the intended device'],
    ['graphics-functional', 'Check actual GPU renderer and compositor logs, including fallback'],
  ]) check(id, false, detail, 'pending');
  return {
    schemaVersion: 1,
    architecture: facts.architecture,
    scope: 'appliance-readiness',
    applianceValidated: false,
    distributionValidated: false,
    status: checks.some(item => item.status === 'fail') ? 'failed' : 'pending-functional-validation',
    checks,
  };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const report = evaluateAppliance(collectApplianceFacts());
  if (process.argv.includes('--json')) {
    console.log(JSON.stringify(report, null, 2));
  } else {
    console.log(`EmuBox appliance readiness (${report.architecture}): ${report.status}`);
    for (const item of report.checks) console.log(`[${item.status.toUpperCase()}] ${item.id}: ${item.detail}`);
    console.log('Read-only inspection. Emulators are not a prerequisite. Functional acceptance remains pending.');
  }
  process.exitCode = report.status === 'failed' ? 1 : 2;
}