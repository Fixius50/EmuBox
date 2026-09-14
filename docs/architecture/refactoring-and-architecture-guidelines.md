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
  - `useEmulatorCrud`: Estado de formulario, validación y navegación espacial en modales de configuración.
  - `useMaintenanceController`: Control y ejecución de acciones de rescate del sistema.
  - `useXmbLibrary`: Estado, búsqueda, ventanas acotadas del catálogo y navegación de biblioteca.
  - `useSettingsController`: Cambios de ajustes, VSync, rendimiento y audio.
  - `useSettingsNavigation`: Navegación entre pestañas y controles de ajustes.
  - `useConsoleInput`: Suscripción unificada a fuentes de entrada.

### C. Capa de Contratos e Interfaces (`solid/src/types/`)
- **Prohibido declarar interfaces o tipos complejos inline** dentro de archivos de componentes o hooks si representan modelos de datos o contratos compartidos.
- Todo tipo o interface debe residir en su módulo correspondiente en `solid/src/types/`:
  - `common.types.ts`: Componentes atómicos comunes (`BadgeProps`, `ConsoleButtonProps`, `SettingCardRowProps`, etc.).
  - `settings.types.ts`: Ajustes, pestañas, mandos y opciones del controlador.
  - `xmb.types.ts`: Estado de navegación, carpetas y contrato de la biblioteca XMB.
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
│   ├── library/        # Presentación XMB; estado en useXmbLibrary
│   ├── modals/         # Modales desacoplados de selección y rescate
│   └── settings/       # Ajustes modularizados por pestañas (tabs/) y modales (modals/)
├── hooks/              # Custom hooks con la lógica pura desacoplada
├── services/           # Servicios de dominio, backend IPC, audio WebAudio, input Gilrs
├── stores/             # Stores reactivos de SolidJS
├── styles/             # CSS Vanilla puro con variables de diseño
└── types/              # Definiciones e interfaces TypeScript centralizadas
```

## 4. Biblioteca XMB y Propiedad de los Estilos

- `services/library/xmb-navigation.ts` contiene reglas puras de movimiento y agrupación visual. No modifica registros ni identificadores de descarga.
- `useXmbLibrary` calcula categorías, búsqueda y ventanas de hasta siete carpetas y cinco versiones. Un movimiento no reconstruye la agrupación si el catálogo y el filtro no cambian.
- `App` da prioridad a mantenimiento, fuentes, selector de emulador y pantalla activa. Los controladores se registran mediante callbacks tipados y se liberan al desmontar, sin puentes de navegación en `window`.
- `xmb.css` contiene solo la biblioteca. `settings.css`, `source-selector.css`, `emulator-selector.css` y `modals.css` son responsables de sus superficies; `console-hardware.css` conserva los iconos compartidos.
- Preferir `rem`, `em`, porcentajes, `dvh`/`dvw`, `minmax`, `aspect-ratio` y propiedades lógicas. La tipografía no se escala según el ancho de ventana. Respetar `prefers-reduced-motion` y el modo gráfico compatible.
- Antes de eliminar módulos, comprobar consumidores de la aplicación y de pruebas. Los servicios de dominio ejercitados por tests no son archivos huérfanos aunque no se importen directamente desde la UI.
- Las pruebas reactivas se ejecutan en Node con la condición de exportación `browser` de Solid; no requieren un navegador.

## 5. Reglas para Rust

- `commands/` es el adaptador IPC: recibe argumentos tipados, delega y devuelve `Result`. No posee consultas SQL, parsers de hardware, configuración de emuladores ni políticas de selección.
- `models/` contiene DTO, modelos compartidos, formatos de manifiesto y enums de dominio. La serialización `camelCase` se coordina con `solid/src/types/`; cualquier campo desconocido se representa con `Option`, no con telemetría inventada.
- `services/` contiene servicios por responsabilidad. Una fachada puede orquestar varios servicios, pero no debe duplicar su política ni su persistencia.
- La política gráfica pura reside en `graphics_policy`; inventario y sondeo, en `graphics_probe`; orquestación, en `graphics_service`; el DTO, en `models/graphics`. El lanzador consume la CLI nativa, sin mantener otro inventario Shell. CPU, GPU, API y compositor son dimensiones distintas. No correlacionar GPU por nombre ni convertir incertidumbre en software confirmado.
- `config_service` posee lectura, validación y escritura de configuración. Solo `NotFound` permite valores de fábrica; un error de permisos o de formato debe propagarse. Escritura temporal y renombrado evitan publicar JSON truncado.
- `input_service`, `display_service`, `audio_service`, `storage_service`, `bios_service`, `log_service` y `power_service` son responsables de sus propios datos y efectos.
- `host_command` centraliza comandos de sondeo con argumentos separados, locale estable y timeout. No construir comandos de shell para operaciones que puedan expresarse con argumentos tipados.
- Usar `match` sobre enums/tuplas para decisiones excluyentes. Las cláusulas de guarda deben expresar requisitos, no convertir falta de Vulkan en falta de GPU.
- Usar APIs estructuradas para JSON, SQLite y filesystem. No añadir una abstracción por cada función: extraer módulos cuando haya responsabilidad o reutilización real.
- No devolver `Ok(())`, listas vacías ni cifras fijas para representar una operación no implementada. Un resultado vacío solo es válido después de una consulta real.
- Operaciones costosas de disco, red o procesos no deben bloquear el hilo de presentación. Separar el adaptador async de las operaciones blocking al ampliar comandos existentes.
- Un método `save_*` debe indicar si persiste una preferencia o si también aplica el cambio al SO. No anunciar que audio, gráficos o energía se han aplicado si solo se guardó JSON.
- Las pruebas usan funciones reales y recursos temporales aislados. Nunca abren la SQLite de producción, ejecutan apagados, instalan paquetes ni descargan juegos como parte de la suite.
- Conservar contratos y comportamiento público durante una extracción mecánica. Ejecutar primero la prueba del módulo afectado y después controles del conjunto.

## 6. Runtime y Compilación

### Organizacion nativa por dominio

La primera fase de refactorizacion conserva los nombres publicos y comandos IPC.
`services/mod.rs` reexporta las fachadas anteriores para que los consumidores no
dependan del traslado fisico. Las implementaciones viven en estas carpetas:

```text
src-tauri/src/
  commands/                 Adaptadores IPC existentes
  models/                   DTO y modelos compartidos, incluido CatalogEntry
  services/
    downloads/
      service/              catalog, platform, repository, orchestration, tests
      manager/              cola en mod, transfer, publication, tests
      installers/           detection, firmware, launch, sandbox, validation, tests
      providers/            HTTP y BitTorrent
      archive.rs            Extraccion 7z/RAR
      preparation.rs        Verificacion SHA-256, ZIP y seleccion conservadora
      manifest.rs           Normalizacion
      manifest_cache.rs     Cache HTTP de manifiestos
      resolver.rs           Capacidad y seleccion de proveedor
      connectors.rs         Resolucion por alojamiento
    graphics/               policy, probe, service
    infrastructure/         database, paths, binary, host_command
    library/
      games/                scanner, repository, catalog, tests
      platforms.rs          Definiciones de plataformas
      compatibility.rs      Asociaciones juego-emulador
      watcher.rs            Notificaciones del filesystem
    runtime/                emulator, capabilities, process
    system/                 audio, display, input, power, config, storage, logs, bios
    emulators/              Perfiles individuales y alternativas Libretro
