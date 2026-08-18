import { KeyboardProvider as KeyboardInputProvider } from '@services/input/keyboard.provider';
import { GamepadProvider as GamepadInputProvider } from '@services/input/gamepad.provider';
import { TauriIpcProvider as MockTauriInputProvider } from '@services/input/tauri-ipc.provider';
import { InputManager } from '@services/input/input.manager';
import { SpatialNavigatorService as EmuBoxSpatialNavigator } from '@services/navigation/spatial-navigator.service';
import { MockBackendService as MockBackend } from '@services/backend/mock-backend.service';
import { TauriBackendService as TauriBackend } from '@services/backend/tauri-backend.service';
import { GameService } from '@services/games/game.service';
import { EmulatorService } from '@services/emulators/emulator.service';
import { SystemService } from '@services/system/system.service';
import { PathService } from '@services/system/path.service';
import { StorageService } from '@services/storage/storage.service';
import { ProcessService } from '@services/process/process.service';
import { BiosScannerService } from '@services/bios/bios-scanner.service';
import { UpdateService } from '@services/update/update.service';
import { EmuBoxError } from '@contracts/errors.types';

import type { InputAction } from '@contracts/input.types';
import type { Game, Emulator } from '@contracts/game.types';
import games10000 from '@data/games-10000.json';

console.log("===============================================================================");
console.log("   EMUBOX: SUITE DE PRUEBAS DE ARQUITECTURA, SERVICIOS Y CONTRATOS             ");
console.log("===============================================================================\n");

let passed = 0;
let failed = 0;

function assert(condition: boolean, message: string) {
  if (condition) {
    console.log(`  ✓ [PASS] ${message}`);
    passed++;
  } else {
    console.error(`  ✗ [FAIL] ${message}`);
    failed++;
  }
}

// ----------------------------------------------------------------------------
// TEST 1: Abstracción de InputProviders e InputManager
// ----------------------------------------------------------------------------
console.log("TEST 1: Abstracción de InputProviders e InputManager...");
const inputManager = new InputManager();
const keyboardProvider = new KeyboardInputProvider();
const gamepadProvider = new GamepadInputProvider();
const tauriProvider = new MockTauriInputProvider();

inputManager.registerProvider(keyboardProvider);
inputManager.registerProvider(gamepadProvider);
inputManager.registerProvider(tauriProvider);

let receivedActions: InputAction[] = [];
const unhookAction = inputManager.onAction((action) => {
  receivedActions.push(action);
});

assert(inputManager.getActiveStatus().isConnected === true, "InputManager reporta estado de conexión activo");

// Inyectar acción desde Tauri IPC
tauriProvider.injectTauriEvent('BUTTON_A');
tauriProvider.injectTauriEvent('BUTTON_X');
tauriProvider.injectTauriEvent('BUTTON_START');

assert(receivedActions.length === 3, "InputManager recibió las 3 acciones semánticas de Tauri IPC");
assert(receivedActions[0] === 'BUTTON_A', "Acción 1 es BUTTON_A (Select/Play)");
assert(receivedActions[1] === 'BUTTON_X', "Acción 2 es BUTTON_X (Favorite)");
assert(receivedActions[2] === 'BUTTON_START', "Acción 3 es BUTTON_START (Menu)");
unhookAction();

// ----------------------------------------------------------------------------
// TEST 2: Backend Desacoplado & Operaciones sobre 10.000 Juegos
// ----------------------------------------------------------------------------
console.log("\nTEST 2: Backend Desacoplado & Operaciones sobre 10.000 Juegos...");
const mockBackend = new MockBackend(games10000 as Game[]);
const backend = new TauriBackend(mockBackend);

// Filtrado de 10.000 items
const t0 = performance.now();
const snesGames = await backend.getGames({ platform: 'snes' });
const filterTime = performance.now() - t0;

assert(snesGames.length === 1238, `Filtrado por plataforma SNES obtenido: ${snesGames.length} juegos`);
assert(filterTime < 15, `Filtrado sobre 10k items completado en ${filterTime.toFixed(2)}ms (<15ms)`);

// Búsqueda
const searchResults = await backend.getGames({ search: 'Chrono' });
assert(searchResults.length > 0, `Búsqueda de 'Chrono' encontrada: ${searchResults.length} resultados`);

