# Estado actual de EmuBox

**Fecha de referencia:** 4 de septiembre de 2026  
**Estado:** fase de catálogo e integración de juegos completada.

## Arquitectura vigente

```text
Arch Linux + systemd
        |
        v
getty@tty1 -> autologin de appliance -> emubox-session
        |
        +--> Gamescope -> EmuBox       (GPU/Vulkan real)
        |
        +--> Cage -> EmuBox            (VM/software/fallback)
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
- `JUGAR` se muestra cuando el escaneo confirma la ROM instalada.
- El store recarga la biblioteca al terminar una descarga.
- Las fuentes se importan desde manifiestos autorizados en `/etc/emubox/download-links.txt` o mediante IPC.
- `DownloadService` guarda trabajos en SQLite y descargas HTTP en `/var/lib/emubox/games/<platform>`.
- El watcher y el escaneo convierten una descarga completada en una entrada instalada.
- PS3/RPCS3, asociaciones de compatibilidad y lanzamiento forman parte del flujo de backend.

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
- `systemctl daemon-reload` se ejecuta después de crear unidades system-wide.
- `systemctl --user daemon-reload` se ejecuta después de crear unidades de usuario.
- `systemd-timesyncd` debe estar habilitado y la hora sincronizada.

## Gráficos

La detección identifica primero drivers virtuales como `vmwgfx`, VMware SVGA,
VirtualBox, `virtio` o `qxl`. No los clasifica como AMD por coincidencias de
texto. En una VM sin Vulkan válido se usa `Cage -> EmuBox`; con aceleración real
se usa `Gamescope -> EmuBox`.

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
