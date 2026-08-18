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

---

## 🧪 Pruebas y Compilación

```bash
# Ejecutar suite de pruebas de arquitectura (18 pruebas automatizadas)
npm test

# Compilar build optimizado de producción
npm run build

# Iniciar entorno de desarrollo local (http://localhost:3001)
npm run dev
```
