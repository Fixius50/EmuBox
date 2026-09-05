import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { createServer } from 'node:net';

const read = file => readFileSync(new URL(`../${file}`, import.meta.url), 'utf8');
const setup = read('scripts/setup-arch.sh');
const autostart = read('scripts/setup-autostart.sh');
const installer = read('installer/install.sh');
const launcher = read('scripts/run.sh');
const cursorEnvironment = launcher.split('\n').filter(line => line.startsWith('export XCURSOR_')).join('\n');
assert.ok(cursorEnvironment.includes('XCURSOR_THEME'));
assert.ok(cursorEnvironment.includes('XCURSOR_SIZE'));
for (const [theme, size, expected] of [['', '', 'Adwaita:32'], ['Custom', '48', 'Custom:48']]) {
  const result = spawnSync('bash', ['-s'], {
    input: `${cursorEnvironment}\nprintf '%s:%s' "$XCURSOR_THEME" "$XCURSOR_SIZE"`,
    env: { ...process.env, XCURSOR_THEME: theme, XCURSOR_SIZE: size }, encoding: 'utf8',
  });
  assert.equal(result.status, 0);
  assert.equal(result.stdout, expected);
}
const cursorCss = read('solid/src/styles/base.css');
assert.ok(!/cursor:\s*(?:url\(|none)/.test(cursorCss));
assert.match(cursorCss, /cursor:\s*default/);
assert.match(cursorCss, /cursor:\s*pointer/);
const heredoc = (source, marker) => {
  assert.ok(source.includes(marker), `Missing generated configuration: ${marker}`);
  return source.split(marker)[1].split('\nEOF')[0];
};

assert.ok(installer.includes('exec bash "${SCRIPT_DIR}/../scripts/setup-arch.sh" "$@"'));
assert.ok(setup.includes('bash "${EMUBOX_DIR}/scripts/setup-autostart.sh"'));
assert.ok(setup.includes('/installer/setup/dependencies.sh'));
assert.ok(!setup.includes('systemctl enable emubox.service'));
for (const source of [setup, autostart]) assert.ok(!source.includes('rm -rf /var/lib/emubox/roms'));
for (const name of ['emubox', 'emubox-launcher']) {
  const script = heredoc(setup, `cat > /usr/local/bin/${name} <<'EOF'\n`);
  assert.ok(script.includes('exec bash /opt/emubox/scripts/run.sh "$@"'));
  assert.equal(spawnSync('bash', ['-n'], { input: script }).status, 0);
}
const tmpfiles = heredoc(autostart, 'cat > /etc/tmpfiles.d/emubox.conf <<EOF\n');
assert.match(tmpfiles, /^d \/run\/emubox 0755 \$EMUBOX_USER \$EMUBOX_USER -/);
assert.ok(autostart.includes('systemctl --global enable pipewire.socket pipewire-pulse.socket wireplumber.service'));
assert.ok(autostart.includes('systemctl disable "$SERVICE_NAME"'));
const getty = heredoc(autostart, 'cat << EOF > "${GETTY_OVERRIDE_DIR}/emubox-autologin.conf"\n');
assert.ok(getty.includes('--autologin ${EMUBOX_USER}'));
assert.ok(autostart.includes('EMUBOX_USER="emubox"'));
assert.ok(autostart.includes('source "$SCRIPT_DIR/../installer/lib/permissions.sh"'));
assert.match(autostart, /^setup_udev_rules$/m);

const directory = mkdtempSync(path.join(tmpdir(), 'emubox-unit-test-'));
try {
  const preflightStart = autostart.indexOf('AUDIO_UNIT_DIR=');
  const preflightEnd = autostart.indexOf('EMUBOX_UID=');
  assert.ok(preflightStart > 0 && preflightEnd > preflightStart);
  assert.ok(preflightEnd < autostart.indexOf('mkdir -p /etc/emubox'));
  const audioDirectory = path.join(directory, 'audio-units');
  mkdirSync(audioDirectory);
  const preflight = autostart.slice(preflightStart, preflightEnd)
    .replace('/usr/lib/systemd/user', audioDirectory);
  const checkAudio = () => spawnSync('bash', ['-s'], {
    input: `set -euo pipefail\n${preflight}\nprintf 'preflight-ok\\n'`, encoding: 'utf8',
  });
  const absent = checkAudio();
  assert.equal(absent.status, 1);
  assert.match(absent.stderr, /pipewire\.socket/);
  assert.match(absent.stderr, /sudo pacman -Syu --needed/);
  assert.ok(!absent.stdout.includes('preflight-ok'));
  for (const unit of ['pipewire.socket', 'pipewire-pulse.socket']) {
    writeFileSync(path.join(audioDirectory, unit), '');
    assert.equal(checkAudio().status, 1);
  }
  writeFileSync(path.join(audioDirectory, 'wireplumber.service'), '');
  assert.equal(checkAudio().status, 0);

  const activationStart = autostart.indexOf('if [[ -S "/run/user/$EMUBOX_UID/bus" ]]');
  assert.ok(activationStart > 0);
  const activationEnd = autostart.indexOf('\nfi', activationStart) + 3;
  const runtimeDirectory = path.join(directory, 'runtime');
  mkdirSync(runtimeDirectory);
  const activation = autostart.slice(activationStart, activationEnd)
    .replaceAll('/run/user/$EMUBOX_UID', runtimeDirectory);
  const activate = () => spawnSync('bash', ['-s'], {
    input: `set -euo pipefail\nEMUBOX_USER=emubox\nrunuser() { printf '%s\\n' "$*"; }\n${activation}`,
    encoding: 'utf8',
  });
  const noSession = activate();
  assert.equal(noSession.status, 0);
  assert.equal(noSession.stdout, '');
  const bus = createServer();
  await new Promise((resolve, reject) => {
    bus.once('error', reject);
    bus.listen(path.join(runtimeDirectory, 'bus'), resolve);
  });
  try {
    const existingSession = activate();
    assert.equal(existingSession.status, 0, existingSession.stderr);
    const commands = existingSession.stdout.trim().split('\n');
    assert.equal(commands.length, 2);
    assert.match(commands[0], /^-u emubox -- env .*systemctl --user daemon-reload$/);
    assert.match(commands[1], /systemctl --user start pipewire.socket pipewire-pulse.socket wireplumber.service$/);
    assert.ok(commands.every(command => command.includes(`DBUS_SESSION_BUS_ADDRESS=unix:path=${runtimeDirectory}/bus`)));
  } finally {
    await new Promise(resolve => bus.close(resolve));
  }

  const unit = heredoc(autostart, 'cat > "/etc/systemd/system/$SERVICE_NAME" <<EOF\n');
  assert.ok(unit.includes('ExecStart=/usr/local/bin/emubox-session'));
  const verificationUnit = unit.replaceAll('$EMUBOX_USER', 'nobody')
    .replaceAll('$EMUBOX_HOME', '/tmp')
    .replace('ExecStart=/usr/local/bin/emubox-session', 'ExecStart=/usr/bin/true');
  const unitPath = path.join(directory, 'emubox-test.service');
  writeFileSync(unitPath, verificationUnit);
  const result = spawnSync('systemd-analyze', ['verify', '--man=no', unitPath], { encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr || result.error?.message);
} finally {
  rmSync(directory, { recursive: true, force: true });
}
console.log('Common launcher, TTY1, tmpfiles, audio and systemd unit syntax: OK');