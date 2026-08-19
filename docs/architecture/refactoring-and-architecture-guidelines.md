# Guía de Arquitectura, Refactorización y Estilo de Código de EmuBox

Este documento define los estándares arquitectónicos, las reglas de refactorización y las convenciones de código para el desarrollo de la interfaz y servicios de EmuBox.

---

## 1. Separación Arquitectónica de Capas

### A. Capa Visual (Componentes de Presentación)
- Los componentes (`.tsx`) deben ser **estrictamente de presentación**.
- No deben contener lógica de negocio, cálculos trigonométricos complejos, polling de dispositivos, orquestación de actualizaciones OTA ni temporizadores empotrados.
- Deben limitarse a:
  - Recibir `props` tipadas desde interfaces en `@contracts/...` (`solid/src/types/`).
  - Renderizar JSX mediante primitivas de SolidJS (`<Show>`, `<For>`, `<Switch>`, `<Match>`).
  - Invocar los callbacks provistos por los hooks o props.

### B. Capa Lógica (Custom Hooks)
- Toda lógica que involucre estado (`createSignal`, `createMemo`, `createEffect`), suscripciones a eventos de ventana, cálculo de geometría 3D, controladores de CRUD o telemetría de hardware debe encapsularse en **Custom Hooks puros** dentro de `solid/src/hooks/`.
- Ejemplos del catálogo actual:
  - `useGamepadDevices`: Polling y detección de mandos en puertos 1-4.
  - `useOtaUpdate`: Gestión de comprobación, descarga y aplicación atómica de actualizaciones.
  - `useEmulatorCrud`: Estado de formulario, validación y navegación espacial en modales de configuración.
  - `useMaintenanceController`: Control y ejecución de acciones de rescate del sistema.
  - `usePlatformWheelLayout`: Trigonometría 3D y ordenación de slots del carrusel de consolas.
  - `useSettingsController`: Orquestación global de ajustes, VSync, rendimiento y audio.
  - `useConsoleNavigation`: Despachador de navegación espacial con mando y teclado.
  - `useConsoleInput`: Suscripción unificada a fuentes de entrada.

### C. Capa de Contratos e Interfaces (`solid/src/types/`)
- **Prohibido declarar interfaces o tipos complejos inline** dentro de archivos de componentes o hooks si representan modelos de datos o contratos compartidos.
- Todo tipo o interface debe residir en su módulo correspondiente en `solid/src/types/`:
  - `common.types.ts`: Componentes atómicos comunes (`BadgeProps`, `ConsoleButtonProps`, `SettingCardRowProps`, etc.).
  - `settings.types.ts`: Ajustes, pestañas, mandos y opciones del controlador.
  - `wheel.types.ts`: Rueda de plataformas, slots cilíndricos y callbacks.
  - `modal.types.ts`: Modales de lanzamiento, selectores y formularios CRUD.
  - `game.types.ts`, `backend.types.ts`, `update.types.ts`, etc.

### D. Centralización y Reutilización de Componentes Comunes (`components/common/`)
- Cuando un patrón visual o interactivo se repita en más de un lugar, debe centralizarse en `solid/src/components/common/`:
  - `Badge`: Insignias de estado y chips (`highlight`, `boost`, `chip`, etc.).
  - `ConsoleButton`: Botones con indicador de botón de mando (`[A]`, `[B]`, `[X]`, etc.).
  - `SettingCardRow`: Filas estándar de opciones de configuración.

---

## 2. Reglas Internas de Estilo de Código

### A. Prioridad de `switch` y Ternarios sobre Cascadas de `if / else if`
- **Reemplazar cadenas extensas de `if (...) else if (...)`** por declaraciones `switch` estructuradas o ternarios concisos.
- **PROHIBICIÓN ESTRICTA**: Prohibido el uso de `switch (true)`.
- **PROHIBICIÓN ESTRICTA**: Prohibido dejar cláusulas `default:` vacías sin propósito; deben retornar, romper o lanzar un error de tipo exhaustivo de TypeScript de forma explícita.

### B. Invariante de Experiencia de Consola (10-Foot UI)
- La aplicación está diseñada para ser operada al 100% mediante un mando físico o teclado desde el sofá (10-foot interface).
- En campos de texto (`<input>`), separar siempre el **Modo Navegación** (foco espacial sin atrapar cursor nativo) del **Modo Escritura** (`.typing`), permitiendo salir limpiamente con `Enter`, `Escape` o flechas del D-pad.

### C. Invariante Estricta de Cero Emojis
- **PROHIBIDO EL USO DE EMOJIS** en toda la aplicación: código fuente, JSX, badges, etiquetas, botones, scripts de shell, logs de terminal y documentación.
- Emplear siempre tipografía limpia, insignias semánticas, acentos de color cian/esmeralda o iconos vectoriales SVG.

---

## 3. Matriz de Módulos y Estructura

```text
solid/src/
├── animations/         # Animaciones puras con Anime.js
├── components/
│   ├── common/         # Componentes atómicos reutilizables (Badge, ConsoleButton, SettingCardRow)
│   ├── emulators/      # Componentes de emuladores
│   ├── layout/         # Shell, Header
│   ├── library/        # Vistas de catálogo, ShelfGrid con TanStack Virtual
│   ├── modals/         # Modales desacoplados de selección y rescate
│   ├── platforms/      # Rueda 3D de plataformas
│   └── settings/       # Ajustes modularizados por pestañas (tabs/) y modales (modals/)
├── hooks/              # Custom hooks con la lógica pura desacoplada
├── services/           # Servicios de dominio, backend IPC, audio WebAudio, input Gilrs
├── stores/             # Stores reactivos de SolidJS
├── styles/             # CSS Vanilla puro con variables de diseño
└── types/              # Definiciones e interfaces TypeScript centralizadas
```
