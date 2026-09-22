# EmuBox: Frontend de Consola Dedicada para Arch Linux

EmuBox es una interfaz de usuario de 10 pies (10-Foot UI) para consolas de emulación dedicadas bajo **Arch Linux (DRM/KMS + compositor Wayland compatible)**.

## Estado actual

### Validacion de archivos abiertos y cerrados

`npm run verify` comprueba dependencias, TypeScript/TSX, cobertura, lint,
formato, arquitectura, texto y tests sobre los archivos guardados del proyecto.
No depende de las pestanas abiertas en VS Code. Los informes se guardan en
`reports/verify/` y el proceso termina con error si falla cualquier comprobacion.

Desde **Terminal > Ejecutar tarea**, seleccionar **EmuBox: validar proyecto completo**
para publicar diagnosticos TypeScript y ESLint en el panel **Problemas**.
La tarea **EmuBox: comprobar todos los TypeScript y JSX** ejecuta solo el compilador.

Si el editor indica que falta `solid-js/jsx-runtime`, pero Dependencias y TypeScript
pasan, ejecutar **TypeScript: Restart TS Server** desde la paleta de comandos.
El compilador comprueba disco; no reproduce errores de cache del editor ni cambios
sin guardar. Las especificaciones de inclusion/exclusion siguen en `tsconfig.json`.

### Runtime, appliance y distribucion

El objetivo actual es ejecutar el mismo runtime y la misma appliance EmuBox en
x86_64 y aarch64. La distribucion reproducible EmuBox OS es una fase posterior;
compilar el binario no demuestra que la appliance completa arranque.

La instalacion manual configura usuario `emubox`, filesystem, servicios, Wayland,
input y audio sin instalar emuladores obligatorios. Los motores se solicitan
aparte con `sudo bash installer/setup/emulator-packages.sh`.

`npm run test:appliance` ejecuta pruebas aisladas; `npm run check:appliance`
inspecciona el equipo sin modificarlo. El segundo devuelve requisitos fallidos
o aceptacion funcional pendiente, nunca certifica soporte ARM por si solo.
Ver [guia de aceptacion de la appliance](docs/architecture/appliance-validation.md).

### Arquitecturas nativas

El instalador admite Arch Linux (`ID=arch`) en **x86_64** y Arch Linux ARM
(`ID=archarm`) en **aarch64**. ARM32/armv7 y otros derivados no están admitidos.
Configura un sistema ya instalado: no particiona discos ni genera imágenes ISO.

`bash scripts/build.sh` compila como usuario sin privilegios, selecciona
`BUILD_ARCH` y `TARGET` nativos y genera `bin/emubox-linux-x86_64` o
`bin/emubox-linux-aarch64`. Valida el ELF antes de instalar `bin/emubox`.
No usa FEX, Box64, QEMU-user ni un frontend distinto para ARM.

**Validación:** build nativo x86_64 comprobado. El workflow Runtime Linux (native) incluye
runners nativos x86_64 y ARM64; su ejecución remota y el arranque gráfico sobre
ARM real están pendientes. Soportar el runtime ARM64 no garantiza disponibilidad
de todos los emuladores, drivers ni rendimiento suficiente para PS3.

La biblioteca muestra juegos reales registrados en SQLite desde el escaneo o
desde los manifiestos configurados; no incorpora el dataset de demostracion.
Las tarjetas muestran los metadatos disponibles y SVG de su categoria. El
catalogo persistido carga desde SQLite: cache de seis horas, peticiones HTTP
condicionales y actualizacion de entradas cambiadas. Los paquetes reconocidos
de un mismo titulo/plataforma se reunen en una tarjeta; sus fuentes y nombres
originales permanecen disponibles en el selector. Las consolas distintas no se mezclan.
El flujo distingue `DESCARGAR` de `JUGAR`; al terminar una descarga autorizada y
detectar la ROM, la biblioteca se actualiza. El lanzamiento requiere además un
emulador compatible con la CPU, su binario nativo y sus requisitos de hardware.

