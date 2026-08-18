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
* **Plataforma Objetivo**: **Arch Linux** (Modo dedicado DRM/KMS + Gamescope Compositor).

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