// Toggle Favorito
const firstGameId = snesGames[0].id;
const newFavStatus = await backend.toggleFavorite(firstGameId);
assert(typeof newFavStatus === 'boolean', `Toggle favorito completado con estado: ${newFavStatus}`);

// Launch Game
const launchResult = await backend.launchGame(firstGameId, 'snes9x');
assert(launchResult.success === true && !!launchResult.pid, `Lanzamiento de juego simulado con éxito (PID: ${launchResult.pid})`);

// ----------------------------------------------------------------------------
// TEST 3: Nuevos Contratos de Backend (Configuración, Sistema, CRUD Emuladores)
// ----------------------------------------------------------------------------
console.log("\nTEST 3: Nuevos Contratos de Backend (Configuración, Sistema, CRUD Emuladores)...");

// System Info
const sysInfo = await backend.getSystemInfo();
assert(sysInfo.architecture === 'x86_64' && sysInfo.hardware.cpuCores > 0, `SystemInfo obtenido: ${sysInfo.hardware.cpuCores} cores, Arch: ${sysInfo.architecture}`);

// First Run Detection
const firstRun = await backend.runFirstRunDetection();
assert(firstRun.vulkanSupported === true && firstRun.configGenerated === true, `First Run Detection validado: Vulkan=${firstRun.vulkanSupported}`);

// EmuBoxConfig (Versioned)
const config = await backend.getConfig();
assert(config.version === 1 && typeof config.paths.roms === 'string', `EmuBoxConfig versionado (v${config.version}) leído correctamente`);
assert(config.updates.autoUpdate === true, `EmuBoxConfig incluye configuración de Auto-Update activo`);

config.audio.volume = 90;
await backend.saveConfig(config);
const updatedConfig = await backend.getConfig();
assert(updatedConfig.audio.volume === 90, `EmuBoxConfig guardado y persistido correctamente (Volumen: ${updatedConfig.audio.volume}%)`);

// Gamepad Status
const padStatus = await backend.getGamepadStatus();
assert(padStatus.connectedCount >= 1 && padStatus.devices.length >= 1, `GamepadStatus consultado: ${padStatus.connectedCount} dispositivos activos`);

// Emulators CRUD
const initialEmus = await backend.getEmulators();
const initialCount = initialEmus.length;
const testEmu: Emulator = {
  id: 'test-core-duck',
  name: 'DuckStation Test Core',
  version: '2.0.0',
  supportedPlatforms: ['ps1'],
  coreType: 'standalone',
  status: 'active',
  executable: 'duckstation-qt',
  arguments: ['-fullscreen']
};

await backend.saveEmulator(testEmu);
const afterAdd = await backend.getEmulators();
assert(afterAdd.length === initialCount + 1, `CRUD Emuladores: Motor añadido con éxito (${afterAdd.length} emuladores)`);

await backend.deleteEmulator('test-core-duck');
const afterDel = await backend.getEmulators();
assert(afterDel.length === initialCount, `CRUD Emuladores: Motor eliminado con éxito (${afterDel.length} emuladores)`);

// ----------------------------------------------------------------------------
// TEST 4: Capa de Servicios de Dominio (Domain Services Layer)
// ----------------------------------------------------------------------------
console.log("\nTEST 4: Capa de Servicios de Dominio (Domain Services Layer)...");
const gameService = new GameService(backend);
const emulatorService = new EmulatorService(backend);
const systemService = new SystemService(backend);
const storageService = new StorageService(backend);
const processService = new ProcessService(backend);
const pathService = new PathService();
const biosScanner = new BiosScannerService();
const updateService = new UpdateService(backend);

const platGames = await gameService.getGamesForPlatform('snes');
assert(platGames.length === 1238, `GameService filtró juegos por plataforma correctamente (${platGames.length} juegos)`);

const emusForPs1 = await emulatorService.getEmulatorsForPlatform('ps1');
assert(emusForPs1.length > 0, `EmulatorService devolvió emuladores para PS1 (${emusForPs1[0].name})`);

const hwInfo = await systemService.getHardwareInfo();
assert(hwInfo.gpuVendor === 'amd' || hwInfo.gpuVendor === 'generic', `SystemService devolvió GPU Vendor: ${hwInfo.gpuVendor}`);

const storageInfo = await storageService.getStorageInfo();
assert(storageInfo.drives.length > 0, `StorageService reportó unidades montadas: ${storageInfo.drives[0].name}`);

