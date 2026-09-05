# Estado actual de EmuBox

**Fecha de referencia:** 5 de septiembre de 2026  
**Estado:** integración nativa x86_64/aarch64 implementada; validación ARM real pendiente.

## Alcance nativo y aceptación

- Instalación admitida: `arch/x86_64` y `archarm/aarch64`, sobre un SO existente.
- Arquitectura normalizada: `x86_64`, `aarch64`, `unsupported`; ARM32 no admitido.
- Rust usa `std::env::consts::ARCH`; `kernelArchitecture` informa `uname -m` por separado.
- Shell centraliza arquitectura, target y validación ELF en `installer/lib/architecture.sh`.
- Build nativo GNU, sin bundle obligatorio ni traducción x86: artefactos por CPU
        `emubox-linux-x86_64` y `emubox-linux-aarch64`, instalados tras validar ELF.
- El actualizador rechaza cambios locales, conserva el binario previo durante el
        pull/build y restaura ese binario si falla. No hace reset ni auto-stash.
- Las dependencias requeridas se consultan con pacman; las opcionales ausentes
        no se presentan como instaladas. La disponibilidad concreta ARM debe verificarse allí.
- `data/emulator-capabilities.json` comparte la matriz conservadora entre Rust y mocks.
- La UI consume capacidades del backend; no detecta la CPU desde el navegador.
- El workflow Native Linux usa `ubuntu-24.04` y `ubuntu-24.04-arm`, no emulación.
        Son entornos de compilación, no prueba de instalación en Arch Linux ARM.
- Se comprobó el build release x86_64. No se ha ejecutado el workflow remoto ni
        un build/arranque en ARM real: no se declara completada la aceptación ARM64.

Pendiente en ARM: instalar dependencias, compilar y verificar ELF AArch64,
arrancar como `emubox` en TTY1, verificar Gamescope con Vulkan y Cage sin Vulkan,
y probar un emulador y core nativos disponibles en ese dispositivo.

## Arquitectura vigente

```text
Arch Linux + systemd
        |
        v
getty@tty1 -> autologin de appliance -> emubox-session
        |
        +--> Gamescope -> EmuBox       (Vulkan hardware + DRM + Gamescope)
        |
        +--> Cage -> EmuBox            (resto de capacidades)
                              |
                              v
                 Tauri v2 + WebKitGTK 4.1
                              |
                              v
                 SolidJS 1.9 + Kobalte
                              |
                              v
       catálogo JSON / SQLite / servicios Rust por IPC
```

- SolidJS gestiona señales, stores y navegación espacial.
- Kobalte aporta diálogos, focus traps y primitivas accesibles.
- CSS propio define la interfaz 10-Foot UI.
- TanStack Virtual permite navegar el catálogo de 10.000 entradas.
- Tauri conecta la UI con servicios Rust mediante IPC.
- SQLite y `/var/lib/emubox/games` son la fuente de verdad de juegos instalados.
- `GameLibraryWatcher` actualiza la biblioteca cuando aparece una ROM.
- `CompatibilityService` resuelve el emulador antes del lanzamiento.

## Fase de juegos: completada

La biblioteca ya está lista para operar:

- `data/games-10000.json` proporciona el catálogo visible cuando SQLite está vacío.
- Las tarjetas muestran portada, título, plataforma, año, valoración, género y desarrollador.
- `DESCARGAR` se muestra para juegos no instalados.
- `JUGAR` requiere una ROM instalada y un emulador compatible; el bloqueo muestra su motivo.
- El store recarga la biblioteca al terminar una descarga.
- Las fuentes se importan desde manifiestos autorizados en `/etc/emubox/download-links.txt` o mediante IPC.
- `DownloadService` guarda trabajos en SQLite y descargas HTTP en `/var/lib/emubox/games/<platform>`.
- El watcher y el escaneo convierten una descarga completada en una entrada instalada.
- PS3/RPCS3, asociaciones de compatibilidad y lanzamiento forman parte del flujo de backend.

RPCS3 no se oculta en ARM: su disponibilidad depende de la matriz, del binario
presente y de CPU/RAM/GPU. Los metadatos de catálogo no garantizan que un juego
pueda descargarse o ejecutarse. No se ha validado aquí una descarga y partida real
para cada plataforma.

El JSON de catálogo contiene metadatos, no enlaces de ROM. Una descarga concreta
solo se inicia si existe una fuente autorizada registrada para su `gameId`.

## Appliance y sistema

- Código y binario: `/opt/emubox`.
- Configuración: `/etc/emubox`.
- Datos y ROMs: `/var/lib/emubox`.
- Caché: `/var/cache/emubox`.
- Logs: `/var/log/emubox`.
- El árbol de trabajo debe pertenecer a `emubox:emubox` para permitir Git y compilaciones desde VS Code.
- Las operaciones de sistema usan `sudo`; no se concede acceso root mediante el grupo `root`.
- La configuración de autologin usa el usuario `emubox`, no root. Cambiar los
        scripts no modifica por sí solo una instalación existente: ejecutar
        `sudo bash scripts/setup-autostart.sh` para regenerarla y reiniciar TTY1 después.
- `systemctl daemon-reload` se ejecuta después de crear unidades system-wide.
- `systemctl --user daemon-reload` se ejecuta después de crear unidades de usuario.
- `systemd-timesyncd` debe estar habilitado y la hora sincronizada.

## Gráficos

La detección consulta DRM/sysfs, dispositivos DRI y Vulkan; PCI es un apoyo,
no un requisito. Reconoce AMD, Intel, NVIDIA, Broadcom, Mali, Adreno, Apple,
drivers virtuales y desconocidos sin inferir GPU desde la CPU. Gamescope requiere
Vulkan hardware, DRM y ejecutable disponible; en otro caso se selecciona Cage.
No tener Vulkan no equivale a renderizar por CPU. No se exporta RADV_PERFTEST global.

## Menú operativo

`script.sh` mantiene solo las tareas de appliance:

1. Compilar EmuBox.
2. Actualizar desde GitHub.
3. Configurar appliance.
4. Diagnosticar entorno.
5. Ejecutar instalación completa de Arch Linux.

Las tareas de desarrollo, tests, Vite y lanzamiento manual no forman parte del
menú principal de la appliance.

## Verificación registrada

- TypeScript: correcto.
- Build frontend: correcto.
- Rust/Tauri: correcto.
- Suite vertical: 68 pruebas pasadas.
- Chequeo arquitectónico: correcto.
- Chequeo de codificación: correcto.
- NTP: sincronizado.
