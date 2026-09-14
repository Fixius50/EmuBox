# Estado actual de EmuBox

**Fecha de referencia:** 13 de septiembre de 2026
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
- `data/emulator-capabilities.json` define la matriz conservadora del runtime Rust.
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
        +--> detectar aceleracion -> elegir OpenGL/Vulkan/software
        |
        +--> compositor compatible (Cage por defecto) -> EmuBox
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
- XMB renderiza una ventana acotada de carpetas y versiones; `useXmbLibrary` controla entradas y estado, sin TanStack Virtual.
- Tauri conecta la UI con servicios Rust mediante IPC.
- SQLite y `/var/lib/emubox/games` son la fuente de verdad de juegos instalados.
- `GameLibraryWatcher` actualiza la biblioteca cuando aparece una ROM.
- `CompatibilityService` resuelve el emulador antes del lanzamiento.

## Catalogo conectado

La biblioteca ya está lista para operar:

- La UI muestra SQLite; no se distribuyen datasets generados ni biblioteca de demostracion.
- Las tarjetas muestran portada, título, plataforma, año, valoración, género y desarrollador.
- `DESCARGAR` se muestra para juegos no instalados.
- `JUGAR` requiere una ROM instalada y un emulador compatible; el bloqueo muestra su motivo.
- El store recarga la biblioteca al terminar una descarga.
- Las fuentes se importan desde `/etc/emubox/download-links.txt` si tiene URLs; si no, desde `data/download-links.txt`.
- Importar crea metadatos y fuentes, no trabajos de descarga. Se actualiza al arrancar y cada seis horas.
- El evento oficial Tauri `library-updated` recarga la biblioteca despues de importar cada manifiesto.
- El gestor guarda trabajos y snapshot de fuente en SQLite, con proveedores HTTP/BitTorrent y staging por trabajo.
- Solo una preparacion no ambigua registra el juego como instalado; archivos pendientes quedan `downloaded`, no jugables automaticamente.
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
HTTP directos y dispone de proveedor BitTorrent mediante aria2 instalado. El conector
publico Pixeldrain no elude limites. EmuBox funciona sin registro; los conectores
con cuenta, incluido 1fichier, estan desactivados por defecto. No se requieren
claves para HTTP publico, BitTorrent publico ni preparacion local. Otros hostings, incluido
GoFile, requieren conectores adicionales. La preparacion reconoce ZIP, 7z y RAR
mediante lectores de archivo; 7z, RAR5 almacenado y comprimido probados localmente.
El selector permite reintentar preparacion y elegir un archivo de lanzamiento
entre candidatos del paquete, sin repetir la descarga. Los EXE Inno Setup compatibles
se extraen mediante innoextract, sin ejecutar el instalador; Wine tiene perfil PC y
prefijo propio por paquete. PKG PS3 usa RPCS3 headless en un entorno aislado, requiere
firmware local y comprueba log y estructura de salida. No son instaladores universales:
EXE de otros formatos, descriptores multidisco y compatibilidad de juegos quedan pendientes.
Ver [detalle y limites](download-providers.md).

## Ampliacion del 14 de septiembre de 2026

- Firmware PS3 4.93 descargado desde el enlace oficial Sony. Los nombres CDN fallaron
        TLS; se uso el enlace HTTP publicado, contrastando el MD5 incluido en la URL obtenida
        por HTTPS. Ese digest no equivale a una firma criptografica independiente.
        RPCS3 confirmo en su log la instalacion; fallo al cerrar el proceso tras instalar.
        PUP conservado en `/var/lib/emubox/bios/ps3/PS3UPDAT.PUP` y `dev_flash` en
        `/var/lib/emubox/emulators/rpcs3/config/rpcs3/dev_flash`. El preparador descubre
        esa ubicacion y permite override `EMUBOX_PS3_FIRMWARE_DIR`.
- RPCS3 0.0.42-19996, Azahar 2126.1.1 y shadPS4 0.18.0 instalados desde publicaciones
        oficiales. Sus binarios descargados se comprobaron con SHA-256 publicado. Azahar
        continua Citra; tambien se instalo su core Libretro x86_64. No se ejecutaron juegos.
- Plataformas PS4 y 3DS registradas. PS4 reconoce carpetas con `eboot.bin` y
        `sce_sys/param.sfo`; 3DS admite los formatos registrados por el perfil Azahar.
