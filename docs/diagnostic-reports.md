# EmuBox: Registro Técnico y Reportes de Diagnóstico de Entorno

Este documento registra la cronología de diagnósticos, pruebas de compilación, hallazgos de arquitectura y resolución de incidencias en el entorno de desarrollo y ejecución de **EmuBox (Arch Linux / Dedicated 10-Foot Console)**.

---

## 📑 Reporte 1: Auditoría de Arquitectura NPM y Frontend SolidJS

* **Fecha**: 28 de Agosto, 2026
* **Objetivo**: Determinar la estructura de empaquetado de dependencias y el origen de errores de compilación anteriores.

### Hallazgos Clave:
1. **Centralización en Raíz**: El proyecto no contiene ni requiere un `package.json` anidado en `solid/`. El único `package.json` y `package-lock.json` válidos residen en `/opt/emubox/` (raíz del repositorio).
2. **Orquestación de Build**: El comando `npm run build` en la raíz ejecuta:
   ```bash
   vite build solid --config solid/vite.config.ts
   ```
   Toma el código fuente desde `solid/src/` y genera los artefactos estáticos en `solid/dist/`, coincidiendo con `frontendDist: "../solid/dist"` de Tauri.
3. **Módulos y Dependencias**: 174 módulos transformados por Vite 6.4.3 con SolidJS 1.9 y Kobalte.

---

## 📑 Reporte 2: Prueba de Entorno y Compilación Nativa (VM Arch Linux)

* **Fecha**: 28 de Agosto, 2026
* **Entorno Evaluado**: Arch Linux x86_64 (VMware SVGA3D / Direct DRM-KMS)
* **Herramientas**:
  * Node.js: `v26.7.0`
  * npm: `12.0.2`
  * Vite: `6.4.3`
  * esbuild: `0.28.2`
  * Rust / Cargo: `stable`

### Resultados:
```text
✓ npm ci: COMPLETADO (0 errores)
✓ npm rebuild esbuild: REBUILT DEPENDENCIES SUCCESSFULLY
✓ npm run build: GENERADO solid/dist/ (index.html, CSS 45.7 kB, JS 22 MB)
✓ npx tauri build --no-bundle: COMPILACIÓN RUST EXITOSA (0 errores)
✓ Binario ELF generado: /opt/emubox/bin/emubox (Permisos 0755)
✓ BUILD_SCRIPT_EXIT = 0
```

### Conclusión:
El pipeline de construcción y empaquetado nativo (`scripts/build.sh`) funciona de forma 100% limpia y sin fallos de permisos cuando se ejecuta como usuario del sistema (`emubox`).

---

## 📑 Reporte 3: Diagnóstico del Sistema Gráfico y Renderizado Autónomo (GPU vs CPU)

* **Fecha**: 28 de Agosto, 2026
* **Problema Abordado**: Rendimiento en VMs (VMware SVGA3D, VirtualBox) o drivers por software (`llvmpipe`) sin aceleración 3D directa (`Accelerated: no`).

### Solución Implementada:
1. **Detección Automática (<1 ms)**: Implementación de [`GraphicsDetectorService`](file:///c:/Users/rober/Desktop/Proyectos%20propios/EmuBox-Lab/solid/src/services/graphics/graphics-detector.service.ts) mediante `UNMASKED_RENDERER_WEBGL`.
2. **Tokens Adaptativos CSS**: Sustitución de los 11 puntos estáticos de `backdrop-filter: blur(...)` y sombras difusas pesadas por tokens dinámicos en [`variables.css`](file:///c:/Users/rober/Desktop/Proyectos%20propios/EmuBox-Lab/solid/src/styles/variables.css).
3. **Cero Convolución en CPU**:
   * *Modo GPU Acelerada*: Superficies translúcidas con desenfoque Gaussiano y resplandores de neón.
   * *Modo CPU Compatible*: Superficies oscuras sólidas de alto contraste (`rgba(10, 15, 26, 0.96)`) con contornos precisos de 2px a **60 FPS estables**.
4. **Verificación**: **42/42 pruebas superadas** en la suite automatizada (`tests/vertical-slice.test.ts`).

---

## 📑 Reporte 4: Diagnóstico de Ejecución en Tiempo Real (Systemd & SIGABRT)

* **Fecha**: 28 de Agosto, 2026
* **Síntoma**: `emubox.service` termina inmediatamente con `status=6/ABRT` (`SIGABRT`, código de salida `134`).
* **Causa Raíz Confirmada**:
  * Traza capturada: `thread 'main' panicked ... tao-0.35.3/src/platform_impl/linux/event_loop.rs: Failed to initialize gtk backend! Failed to initialize GTK`.
  * La sesión SSH no dispone de servidor gráfico (`DISPLAY=<vacio>`, `WAYLAND_DISPLAY=<vacio>`, `XDG_SESSION_TYPE=tty`).
  * `tao` / `gtk::init()` lanza un panic en Rust al no encontrar pantalla disponible, generando `SIGABRT` y bucle de reinicios en systemd.

---

## 📑 Reporte 5: Decisión Arquitectónica del Stack Gráfico (Pure Wayland)

* **Fecha**: 28 de Agosto, 2026
* **Stack Gráfico Objetivo**: **Wayland puro** (Direct DRM/KMS + Gamescope / Cage).
* **Decisiones Inmutables**:
  1. No se introduce Xorg / X11. EmuBox es un entorno de consola moderno basado 100% en Wayland.
  2. **Gamescope** (con Cage como alternativa ligera) actuará como compositor de pantalla completa embebido sobre DRM/KMS.
  3. **SSH** se mantiene estrictamente para tareas de administración remota, no para arrancar el entorno gráfico.
  4. El arranque de la interfaz se orquestará localmente en la TTY física (`/dev/tty1`) mediante autologin y sesión Wayland de usuario.

---

## 📑 Reporte 6: Validación Exitosa de Cage en DRM Directo & Arranque en Frío

* **Fecha**: 29 de Agosto, 2026
* **Hito Alcanzado**: **EmuBox desplegado con éxito en pantalla física a 1080p**.

### Resultados de la Prueba en TTY1:
```text
✓ Detección de Virtualización: oracle (VirtualBox)
✓ GPU Virtual: VMware SVGA II Adapter (Kernel: vmwgfx)
✓ DRM/KMS: Activo en /dev/dri/card0 y /dev/dri/renderD128
✓ Sondeo Vulkan: Incompatible en VM -> Modo COMPATIBILIDAD seleccionado automáticamente
✓ Compositor: Cage Wayland Kiosk ejecutado directamente sobre DRM/EGL
✓ Renderizado: EmuBox OS desplegado a pantalla completa (1920x1080)
✓ Cero bucles de reinicio y cero dependencias de SSH
```

### Siguiente Paso:
Validación del ciclo completo de arranque en frío (`sudo reboot`) comprobando el encendido autónomo desde UEFI/systemd hasta la interfaz visual sin intervención manual.


