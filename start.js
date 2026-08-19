import readline from 'readline';
import { spawn, execSync } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const cyan = (text) => `\x1b[36m${text}\x1b[0m`;
const green = (text) => `\x1b[32m${text}\x1b[0m`;
const yellow = (text) => `\x1b[33m${text}\x1b[0m`;
const magenta = (text) => `\x1b[35m${text}\x1b[0m`;
const bold = (text) => `\x1b[1m${text}\x1b[0m`;
const dim = (text) => `\x1b[2m${text}\x1b[0m`;

let activeChild = null;
let isRunningApp = false;

function clearScreen() {
  process.stdout.write('\x1b[2J\x1b[0f');
}

function freePort(port) {
  if (process.platform === 'win32') {
    try {
      const output = execSync(`netstat -ano | findstr :${port}`, { encoding: 'utf-8', stdio: ['pipe', 'pipe', 'ignore'] });
      const lines = output.split('\n');
      for (const line of lines) {
        if (line.includes('LISTENING')) {
          const parts = line.trim().split(/\s+/);
          const pid = parts[parts.length - 1];
          if (pid && pid !== '0' && pid !== process.pid.toString()) {
            try {
              execSync(`taskkill /F /T /PID ${pid}`, { stdio: 'ignore' });
            } catch (e) {}
          }
        }
      }
    } catch (e) {}
  }
}

function killActiveChild() {
  if (activeChild && activeChild.pid) {
    if (process.platform === 'win32') {
      try {
        execSync(`taskkill /F /T /PID ${activeChild.pid}`, { stdio: 'ignore' });
      } catch (e) {}
    } else {
      try {
        activeChild.kill('SIGKILL');
      } catch (e) {}
    }
    activeChild = null;
  }
  freePort(3001);
}

function handleCleanExit() {
  killActiveChild();
  console.log(cyan('\n  Cerrando procesos y liberando puertos...'));
  console.log(green('  ¡Todo limpio! Hasta luego.\n'));
  process.exit(0);
}

process.on('SIGINT', () => {
  if (isRunningApp) {
    killActiveChild();
    isRunningApp = false;
    setTimeout(() => showMenu(), 200);
  } else {
    handleCleanExit();
  }
});

process.on('SIGTERM', handleCleanExit);
process.on('exit', () => killActiveChild());

readline.emitKeypressEvents(process.stdin);
if (process.stdin.isTTY) {
  process.stdin.setRawMode(true);
}

process.stdin.on('keypress', (str, key) => {
  if (key && (key.ctrl && key.name === 'c') || (key && key.name === 'escape') || (str === 'q' && isRunningApp)) {
    if (isRunningApp) {
      console.log(yellow('\n\n  [INFO] Deteniendo servidor y volviendo al menú...'));
      killActiveChild();
      isRunningApp = false;
      setTimeout(() => showMenu(), 300);
    } else {
      handleCleanExit();
    }
  }
});

function showMenu() {
  isRunningApp = false;
  clearScreen();
  console.log(cyan(bold(`
  ===================================================================
     🎮  EMUBOX CONSOLE LAUNCHER (SOLIDJS 1.9 + KOBALTE)  🎮
  ===================================================================
  `)));
  console.log(`  ${green(bold('[1]'))} ${bold('Iniciar EmuBox Modo Desarrollo')}  ${dim('(Vite http://localhost:3001 - HMR)')}`);
  console.log(`  ${magenta(bold('[2]'))} ${bold('Ejecutar Test Suite F3')}         ${dim('(34 pruebas automatizadas de arquitectura)')}`);
  console.log(`  ${yellow(bold('[3]'))} ${bold('Compilar Frontend SolidJS')}     ${dim('(npm run build - dist/)')}`);
  console.log(`  ${cyan(bold('[4]'))} ${bold('Compilar Binario Tauri (Prod)')}  ${dim('(cargo build --release)')}\n`);

  console.log(`  ${dim('[0]')} ${dim('Salir (o presiona Ctrl+C / Esc)')}\n`);
  
  process.stdout.write(cyan(bold('  Introduce tu opción (0-4): ')));
}

process.stdin.on('data', (data) => {
  if (!isRunningApp) {
    const input = data.toString().trim();
    if (['1', '2', '3', '4', '0'].includes(input)) {
      handleChoice(input);
    }
  }
});

function runNpmScript(scriptName, label, port) {
  clearScreen();
  killActiveChild();
  if (port) freePort(port);
  isRunningApp = true;

  console.log(cyan(bold(`\n  ===================================================================`)));
  console.log(cyan(bold(`     Iniciando ${label}`)));
  console.log(cyan(bold(`  ===================================================================\n`)));
  console.log(yellow(`  ▶ Pulsa [Ctrl+C], [Esc] o escribe 'q' para volver al menú.\n`));
  console.log(dim('  -------------------------------------------------------------------\n'));

  const isWindows = process.platform === 'win32';
  const npmCmd = isWindows ? 'npm.cmd' : 'npm';

  activeChild = spawn(npmCmd, ['run', scriptName], {
    cwd: __dirname,
    stdio: 'inherit',
    shell: true
  });

  activeChild.on('exit', (code) => {
    if (isRunningApp) {
      isRunningApp = false;
      killActiveChild();
      console.log(dim(`\n  Proceso finalizado (código ${code}). Presiona cualquier tecla para volver al menú...`));
      const onKey = () => {
        process.stdin.removeListener('data', onKey);
        showMenu();
      };
      process.stdin.once('data', onKey);
    }
  });

  activeChild.on('error', (err) => {
    console.error(magenta(`  Error al ejecutar: ${err.message}`));
    isRunningApp = false;
    setTimeout(() => showMenu(), 2000);
  });
}

function handleChoice(choice) {
  switch (choice) {
    case '1':
      runNpmScript('dev', 'EmuBox Console UI (http://localhost:3001)', 3001);
      break;
    case '2':
      runNpmScript('test', 'Suite de Pruebas del Vertical Slice', null);
      break;
    case '3':
      runNpmScript('build', 'Compilación de Producción (SolidJS)', null);
      break;
    case '4':
      runNpmScript('tauri:build', 'Compilación de Binario Nativo Tauri (Producción)', null);
      break;
    case '0':
      handleCleanExit();
      break;
  }
}

showMenu();