const isRunning = await processService.isGameRunning();
assert(typeof isRunning === 'boolean', `ProcessService comprobó estado de proceso: running=${isRunning}`);

const romPath = pathService.getRomsDir('ps2');
assert(romPath === '~/.local/share/emubox/roms/ps2', `PathService resolvió ruta XDG: ${romPath}`);

const biosStatus = await biosScanner.scanBios();
assert(biosStatus.totalRequired > 0, `BiosScannerService detectó ${biosStatus.totalRequired} BIOS requeridas`);

// Error model check
const err = new EmuBoxError('EmulatorNotInstalled', 'PCSX2 no encontrado');
assert(err.code === 'EmulatorNotInstalled' && err.message.includes('PCSX2'), `EmuBoxError estructurado validado`);

// ----------------------------------------------------------------------------
// TEST 5: Actualización Desacoplada OTA & Auto-Update
// ----------------------------------------------------------------------------
console.log("\nTEST 5: Actualización Desacoplada OTA & Auto-Update...");
const updateCheck = await updateService.checkForUpdates('stable');
assert(updateCheck.updateAvailable === true && updateCheck.targetVersion === 'v1.0.1', `OTA Check detectó actualización disponible (${updateCheck.targetVersion})`);

const updateProgress = await updateService.applyUpdate('v1.0.1');
assert(updateProgress.stage === 'ready_to_restart' && updateProgress.percent === 100, `OTA Apply ejecutó instalación atómica en /opt/emubox/releases/`);

// ----------------------------------------------------------------------------
// TEST 6: Motor de Navegación Espacial 2D y Jerarquía de Contenedores
// ----------------------------------------------------------------------------
console.log("\nTEST 6: Motor de Navegación Espacial 2D y Jerarquía de Contenedores...");
const navigator = new EmuBoxSpatialNavigator();

// Registrar nodos en un grid simulado de 7 columnas
for (let row = 0; row < 4; row++) {
  for (let col = 0; col < 7; col++) {
    const idx = row * 7 + col;
    navigator.register({
      id: `game-card-${idx}`,
      containerId: 'library',
      rect: {
        x: col * 180,
        y: row * 260,
        left: col * 180,
        top: row * 260,
        width: 160,
        height: 240
      },
      priority: 1
    });
  }
}

// Registrar botones del modal (contenedor aislado)
navigator.register({
  id: 'btn-play-game',
  containerId: 'modal',
  rect: { x: 500, y: 400, left: 500, top: 400, width: 200, height: 50 },
  priority: 10
});
navigator.register({
  id: 'btn-fav-game',
  containerId: 'modal',
  rect: { x: 500, y: 470, left: 500, top: 470, width: 200, height: 50 },
  priority: 10
});

// Foco inicial
navigator.setFocus('game-card-0');
assert(navigator.getCurrentFocusId() === 'game-card-0', "Foco inicial asignado al primer ítem del grid");

// Mover a la derecha en la misma fila
navigator.navigate('NAV_RIGHT');
assert(navigator.getCurrentFocusId() === 'game-card-1', `D-pad Derecha -> ${navigator.getCurrentFocusId()}`);

// Mover abajo (salto de fila en grid)
navigator.navigate('NAV_DOWN');
assert(navigator.getCurrentFocusId() === 'game-card-8', `D-pad Abajo -> ${navigator.getCurrentFocusId()} (fila siguiente)`);

// Cambiar de contenedor (abrir modal)
navigator.pushContainer('modal', true);
assert(navigator.getCurrentFocusId() === 'btn-play-game', "Foco atrapado en el primer botón del modal (btn-play-game)");

// Navegar dentro del modal
navigator.navigate('NAV_DOWN');
assert(navigator.getCurrentFocusId() === 'btn-fav-game', `Navegación vertical dentro del modal -> ${navigator.getCurrentFocusId()}`);

// Restaurar foco al contenedor de biblioteca tras cerrar modal
navigator.popContainer();
assert(navigator.getCurrentFocusId() === 'game-card-8', "Foco restaurado automáticamente a la tarjeta previa (game-card-8)");

// ----------------------------------------------------------------------------
// RESUMEN FINAL
// ----------------------------------------------------------------------------
console.log("\n===============================================================================");
console.log(`   RESULTADO: ${passed} PRUEBAS PASADAS / ${failed} FALLADAS                   `);
console.log("===============================================================================\n");

if (failed > 0) {
  process.exit(1);
}
