import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { GamepadProvider } from '../solid/src/services/input/gamepad.provider';
import { InputManager } from '../solid/src/services/input/input.manager';
import { downloadModalJobCommand } from '../solid/src/components/modals/DownloadSourceModal';
import type { InputAction, InputDeviceStatus, IInputProvider } from '../solid/src/types/input.types';

const buttons = Array.from({ length: 17 }, () => ({ pressed: false, touched: false, value: 0 }));
const axes = [0, 0];
const pad = { buttons, axes };
const actions: InputAction[] = [];
const provider = new GamepadProvider();
const library = readFileSync(new URL('../solid/src/components/library/XmbLibrary.tsx', import.meta.url), 'utf8');
assert.ok(!library.includes('onMouseEnter='));
assert.ok(!library.includes('onMouseMove='));
try {
  provider.onAction(action => actions.push(action));
  const tick = (time: number) => provider.processGamepadInput(pad, time);
  buttons[13].pressed = true;
  axes[1] = 1;
  tick(1);
  assert.deepEqual(actions, ['NAV_DOWN']);
  tick(100);
  tick(350);
  assert.equal(actions.length, 1);
  tick(351);
  assert.deepEqual(actions, ['NAV_DOWN', 'NAV_DOWN']);
  tick(470);
  assert.equal(actions.length, 2);
  tick(471);
  assert.equal(actions.length, 3);
  buttons[13].pressed = false;
  axes[1] = 0;
  tick(480);
  buttons[13].pressed = true;
  tick(481);
  assert.equal(actions.length, 4);
  buttons[13].pressed = false;
  buttons[12].pressed = true;
  tick(482);
  assert.equal(actions.at(-1), 'NAV_UP');
  assert.equal(actions.length, 5);
  buttons[12].pressed = false;
  tick(483);
  axes[0] = -1;
  tick(484);
  assert.equal(actions.at(-1), 'NAV_LEFT');
  axes[0] = 1;
  tick(485);
  assert.equal(actions.at(-1), 'NAV_RIGHT');
  assert.equal(actions.length, 7);
  console.log('D-pad/stick: one action per direction, controlled hold repeat and release: OK');
} finally {
  provider.destroy();
}

let emitAction!: (action: InputAction) => void;
const activeGamepad: InputDeviceStatus = {
  isConnected: true,
  deviceName: 'Fixture gamepad',
  source: 'gamepad',
};
const fixtureProvider: IInputProvider = {
  id: 'fixture-gamepad', name: 'Fixture Gamepad', init: () => {}, destroy: () => {},
  onAction: listener => { emitAction = listener; return () => {}; },
  onStatusChange: () => () => {},
  getStatus: () => activeGamepad,
};
const manager = new InputManager();
let actionSource: InputDeviceStatus | undefined;
manager.registerProvider(fixtureProvider);
manager.onAction((_action, status) => { actionSource = status; });
emitAction('BUTTON_Y');
assert.deepEqual(actionSource, activeGamepad);
manager.destroy();
console.log('Input source: the last action retains its originating provider.');

for (const [status, start, select] of [
  ['queued', 'pause', 'cancel'],
  ['downloading', 'pause', 'cancel'],
  ['paused', 'resume', 'cancel'],
  ['failed', 'resume', 'cancel'],
  ['downloaded', 'resume', null],
  ['cancelled', 'retry', 'delete'],
  ['completed', null, null],
] as const) {
  assert.equal(downloadModalJobCommand('BUTTON_START', status), start);
  assert.equal(downloadModalJobCommand('BUTTON_SELECT', status), select);
}
assert.equal(downloadModalJobCommand('BUTTON_SELECT'), null);
assert.equal(downloadModalJobCommand('BUTTON_A', 'cancelled'), null);