# 🎮 EmuBox: Frontend de Consola Dedicada para Arch Linux

EmuBox es una interfaz de usuario cinematográfica de 10 pies (10-Foot UI) para consolas de emulación dedicadas bajo **Arch Linux (Direct DRM/KMS + Gamescope Compositor)**.

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
permite sincronizar sin abrir la UI. La descarga empieza al pulsar una tarjeta.
Actualmente se admiten archivos HTTP directos; magnet/torrent necesitan un motor
BitTorrent aun no implementado. Una fuente publicada no garantiza su seguridad
ni disponibilidad. Ver [conexion del catalogo](docs/architecture/catalog-sources.md).
La arquitectura vigente está en
[Estado actual de EmuBox](docs/architecture/current-state.md).

---

## 🏛️ El Cuarteto Arquitectónico Inmutable

> 🧠 **SolidJS 1.9** = **Cerebro** (Reactividad nativa de granularidad fina, stores y ciclo de vida).  
> 🕹️ **Kobalte** = **Comportamiento** (Primitivas headless, accesibilidad de consola y focus traps).  
> 🎨 **CSS Propio** = **Apariencia** (100% Unidades relativas `rem`/`em`/`clamp()`, Obsidian & Neon glow, 60/120 FPS).  
> ✨ **Anime.js** = **Movimiento** (Coreografía compleja, entradas escalonadas *stagger*, transiciones entre vistas).

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

## 🚀 Arquitectura en Capas Limpia

```text
solid/src/
├── types/         # @contracts/*  -> Interfaces TypeScript (Game, Platform, Emulator, InputAction)
├── services/      # @services/*   -> SoundFx, Backend Mock/Tauri, InputManager, SpatialNavigator
├── stores/        # @stores/*     -> LibraryStore, SystemStore, NavigationStore, ModalStore
├── hooks/         # @hooks/*      -> useConsoleInput, useConsoleNavigation, useGameLauncher
├── animations/    # @animations/* -> screen-transitions, shelf-animations, modal-animations
├── components/    # @components/* -> Layout (Shell, Header), Wheel 3D, ShelfGrid, Modals
├── styles/        # @styles/*     -> CSS modular por zonas (100% relativo)
└── App.tsx        # Orquestador raíz declarativo
```

---

## 🎮 Navegación en 3 Niveles de Consola

1. **Nivel 1: Rueda 3D de Consolas (`PlatformWheel.tsx`)**: Carrusel de logos y emblemas flotantes continuos con iluminación dinámica de marca (sin cards genéricas).
2. **Nivel 2: Catálogo del Sistema (`PlatformGamesView.tsx`)**: Hero banner cinemático superior y estante virtualizado (*TanStack Virtual*) con entrada escalonada (*Anime.js Stagger*).
3. **Nivel 3: Selector de Motor de Emulación (`EmulatorSelectorModal.tsx`)**: Diálogo Kobalte para elegir entre binarios Standalone (Vulkan) o núcleos Libretro antes del arranque DRM/KMS.

---

## 🖥️ Pipeline Gráfico y Sincronización de Resolución Dinámica

EmuBox implementa una arquitectura gráfica desacoplada y adaptativa:

```text
                                 EMUBOX OS
                                     │
                     ┌───────────────┴───────────────┐
                     ▼                               ▼
               VULKAN + DRM + GAMESCOPE          SIN ESTAS CAPACIDADES
                  (independiente de CPU)          (no implica solo CPU)
                     │                               │
                     ▼                               ▼
                 GAMESCOPE                          CAGE
                     │                               │
                     └───────────────┬───────────────┘
                                     ▼
                                 EMUBOX UI
                     (Tauri v2 + WebKitGTK + SolidJS)
```