- Alternativas Libretro con core explicito: Dolphin, Flycast, melonDS, mGBA,
        PPSSPP, Azahar, Snes9x, Mupen64Plus Next, Genesis Plus GX y FinalBurn Neo.
        Un perfil registrado no implica core instalado; FinalBurn Neo no estaba disponible.
        RetroArch generico sin core no se anuncia listo para lanzar.
- Instalados y comprobados via pacman: Wine, cabextract, innoextract, unshield,
        aria2, libarchive, los cores Snes9x/Mupen64Plus Next/Genesis Plus GX, mGBA Qt,
        PPSSPP, GameMode y MangoHud. Estos dos ultimos no se activan automaticamente.
- La tarjeta abre la seleccion de motor; permite guardar preferencia por juego sin
        ejecutarlo. El lanzamiento directo respeta la asociacion. Se corrigio el contrato
        `customArgs`, conservando lectura del alias anterior `customArguments`.
- Preparadores ejecutados con bubblewrap sin red, entradas de solo lectura y staging
        escribible, limites de CPU/tiempo/espacio virtual y comprobacion periodica de disco.
        No es una cuota estricta de disco ni un sandbox del juego una vez lanzado por Wine.
        `WINEDLLOVERRIDES=mscoree,mshtml=` evita descargas automaticas Mono/Gecko.
- Pruebas reales: extraccion del instalador oficial Inno 6.0.5, conservacion del
        original, seleccion y configuracion Wine sin ejecutar el EXE extraido; RPCS3 headless
        y rechazo de PKG invalido con firmware. Sigue sin validarse un PKG de juego correcto.

La ausencia de Vulkan en esta VM es una limitacion del entorno de prueba, no una exclusion de Intel, AMD o NVIDIA. La deteccion y las rutas GPU se conservan para hardware compatible; no se certifican drivers, extensiones ni juegos no probados.
shadPS4 necesita Vulkan 1.3 y otros requisitos de CPU/GPU: su instalacion no garantiza ejecucion en esta VM. No se encontraron modulos PS4 ni datos BIOS/3DS locales.
No se descargaron copias de firmware de terceros sin autorizacion.

UMU con Proton permite ejecutar fuera de Steam sin cuenta; fue investigado, no instalado. DXVK y vkd3d-proton se evaluaron como complementos Vulkan, no se activaron ni se presentan como mejoras universales. No hay mediciones de rendimiento en GPU fisicas.

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

La seleccion automatica prioriza GPU real o 3D virtual. Dos sondeos concluidos
con renderers software permiten CPU; un sondeo fallido conserva `indeterminate`
y se intenta inicializacion automatica, con fallback software solo explicito. Cage no implica CPU:
puede componer con OpenGL acelerado. Un valor legado `software` en la configuracion
no prevalece sobre aceleracion detectada; esto no certifica la estabilidad de SVGA3D.

La detección consulta DRM/sysfs, dispositivos DRI, EGL/OpenGL y Vulkan; PCI es un apoyo,
no un requisito. Reconoce AMD, Intel, NVIDIA, Broadcom, Mali, Adreno, Apple,
drivers virtuales y desconocidos sin inferir GPU desde la CPU. Primero se selecciona
OpenGL acelerado, Vulkan acelerado o software. Cage es la opcion automatica con
OpenGL/DRM; Gamescope mantiene sus requisitos Vulkan propios, pero solo se elige
por preferencia compatible o si Cage no puede servir el backend. Si falta un
compositor compatible se informa del error sin degradar una GPU detectada a CPU.
No tener Vulkan no equivale a renderizar por CPU. No se exporta RADV_PERFTEST global.

El sondeo EGL/OpenGL identifica `SVGA3D` acelerado en la VM actual, sin confundir
su texto `LLVM` con `llvmpipe`. Vulkan no está disponible allí y se elige Cage.
CPU expuesta: Ryzen 5 5600G, cuatro vCPU; GPU expuesta: VMware SVGA II con vmwgfx.
La Radeon integrada mencionada en el nombre de la CPU no es la GPU PCI del invitado.
Ver [aceleración en VirtualBox](virtualbox-graphics.md).

DRM y las evidencias EGL/Vulkan se correlacionan por identificadores, nunca por
nombre. MultiGPU sin identidad verificable conserva observaciones sin asociar;
monoGPU puede usar inferencia declarada. El dispositivo usado por el proceso no
se afirma sin evidencia. Rust proporciona tanto IPC como el sondeo del lanzador.
La especificacion de producto esta en [emubox-os-specification.md](emubox-os-specification.md),
todavia pendiente de decisiones de hardware/instalacion/actualizacion.

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
