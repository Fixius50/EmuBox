# EmuBox Requirements & Stack Specification

## 1. Tríada Arquitectónica Inmutable de UI

> ### 📌 Regla de Oro del Proyecto
> * **Kobalte** controla cómo **funciona** un componente (comportamiento, accesibilidad WAI-ARIA, trampas de foco en modales).
> * **CSS propio** controla cómo **se ve** (identidad visual 100% Obsidian/Neón, unidades relativas, cero frameworks prediseñados).
> * **SolidJS** controla cómo **reacciona** (reactividad de grano fino sin Virtual DOM, señales directas, 0 re-renders innecesarios, 120 FPS).
>
> *Esta separación garantiza soberanía visual absoluta: **EmuBox jamás dependerá del diseño predeterminado de ninguna librería de componentes**.*

---

## 2. Stack Tecnológico Definitivo

* **Framework Frontend**: **SolidJS 1.9** (Reactividad de grano fino sin Virtual DOM, bajo consumo de memoria y máximo rendimiento para 60-120 FPS).
* **Primitivas Headless**: **Kobalte (`@kobalte/core`)** (Accesibilidad WAI-ARIA, trampas de foco en modales, Tabs, Switches y Sliders sin estilos CSS impuestos).
* **Diseño Visual**: **CSS Propio** (100% unidades relativas `rem`, `em`, `clamp()`, `%`, `vh`, `vw`, modularizado por zonas en `solid/src/styles/`).
* **Lógica y Contratos**: **TypeScript 5.7+** (Tipado estricto de dominio e interfaces desacopladas para Input y Backend).
* **Runtime de Escritorio**: **Tauri v2** (IPC Rust con ultra-baja latencia y soporte de mandos mediante `gilrs`).
* **Plataforma Objetivo**: **Arch Linux** (Modo dedicado DRM/KMS + compositor Wayland compatible; Vulkan no obligatorio).

---

## 3. Requisitos de Experiencia de Consola (10-Foot UI)

1. **Rueda 3D de Consolas**:
   - Scroll horizontal infinito de plataformas (PS1, PS2, N64, SNES, Genesis, GBA, Dreamcast, Arcade...).
   - Sin cajas / cards: Logos tipográficos y emblemas flotantes con iluminación reactiva de marca.
2. **Catálogo de Juegos por Sistema**:
   - Despliegue de juegos filtrados por el sistema seleccionado.
   - Navegación indexada determinista con autodesplazamiento virtual sobre 10.000 títulos.
3. **Selector de Emulador y Núcleo**:
   - Diálogo modal con Kobalte para elegir el motor de emulación (*Standalone* vs *Libretro Core*) y los argumentos de renderizado.
4. **Sintetizador Web Audio API**:
   - Efectos de sonido procedurales para navegación, confirmación y cancelación sin archivos de audio pesados.

## 4. Estado de la fase de juegos

La fase de catálogo e integración tiene implementados los siguientes componentes;
no equivale a cobertura universal de fuentes, formatos o juegos:

- SQLite y el escaneo alimentan el catalogo; una biblioteca vacia no se sustituye por datos de demostracion;
- las tarjetas exponen los metadatos disponibles, sin inventar año o valoración;
- la navegación XMB monta las filas próximas a la selección sobre el catálogo real;
- `DESCARGAR` y `JUGAR` dependen del estado instalado;
- `downloads/service/` conserva catálogo, plataforma, repositorio y orquestación;
- el resolver clasifica cada URI y comprueba proveedor y conector antes de ofrecerla;
- `downloads/connectors.rs` implementa Pixeldrain público y 1fichier con cuenta opt-in;
- `downloads/providers/` implementa HTTP y BitTorrent mediante aria2;
- `downloads/manager/` gestiona cola, controles, transferencia, verificación y publicación;
- preparación e instaladores locales distinguen `downloaded` de `completed`;
- el watcher y el escaneo actualizan SQLite cuando aparece una ROM;
- el selector de emulador y `CompatibilityService` resuelven el lanzamiento.

El JSON de catálogo aporta metadatos y localizadores, no certifica permisos,
seguridad ni disponibilidad remota. `DownloadService` permanece como fachada
compatible, no como implementación monolítica. `downloadable` indica una ruta
de proveedor compatible para la fuente seleccionada; HTTP puede fallar por
403/404, HTML, límites o red, y BitTorrent necesita metadata y peers/seeds.
Abrir una ficha o importar un manifiesto no inicia una transferencia.

El flujo público no requiere cuentas de EmuBox ni suscripciones. Los conectores
con cuenta están desactivados por defecto. No se evaden restricciones de terceros.
Quedan pendientes otros hostings, instaladores distintos de Inno, validación
completa de PKG PS3, descriptores multidisco y compatibilidad real por juego.
La preparación correcta no sustituye esa validación funcional.

Referencias vigentes: [estado actual](../architecture/current-state.md) y
[cobertura de proveedores](../architecture/download-providers.md).
