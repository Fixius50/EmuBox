# EmuBox Design Specification (10-Foot Console UI)

## 1. Tríada Arquitectónica Inmutable de UI

> ### 📌 Regla de Oro del Proyecto
> * **Kobalte** controla cómo **funciona** un componente (comportamiento, accesibilidad WAI-ARIA, trampas de foco).
> * **CSS propio** controla cómo **se ve** (identidad visual 100% Obsidian/Neón, unidades relativas sin frameworks prediseñados).
> * **SolidJS** controla cómo **reacciona** (reactividad de grano fino, señales directas, 0 re-renders innecesarios, 120 FPS).
>
> *Esta separación garantiza soberanía visual absoluta: **EmuBox jamás dependerá del diseño predeterminado de ninguna librería de componentes**.*

---

## 2. Visión y Ergonomía

EmuBox es una interfaz de consola diseñada para ser operada desde una distancia de visualización de 2 a 3 metros (10-Foot Interface) sobre una pantalla de televisión o monitor de 1080p (1920x1080) / 4K.

### 2.1 Restricciones de Diseño
* **Resolución base de referencia**: 1920x1080 píxeles a escala fluida (`clamp()`).
* **Sin frameworks prediseñados**: No Tailwind, no MUI, no shadcn/ui, no Bootstrap. CSS 100% propio modularizado por zonas.
* **Control primario**: Gamepad / D-Pad y teclado. El puntero del ratón está oculto por defecto.
* **Navegación espacial 2D & Indexada**: Desplazamiento ágil en direcciones cardinales con autodesplazamiento virtual.

---

## 3. Sistema de Tokens Visuales (Design Tokens)

### 3.1 Paleta Cromática
* **Canvas / Background**: `#05070a` con degradado radial profundo a `#080c14`.
* **Surface / Tarjeta Base**: `rgba(12, 17, 28, 0.88)` con borde sutil `rgba(255, 255, 255, 0.08)`.
* **Surface Active / Focused**: `rgba(22, 33, 56, 0.98)` con borde `#00f0ff`.
* **Focus Glow (Cyan)**: `#00f0ff` (con halo `box-shadow: 0 0 1.5rem rgba(0, 240, 255, 0.45)`).
* **Accent Warning (Amber)**: `#f59e0b` (para estado Favorito y destacados).
* **Accent Success (Emerald)**: `#10b981` (para botones de acción [A] y estado Listo).
* **Accent Danger (Crimson)**: `#ef4444` (para botones de volver [B]).
* **Texto Primario**: `#ffffff` (Alto contraste).
* **Texto Secundario**: `#94a3b8` (Metadatos, descripciones secundarias).
* **Texto Terciario / Inactivo**: `#64748b`.
