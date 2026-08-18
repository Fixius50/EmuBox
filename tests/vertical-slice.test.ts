import { KeyboardProvider as KeyboardInputProvider } from '@services/input/keyboard.provider';
import { GamepadProvider as GamepadInputProvider } from '@services/input/gamepad.provider';
import { TauriIpcProvider as MockTauriInputProvider } from '@services/input/tauri-ipc.provider';
import { InputManager } from '@services/input/input.manager';
import { SpatialNavigatorService as EmuBoxSpatialNavigator } from '@services/navigation/spatial-navigator.service';
import { MockBackendService as MockBackend } from '@services/backend/mock-backend.service';
import { TauriBackendService as TauriBackend } from '@services/backend/tauri-backend.service';

import type { InputAction } from '@contracts/input.types';
import type { Game } from '@contracts/game.types';
import games10000 from '@data/games-10000.json';

console.log("===============================================================================");
console.log("   EMUBOX FASE 3: SUITE DE PRUEBAS DEL VERTICAL SLICE (SOLID + KOBALTE)        ");
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
// TEST 3: Motor de Navegación Espacial 2D y Jerarquía de Contenedores
// ----------------------------------------------------------------------------
console.log("\nTEST 3: Motor de Navegación Espacial 2D y Jerarquía de Contenedores...");
const navigator = new EmuBoxSpatialNavigator();

// Registrar nodos en un grid simulado de 7 columnas
for (let row = 0; row < 4; row++) {
  for (let col = 0; col < 7; col++) {
    const idx = row * 7 + col;
    navigator.register({
      id: `game-card-${idx}`,
      containerId: 'library',
      rect: { x: col * 180, y: row * 240, width: 160, height: 220 }
    });
  }
}

// Registrar botones en contenedor modal
navigator.register({
  id: 'btn-play-game',
  containerId: 'modal',
  rect: { x: 500, y: 400, width: 200, height: 50 }
});
navigator.register({
  id: 'btn-fav-game',
  containerId: 'modal',
  rect: { x: 500, y: 460, width: 200, height: 50 }
});

navigator.setFocus('game-card-0');
assert(navigator.getCurrentFocusId() === 'game-card-0', "Foco inicial asignado al primer ítem del grid");

// Movimiento Derecha
navigator.move('NAV_RIGHT');
assert(navigator.getCurrentFocusId() === 'game-card-1', "D-pad Derecha -> game-card-1");

// Movimiento Abajo
navigator.move('NAV_DOWN');
assert(navigator.getCurrentFocusId() === 'game-card-8', "D-pad Abajo -> game-card-8 (fila siguiente)");

// Push Container: Modal
navigator.pushContainer('modal', true);
assert(navigator.getActiveContainerId() === 'modal', "Contenedor activo cambiado a 'modal'");

// El foco en modal debe atraparse en el primer elemento interactivo
assert(navigator.getCurrentFocusId() === 'btn-play-game', "Foco atrapado en el primer botón del modal (btn-play-game)");

// Navegación dentro del modal
navigator.move('NAV_DOWN');
assert(navigator.getCurrentFocusId() === 'btn-fav-game', "Navegación vertical dentro del modal -> btn-fav-game");

// Pop Container: Restaurar librería y foco previo
navigator.popContainer();
assert(navigator.getActiveContainerId() === 'library', "Contenedor restaurado a 'library'");
assert(navigator.getCurrentFocusId() === 'game-card-8', "Foco restaurado automáticamente a la tarjeta previa (game-card-8)");

console.log("\n===============================================================================");
console.log(`   RESULTADO: ${passed} PRUEBAS PASADAS / ${failed} FALLADAS                   `);
console.log("===============================================================================\n");

if (failed > 0) {
  process.exit(1);
} else {
  process.exit(0);
}