* **Gamescope**: `Gamescope -> EmuBox` con Vulkan hardware, DRM y Gamescope instalado, tanto en x86_64 como en aarch64.
* **Cage**: `Cage -> EmuBox` cuando falta alguna de esas capacidades. Puede usar aceleración OpenGL aunque no haya Vulkan.
* **Sincronización Event-Driven (`emubox-drm-sync`)**: Escucha eventos nativos del kernel Linux (`SUBSYSTEM=drm`, `HOTPLUG=1`) mediante `udevadm` (0% CPU, sin polling). Al redimensionar la ventana o cambiar de monitor, adapta la superficie Wayland y la UI SolidJS en caliente sin resoluciones fijas ni reinicios.

## Documentación y Guías de Arquitectura

- [Requisitos](docs/specification/requirements.md), [diseño](docs/specification/design.md) y [matriz de primitivas](docs/specification/headless_primitives_matrix.md).
- [Aceleración 3D en VirtualBox](docs/architecture/virtualbox-graphics.md).
- [Arquitectura de Arranque Autónomo y Sesión Wayland](docs/architecture/console-appliance-boot-architecture.md)
- [Guía de Arquitectura, Refactorización y Estilo de Código](docs/architecture/refactoring-and-architecture-guidelines.md)
- [Contratos de Backend y Servicios de Dominio](docs/architecture/backend-contracts.md)
- [Convención de Archivos y Rutas XDG](docs/architecture/filesystem-convention.md)
- [Especificación IPC Tauri/Rust](docs/architecture/tauri-rust-ipc-spec.md)
- [Reportes de Diagnóstico de Entorno](docs/diagnostic-reports.md)
- [Estado actual de EmuBox](docs/architecture/current-state.md)

---

## 🛠️ Administración Remota por SSH

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
* **Usuario por defecto**: `emubox`
* **Contraseña interna**: `1234`

> 📌 **Aislamiento Garantizado**: La sesión SSH se abre en un pseudo-terminal (`pts/*`), por lo que puedes conectarte, compilar, administrar y cerrar la sesión SSH sin interrumpir la interfaz gráfica que sigue ejecutándose en la pantalla.

### 3. Acceso sin Contraseña para Agentes/Automatización (Clave Pública)
Para permitir que un agente (Copilot, scripts CI, etc.) opere sobre la VM sin depender de contraseñas interactivas ni exponerlas, autoriza una clave pública dedicada:

1. **En tu equipo anfitrión**, genera un par de claves dedicado (si no existe):
   ```bash
   ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519_emubox -N "" -C "agente-emubox-vm"
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

> ⚠️ **Nota de seguridad**: Si tras varios intentos fallidos de contraseña aparece `Permission denied` de forma persistente, revisa `pam_faillock` (bloqueo temporal por intentos fallidos) con `sudo journalctl -u sshd -n 50`, no necesariamente es una contraseña incorrecta.

---

## 🐙 Configuración de Git y Flujo de Trabajo

Para trabajar sobre el código fuente directamente en la máquina o enviar contribuciones:

### 1. Configurar Identidad de Git
```bash
git config --global user.name "Tu Nombre o Usuario"
git config --global user.email "tu-email@ejemplo.com"
```

### 2. Configurar Autenticación SSH con GitHub
Comprobar si existe clave SSH y verificar acceso:
```bash
ssh -T git@github.com
```
Asegurar que el repositorio remoto apunta a la URL SSH oficial:
```bash
cd /opt/emubox
git remote set-url origin git@github.com:Fixius50/EmuBox.git
```

### 3. Ciclo Típico de Actualización y Despliegue
```bash
cd /opt/emubox
git pull
bash scripts/build.sh
```

---

## 🧭 Centro de Control Interactivo (`script.sh`)

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

## 🧪 Pruebas y Compilación Manual

```bash
# Ejecutar suite de pruebas de arquitectura y contratos (42 pruebas automatizadas)
npm test

# Compilar bundle de producción para SolidJS
npm run build

# Iniciar entorno de desarrollo web interactivo
npm run dev
```