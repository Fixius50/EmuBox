# Estado actual de EmuBox

**Fecha de referencia:** 5 de septiembre de 2026  
**Estado:** runtime nativo y configuracion de appliance x86_64/aarch64 implementados;
aceptacion funcional ARM real pendiente. Distribucion reproducible: fase posterior.

## Tres capas y prioridad

Runtime (Tauri/Rust/SolidJS/IPC/SQLite), appliance (systemd/usuario/filesystem/
Wayland/input/audio) y distribucion (base/kernel/boot/imagen) tienen criterios
distintos. La prioridad actual es Runtime + Appliance, no construir una ISO.
La ruta manual convierte Arch existente en appliance; no genera la distribucion.

La base ya no instala motores: RetroArch y otros emuladores son opcionales mediante
`installer/setup/emulator-packages.sh`. La configuracion converge en un lanzador,
autologin `emubox`, audio de usuario y tmpfiles para `/run/emubox`.
`npm run test:appliance` verifica paquetes, configuracion y bootstrap sin motores;
`npm run check:appliance` inspecciona sin modificar y no certifica aceptacion fisica.
Ver [criterios de validacion](appliance-validation.md).

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
- El workflow Runtime Linux (native) usa `ubuntu-24.04` y `ubuntu-24.04-arm`, no emulación.
        Son entornos de compilación, no prueba de instalación en Arch Linux ARM.
- Se comprobó el build release x86_64. No se ha ejecutado el workflow remoto ni
        un build/arranque en ARM real: no se declara completada la aceptación ARM64.

Pendiente en ARM: dependencias nativas, ELF AArch64 y arranque como `emubox` en
TTY1, UI/IPC/SQLite/filesystem, input/audio y graficos reales. Validar Gamescope
donde haya capacidades y Cage como alternativa. Los emuladores/cores se prueban
despues y no condicionan la aceptacion de la appliance.

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
- TanStack Virtual permite navegar el catalogo importado sin renderizar todas sus tarjetas.
- Tauri conecta la UI con servicios Rust mediante IPC.
- SQLite y `/var/lib/emubox/games` son la fuente de verdad de juegos instalados.
- `GameLibraryWatcher` actualiza la biblioteca cuando aparece una ROM.
- `CompatibilityService` resuelve el emulador antes del lanzamiento.

## Catalogo conectado

La biblioteca ya está lista para operar:

- La UI muestra SQLite; `data/games-10000.json` queda solo para pruebas, sin fallback en la appliance.
- Las tarjetas muestran portada, título, plataforma, año, valoración, género y desarrollador.
- `DESCARGAR` se muestra para juegos no instalados.
- `JUGAR` requiere una ROM instalada y un emulador compatible; el bloqueo muestra su motivo.
- El store recarga la biblioteca al terminar una descarga.
- Las fuentes se importan desde `/etc/emubox/download-links.txt` si tiene URLs; si no, desde `data/download-links.txt`.
- Importar crea metadatos y fuentes, no trabajos de descarga. Se actualiza al arrancar y cada seis horas.
- El evento oficial Tauri `library-updated` recarga la biblioteca despues de importar cada manifiesto.
- `DownloadService` guarda trabajos en SQLite y descargas HTTP en `/var/lib/emubox/games/<platform>`.
- El watcher y el escaneo convierten una descarga completada en una entrada instalada.
- PS3/RPCS3, asociaciones de compatibilidad y lanzamiento forman parte del flujo de backend.

RPCS3 no se oculta en ARM: su disponibilidad depende de la matriz, del binario
presente y de CPU/RAM/GPU. Los metadatos de catálogo no garantizan que un juego
pueda descargarse o ejecutarse. No se ha validado aquí una descarga y partida real
para cada plataforma.

El JSON de catálogo contiene metadatos, no enlaces de ROM. Una descarga concreta
solo se inicia si existe una fuente autorizada registrada para su `gameId`.

La importacion real del 5 de septiembre registro 167.901 juegos y 278.460 fuentes
unicas, con cero trabajos de descarga. Varias URLs devolvieron 403/404 o fallaron
por conectividad; no se presentan como importadas. El backend descarga archivos
HTTP directos y rechaza magnet/torrent con un mensaje explicito hasta incorporar
un motor BitTorrent. Ver [detalle y limites](catalog-sources.md).

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

La seleccion automatica prioriza GPU real o 3D virtual. Sin Vulkan hardware ni
OpenGL acelerado detectados, usa CPU como alternativa. Cage no implica CPU:
puede componer con OpenGL acelerado. El override local de diagnostico `software`
se ha devuelto a `auto`; esto no certifica la estabilidad de SVGA3D.

La detección consulta DRM/sysfs, dispositivos DRI y Vulkan; PCI es un apoyo,
no un requisito. Reconoce AMD, Intel, NVIDIA, Broadcom, Mali, Adreno, Apple,
drivers virtuales y desconocidos sin inferir GPU desde la CPU. Gamescope requiere
Vulkan hardware, DRM y ejecutable disponible; en otro caso se selecciona Cage.
No tener Vulkan no equivale a renderizar por CPU. No se exporta RADV_PERFTEST global.

El sondeo EGL/OpenGL identifica `SVGA3D` acelerado en la VM actual, sin confundir
su texto `LLVM` con `llvmpipe`. Vulkan no está disponible allí y se elige Cage.
CPU expuesta: Ryzen 5 5600G, cuatro vCPU; GPU expuesta: VMware SVGA II con vmwgfx.
La Radeon integrada mencionada en el nombre de la CPU no es la GPU PCI del invitado.
Ver [aceleración en VirtualBox](virtualbox-graphics.md).

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
