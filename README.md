# 🎮 EmuBox: Frontend de Consola Dedicada para Arch Linux

EmuBox es una interfaz de usuario cinematográfica de 10 pies (10-Foot UI) para consolas de emulación dedicadas bajo **Arch Linux (Direct DRM/KMS + Gamescope Compositor)**.

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

## Documentación y Guías de Arquitectura

- [Arquitectura de Arranque Autónomo y Sesión Wayland](docs/architecture/console-appliance-boot-architecture.md)
- [Guía de Arquitectura, Refactorización y Estilo de Código](docs/architecture/refactoring-and-architecture-guidelines.md)
- [Contratos de Backend y Servicios de Dominio](docs/architecture/backend-contracts.md)
- [Convención de Archivos y Rutas XDG](docs/architecture/filesystem-convention.md)
- [Especificación IPC Tauri/Rust](docs/architecture/tauri-rust-ipc-spec.md)
- [Reportes de Diagnóstico de Entorno](docs/diagnostic-reports.md)

---

## 🛠️ Administración Remota por SSH

EmuBox opera como una consola autónoma en pantalla física local (`tty1`). Para tareas de mantenimiento, desarrollo o diagnóstico desde otro PC, utiliza SSH:

### 1. Obtener la IP de la Consola / VM
En la máquina con EmuBox ejecuta:
```bash
ip addr  # o: hostname -I
```

### 2. Conectarse por SSH
Desde tu terminal de desarrollo:
```bash
ssh emubox@<IP_DE_LA_CONSOLA>
```
* **Usuario por defecto**: `emubox`
* **Contraseña interna**: `1234`

> 📌 **Aislamiento Garantizado**: La sesión SSH se abre en un pseudo-terminal (`pts/*`), por lo que puedes conectarte, compilar, administrar y cerrar la sesión SSH sin interrumpir la interfaz gráfica que sigue ejecutándose en la pantalla.

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
* **[1] Compilar EmuBox**: Ejecuta `scripts/build.sh` (npm ci, Vite, Tauri Rust).
* **[2] Actualizar desde GitHub**: Ejecuta `scripts/update-emubox.sh` (pull, build atómico y reinicio seguro).
* **[3] Configurar Autoarranque**: Ejecuta `scripts/setup-autostart.sh` (TTY1 autologin y lanzador adaptativo).
* **[4] Probar Lanzamiento**: Lanza la sesión Wayland en modo kiosko/consola.
* **[5] Ejecutar Tests**: Ejecuta los 42 tests de arquitectura y contratos.
* **[6] Iniciar Dev Server**: Inicia el servidor de desarrollo local de SolidJS.
* **[7] Diagnosticar Entorno**: Muestra el estado de GPU, Vulkan, DRM, systemd y logs.
* **[8] Reiniciar Consola Física**: Reinicia la sesión en `tty1`.
* **[9] Instalación Completa Arch Linux**: Ejecuta el aprovisionamiento de dependencias.

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