```

- El gestor mantiene un unico propietario de la cola; la publicacion coordina sus
  cambios con ese propietario. No se han creado colas adicionales por mover archivos.
- Los bloques `impl DownloadService` y `impl GameService` se distribuyen por
  responsabilidad. Son una transicion compatible, no una capa de repositorios
  independiente completa: el importador y el escaner todavia contienen SQL.
- Los recursos embebidos se localizan desde `CARGO_MANIFEST_DIR`, evitando que un
  nivel adicional de carpetas cambie el fichero de datos incluido.
- La apertura SQLite serializa configuracion WAL y esquema dentro del proceso;
  las consultas y transacciones posteriores no quedan bajo ese bloqueo. Se prueba
  con ocho conexiones concurrentes y bases temporales, nunca con produccion.
- `scripts/architecture-check.mjs` comprueba la presencia de los modulos canonicos;
  Cargo comprueba resolucion, visibilidad e importaciones. La comprobacion de rutas
  por si sola no demuestra ausencia de ciclos arquitectonicos.
- Pendientes de otra fase: reducir SQL y mapeos duplicados, retirar el importador
  legado inalcanzable tras normalizacion, revisar heuristicas de plataforma y errores
  silenciados, y completar extracciones de runtime/graficos donde aporten claridad.
  No se cambian esas politicas dentro de un traslado mecanico.

- La aplicación requiere Tauri; no existe un backend alternativo de datos ficticios ni un catálogo de demostración.
- Las funciones sin implementación nativa deben rechazar la operación o estar deshabilitadas explícitamente.
- No ejecutar comandos de Git ni pruebas de navegador en el flujo de trabajo actual.
- El build central reutiliza dependencias y frontend por huellas verificadas; Cargo conserva sus artefactos. No ejecutar `cargo clean` ni `npm ci` incondicionalmente.
- `EMUBOX_REINSTALL_DEPS=1` fuerza reinstalar dependencias y `EMUBOX_REBUILD_FRONTEND=1` fuerza Vite. Nunca omitir comprobación de arquitectura del ELF para ahorrar tiempo.