Los manifiestos se leen desde `/etc/emubox/download-links.txt` si contiene enlaces
activos; en caso contrario, desde `data/download-links.txt`. Se importan solo
metadatos y fuentes, nunca juegos automaticamente. `bin/emubox --import-catalog`
 permite sincronizar sin abrir la UI. Abrir una ficha no descarga: se debe elegir
una fuente y confirmar la accion. HTTP/HTTPS usa el proveedor nativo; magnet btih
y enlaces torrent directos usan qBittorrent-nox >= 5.0, si esta instalado.
Jackett local permite buscar candidatos desde la ficha y anadir una fuente
explicitamente; buscar no crea descargas ni juegos. Configuracion local en la
[guia de descargas](docs/architecture/download-providers.md#qbittorrent-y-jackett-locales).
Pixeldrain publico tiene conector HTTP; 1fichier es opcional, requiere cuenta
propia y permanece desactivado salvo opt-in explicito. Otros hostings necesitan
conectores compatibles; no se eluden login, CAPTCHA ni limites del servidor.
`downloadable` indica una ruta compatible, no una URL comprobada ni segura.
`downloaded` conserva contenido pendiente de preparacion o seleccion;
`completed` indica un candidato preparado, no compatibilidad garantizada del juego.
Ver [conexion del catalogo](docs/architecture/catalog-sources.md) y
[cobertura de fuentes y limites](docs/architecture/download-providers.md).
La arquitectura vigente está en
[Estado actual de EmuBox](docs/architecture/current-state.md).

---

## El Cuarteto Arquitectónico Inmutable

>  **SolidJS 1.9** = **Cerebro** (Reactividad nativa de granularidad fina, stores y ciclo de vida).  
>  **Kobalte** = **Comportamiento** (Primitivas headless, accesibilidad de consola y focus traps).  
> **CSS Propio** = **Apariencia** (unidades relativas `rem`/`em`, tokens y estilos por componente; rendimiento sujeto a medicion).  
>  **Anime.js** = **Movimiento** (Coreografía compleja, entradas escalonadas *stagger*, transiciones entre vistas).

```
   ┌─────────────────────────────────────────────────────────────┐
   │                     1. SOLIDJS 1.9                          │
   │  Reactividad pura, Signals, Stores, Orquestación de ciclo   │
   └──────────────────────────────┬──────────────────────────────┘
                                  │
   ┌──────────────────────────────▼──────────────────────────────┐
   │                     2. KOBALTE CORE                         │
   │  Comportamiento headless: Dialogs, Tabs, Switches, Sliders  │
   └──────────────────────────────┬──────────────────────────────┘
                                  │
   ┌──────────────────────────────▼──────────────────────────────┐
   │                     3. CSS PROPIO                           │
   │  100% Unidades relativas (rem, em, clamp), Obsidian/Neon   │
   └──────────────────────────────┬──────────────────────────────┘
                                  │
   ┌──────────────────────────────▼──────────────────────────────┐
   │                     4. ANIME.JS                             │
   │  Capa de movimiento: Stagger de tarjetas, fades, modales    │
   └─────────────────────────────────────────────────────────────┘
```

---

## Arquitectura en Capas Limpia

```text
solid/src/
├── types/         # @contracts/*  -> Interfaces TypeScript (Game, Platform, Emulator, InputAction)
├── services/      # @services/*   -> SoundFx, Tauri IPC, InputManager, SpatialNavigator
├── stores/        # @stores/*     -> LibraryStore, SystemStore, NavigationStore, ModalStore
├── hooks/         # @hooks/*      -> useXmbLibrary, useSettingsNavigation, useConsoleInput, useGameLauncher
├── animations/    # @animations/* -> Transiciones de ajustes, emuladores y modales
├── components/    # @components/* -> Biblioteca XMB, ajustes y modales
├── styles/        # @styles/*     -> CSS modular por zonas (100% relativo)
└── App.tsx        # Orquestador raíz declarativo
```

---

## Navegación XMB

1. **Categorías**: ajustes, todos los juegos, favoritos, instalados y plataformas del catálogo real.
2. **Carpetas y versiones**: cada carpeta canónica abre directamente las versiones reales, con contador y estado de instalación; las tarjetas se construyen bajo demanda.
3. **Ficha y acciones**: título canónico, versión seleccionada, identificación Libretro/local, favorito compartido y fuentes de esa versión. El emulador se elige para la variante instalada. Abrir una ficha no inicia descargas.

Flechas y D-pad navegan; Enter/A abre; Escape/B retrocede. LB/RB cambia categoría, X alterna favorito y Y enfoca búsqueda. En la ficha, LT/RT desplaza los detalles; en fuentes, Y permite jugar si el título instalado es compatible.

El CSS se organiza por componente, con medidas relativas, propiedades lógicas y movimiento reducido. Las reglas y controladores de navegación se prueban en Node. La aplicación instalada requiere Tauri; la previsualizacion de desarrollo descrita abajo no sustituye el backend nativo.

## Desarrollo de UI

`npm run dev` abre la previsualizacion web en `http://localhost:3001`, sin compilar
Tauri. Usa los componentes reales con juegos ficticios identificados como ejemplos.
Favoritos y ajustes viven solo en memoria y se reinician
al recargar. No contiene ROMs, fuentes de descarga ni cuentas reales.

La previsualizacion solo se habilita con `import.meta.env.DEV` fuera de Tauri.
Descargas, Jackett, lanzamiento de juegos y mantenimiento nativo permanecen
bloqueados. En Tauri se conserva el arranque real; en produccion, abrir la web
sin Tauri sigue mostrando el requisito de runtime nativo. `test:ui` y `verify:dev`
son comprobaciones y no arrancan la aplicacion.

Ajustes se abre en un unico contenedor bajo las categorias: bajar entra y subir
desde el primer control vuelve a la categoria. No hay acceso a Mantenimiento ni
pestana de gestion de emuladores; los motores se muestran como informacion en
Sistema y pantalla. Las opciones editables son Preferencia de rendimiento y VSync.

Audio general permite seleccionar entrada y salida entre los dispositivos que
expone el navegador, con `Automatico (sistema)` como valor inicial. No abre el
microfono ni pide permiso de captura; el navegador puede limitar los nombres y
dispositivos disponibles. En desarrollo, las selecciones son preferencias en
memoria: no cambian el dispositivo fisico ni la configuracion de PipeWire.
La aplicacion nativa consulta los dispositivos mediante `getAudioInfo`; aplicar
estas nuevas preferencias al enrutamiento nativo de audio queda pendiente.

---

## Pipeline Gráfico y Sincronización de Resolución Dinámica

EmuBox implementa una arquitectura gráfica desacoplada y adaptativa:

```text
                   Sondeos GPU y evidencias por dispositivo
                             |
                  +-------------------+-------------------+
                  |                   |                   |
                Acelerado           Software           Indeterminado
                OpenGL/Vulkan       confirmado         Auto, sin forzar CPU
                  |                   |                   |
                Compositor          Cage/Pixman        Cage automatico
                compatible                            o fallback explicito
                  +-------------------+-------------------+
                             |
                      Tauri + WebKitGTK + SolidJS
```

* **Backend**: OpenGL acelerado tiene preferencia para la sesión Cage/WebKitGTK; Vulkan acelerado es otra ruta disponible. Errores de sondeo no equivalen a ausencia de GPU: mantienen `indeterminate` y backend `auto`. Software requiere evidencia suficiente o fallback explícito ante incertidumbre.
* **Cage**: opción automática con DRM y OpenGL, o intento automático ante detección inconclusa; Pixman solo por software confirmado o fallback explícito.
* **Gamescope**: sus requisitos propios incluyen Vulkan acelerado, ejecutable y salida compatible. Se elige mediante `EMUBOX_COMPOSITOR_PREFERENCE=gamescope` si es compatible, o como alternativa cuando Cage no soporta el backend disponible. Detectar Vulkan no fuerza Gamescope. Sin compositor compatible se informa del error, no se oculta pasando a CPU.
* **GPU virtual**: SVGA3D/virgl con OpenGL y sin Vulkan permanece acelerada. CPU `aarch64` y GPU Mali/AMD/Intel/NVIDIA son datos independientes.
* **MultiGPU**: inventario y observaciones se correlacionan por identidad DRM. Sin correspondencia se informa `unknown`; monoGPU permite inferencia declarada. `selectedDeviceId` no es evidencia de `activeDeviceId`.
* **Sincronización Event-Driven (`emubox-drm-sync`)**: escucha eventos DRM mediante `udevadm`, sin polling continuo. La aplicacion de cambios depende de los protocolos y salidas del compositor; no garantiza coste cero ni compatibilidad universal de hotplug.

## Documentacion

El [indice documental](docs/README.md) distingue referencias vigentes, requisitos
y evidencia historica. No hay especificaciones IPC, PS3 o diseno duplicadas.
El [estado actual](docs/architecture/current-state.md) resume funciones y limites;
los informes de diagnostico conservan observaciones fechadas, no promesas de soporte.

---

## Administración Remota por SSH

EmuBox opera como una consola autónoma en pantalla física local (`tty1`). Para tareas de mantenimiento, desarrollo o diagnóstico desde tu PC anfitrión u otro equipo:

### 1. Conexión Local (Máquina Virtual / VirtualBox con Reenvío de Puertos)
Si estás desarrollando en local con VirtualBox (configuración NAT con Port Forwarding `2222 -> 22`):
```bash
ssh -p 2222 emubox@127.0.0.1
```

### 2. Conexión en Red Local / Hardware Físico (IP Directa)
Si la máquina o consola está conectada a tu red local (o adaptador puente):
```bash
ssh emubox@<IP_DE_LA_CONSOLA>
```
* Obtener IP en la máquina: `hostname -I` o `ip addr`
* **Usuario de la appliance**: `emubox`
* Configura una contraseña propia durante el aprovisionamiento o utiliza una clave
   SSH dedicada. No existe una contraseña pública recomendada para EmuBox.
* Si se utilizó una credencial de desarrollo publicada, cámbiala localmente con
   `passwd`. Retirarla de esta documentación no la revoca ni elimina copias antiguas.
   No envíes contraseñas ni claves privadas por chat.

> **Sesion independiente**: SSH usa un pseudo-terminal (`pts/*`) distinto de TTY1. Cerrar SSH no cierra por si solo la UI, pero comandos administrativos o cargas intensas pueden afectar a la sesion grafica.

### 3. Acceso sin Contraseña para Agentes/Automatización (Clave Pública)
Para permitir que un agente (Copilot, scripts CI, etc.) opere sobre la VM sin depender de contraseñas interactivas ni exponerlas, autoriza una clave pública dedicada:

1. **En tu equipo anfitrión**, genera un par de claves dedicado (si no existe):
   ```bash
   ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519_emubox -C "agente-emubox-vm"
   ```
2. **Desde una sesión ya autenticada por contraseña en la VM**, autoriza la clave pública generada:
   ```bash
   mkdir -p ~/.ssh && chmod 700 ~/.ssh
   echo "<contenido-de-id_ed25519_emubox.pub>" >> ~/.ssh/authorized_keys
   chmod 600 ~/.ssh/authorized_keys
   ```
3. **Conecta sin contraseña** usando la clave privada:
   ```bash
   ssh -p 2222 -i ~/.ssh/id_ed25519_emubox -o BatchMode=yes emubox@127.0.0.1
   ```

Protege la clave con una frase de paso introducida directamente en tu terminal
y cárgala en el agente SSH local antes de usar `BatchMode=yes`. Verifica una
segunda conexión con clave antes de desactivar cualquier método de acceso existente.

>  **Nota de seguridad**: Si tras varios intentos fallidos de contraseña aparece `Permission denied` de forma persistente, revisa `pam_faillock` (bloqueo temporal por intentos fallidos) con `sudo journalctl -u sshd -n 50`, no necesariamente es una contraseña incorrecta.

---

## Despliegue local

El flujo de trabajo actual no ejecuta Git ni pruebas de navegador. Con los cambios
locales preparados, validar y compilar como usuario sin privilegios:
```bash
cd /opt/emubox
npm run verify
bash scripts/build.sh
```

El usuario reinicia `getty@tty1` al terminar, no durante una descarga o partida.

---

## Centro de Control Interactivo (`script.sh`)

Para evitar tener que recordar y escribir comandos largos, dispones de un menú interactivo en la raíz del proyecto:

```bash
# Dar permisos y ejecutar el menú interactivo
chmod +x script.sh
./script.sh
```

El menú te permite seleccionar con un solo número:
* **[1] Compilar EmuBox**: Ejecuta `scripts/build.sh` (frontend + binario Tauri).
* **[2] Actualizar desde GitHub**: Ejecuta `scripts/update-emubox.sh` (pull, build y despliegue).
* **[3] Configurar Appliance**: Ejecuta `scripts/setup-autostart.sh` (permisos, autologin y servicios).
* **[4] Diagnosticar Entorno**: Muestra GPU, Vulkan, DRM, systemd y logs.
* **[5] Instalación Completa Arch Linux**: Ejecuta el aprovisionamiento inicial.

---

## Pruebas y Compilación Manual

Las pruebas portables usan el runner nativo de Node (Node 22 o superior, como en
CI) y el cargador `tsx` ya instalado. Cada archivo se ejecuta en un proceso
aislado, con un maximo de cuatro procesos simultaneos. Un fallo no impide obtener
el resultado del resto de archivos. Los casos XMB, ajustes y backend tienen
nombres propios y estado local; no dependen del orden de otros casos.

`npm test` descubre `tests/*.test.ts` e incluye explicitamente las pruebas
portables de cache y lectura de telemetria. Un nuevo test TypeScript en esa ruta
debe ser portable y no requerir hardware ni servicios reales. Las suites
`test:architecture` y `test:appliance` permanecen separadas porque necesitan
Bash/Linux. Las pruebas Rust y C siguen siendo nativas.

`test:ui` comprueba biblioteca, agrupacion, navegacion y ajustes con la
reactividad cliente de Solid. No abre un navegador, no sustituye Tauri y no
certifica la presentacion visual ni la appliance. No se generan builds.

```bash
# Bucle corto para cambios en logica de UI
npm run test:ui

# Un solo comportamiento dentro de un archivo
node --import tsx --conditions=browser --test --test-name-pattern="Settings: modal" tests/settings.test.ts

# Tests, lint, formato, arquitectura/contratos y texto, sin invocar el compilador
npm run verify:dev

# Seleccion explicita; las dependencias siempre se comprueban primero
npm run verify -- --only tests --only lint
```

`verify` sin argumentos mantiene todas las comprobaciones existentes. `--only`
admite `typecheck`, `validator`, `lint`, `format`, `architecture`, `text` y `tests`;
los nombres desconocidos se rechazan y los repetidos se ejecutan una sola vez.
Los informes en `reports/verify/` se actualizan solo para los controles ejecutados;
otros informes pueden pertenecer a ejecuciones anteriores. El resumen identifica
la seleccion y devuelve error si falla cualquiera de sus controles.

```bash
# Comprobar TypeScript; reutiliza cache incremental local si existe
npm run typecheck

# Ejecutar todas las pruebas portables de contratos y comportamiento
npm test

# Compilar bundle de producción para SolidJS
npm run build

# Iniciar entorno de desarrollo web interactivo
npm run dev
```