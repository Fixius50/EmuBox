# Matriz de Evaluación: Primitivas Headless para EmuBox (Kobalte vs Bits UI vs Reka UI / Radix Vue)

## 1. Contexto y Distinción Arquitectónica

Radix UI original fue concebido como una suite de primitivas headless (sin estilos) exclusivamente para React. En nuestro laboratorio para **EmuBox**, evaluamos los puertos e implementaciones nativas independientes desarrolladas por cada comunidad, las cuales reproducen las especificaciones de comportamiento WAI-ARIA y gestión de foco sin arrastrar el runtime de React:

* **Kobalte (`@kobalte/core`)**: Implementación nativa para **SolidJS**. Basada en Signals directos y JSX de Solid.
* **Bits UI (`bits-ui`) / Melt UI**: Implementación nativa para **Svelte 5**. Construida aprovechando Runes (`$state`, `$derived`, snippets) y actions de Svelte.
* **Reka UI / Radix Vue (`radix-vue`)**: Implementación nativa para **Vue 3**. Construida con Composition API y directivas de Vue.

---

## 2. Matriz de Criterios de Evaluación

| Criterio | Kobalte (SolidJS) | Bits UI (Svelte 5) | Reka UI / Radix Vue (Vue 3) |
| :--- | :--- | :--- | :--- |
| **Arquitectura Interna** | Signals puros (sin VDOM) | Runes + Compilador Svelte | Composition API + Proxies |
| **Primitiva `Dialog` (Modales)** | Focus trap nativo, portal directo al body, backdrop personalizable | Focus trap nativo, portal accesible, scroll lock configurable | Focus trap nativo con `FocusScope`, teleports limpios |
| **Primitiva `Tabs` (Secciones)** | Navegación horizontal/vertical por teclado automática | Roving tabindex integrado, soporte de snippets | Roving focus integrado, v-model reactivo |
| **Primitiva `Switch` (Shaders/VSync)**| Estado booleano en Signal, cero lag de repintado | Binding directo `$state()`, transición CSS pura | `v-model:checked`, evento de cambio nativo |
| **Primitiva `Slider` (Volumen/Deadzone)**| Arrastre táctil/ratón + soporte D-pad discreto | Interacción continua por teclado/mando | Soporte de pasos (`step`) con feedback reactivo |
| **Integración con Gamepad** | Excelente vía callbacks de Signals | Excelente vía props y bindings reactivos | Excelente vía listeners de eventos estándar |
| **Independencia Visual** | **100% Headless**: No inyecta clases ni estilos; 100% gobernado por nuestras variables CSS | **100% Headless**: Unstyled por diseño; control total de clases y pseudo-selectores | **100% Headless**: Cero estilos predeterminados; clases BEM/custom |
| **Impacto en Bundle (Gzip)** | **~12 KB** (árboles sacudibles por componente) | **~14 KB** (código compilado compacto) | **~18 KB** (runtime de componentes Vue) |
| **Ergonomía de API** | Muy declarativa (`<Dialog.Root>`, `<Dialog.Portal>`, `<Dialog.Content>`) | Intuitiva con snippets Svelte 5 (`<Dialog.Root>`, `<Dialog.Content>`) | Sintaxis idiomática de Vue (`<DialogRoot>`, `<DialogContent>`) |

---

## 3. Principio de Estilizado y Animaciones

1. **Cero Frameworks CSS Prefabricados**: Queda terminantemente prohibido el uso de TailwindCSS, shadcn/ui, Bootstrap o MUI.
2. **Sistema Visual EmuBox Obsidian/Cyan Glow**: Todas las primitivas headless consumen exclusivamente los tokens de `shared/styles/variables.css` y las clases de `shared/styles/base.css`.
3. **Animaciones Estandarizadas (CSS/WAAPI)**: No se utilizan motores de animación propietarios para evitar sesgos en el benchmark de rendimiento. Las transiciones de apertura/cierre de Dialog, tabs y toggles se ejecutan mediante CSS GPU Transforms (`transform: translateZ(0)`, `backdrop-filter: blur(12px)`) y Web Animations API.
