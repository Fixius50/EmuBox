# Estado actual de EmuBox

Revision: 2026-09-22. Vision resumida; los documentos por responsabilidad contienen
el detalle. [Indice de documentacion](../README.md).

## Runtime y appliance

Tauri v2/Rust + SolidJS/Kobalte/CSS, SQLite persistente e IPC nativo. Vite solo
no sustituye el backend. Runtime x86_64/aarch64; ARM32 excluido. Build x86_64 no
certifica ARM fisico. Los scripts configuran Arch existente, no una ISO ni OTA del SO.

Sesion: getty@tty1 -> emubox-session -> scripts/run.sh -> compositor -> bin/emubox.
El build oficial publica ese binario; compilar solo `target/release` no implica
actualizarlo. No se reinicia TTY automaticamente.

El coordinador prepara configuracion, biblioteca cacheada, hardware y emuladores
con dependencias y plazos. Reindexacion/escaneo/sincronizacion secundarios esperan
al reconocimiento de la UI. [Arranque](console-appliance-boot-architecture.md).

Resolucion, frecuencia, fullscreen y compositor son decisiones adaptativas de la
sesion actual, obtenidas desde Wayland y la evidencia de hardware. El paralelismo
de arranque se ajusta una vez con los recursos disponibles al iniciar; no se
recalcula continuamente durante la sesion. Estos valores no se conservan como
preferencias; las configuraciones heredadas se migran al leerse para retirar esos
valores persistidos.

## Biblioteca y descargas

Carpetas por identidad canonica; versiones con IDs operativos. Favorito canonico,
instalacion y emulador por variante. Libretro identifica coincidencias exactas;
los registros no identificados mantienen un fallback local explicito. Sin fuzzy
matching automatico. [Catalogo](catalog-sources.md).

HTTP y qBittorrent-nox son proveedores separados. `bittorrent` se conserva como
valor persistido del protocolo, pero ya no selecciona aria2. Magnet sigue siendo
BitTorrent. Jackett busca fuentes, no define juegos ni descarga contenido. No se
activan indexadores automaticamente. [Descargas](download-providers.md).

Transferir bytes no equivale a instalar. ZIP/7z/RAR validan archivos; Inno se extrae
sin ejecutarlo; PKG PS3 requiere firmware y evidencia de preparacion. No se borran
parciales de aria2 ni se prometen resumes intercambiables entre motores.

## Seguridad

Todo juego se lanza con Bubblewrap, perfiles nativos y herramientas protegidas,
datos acotados, entorno limpio y sin red por defecto. Wine tiene prefijo privado.
Se elimino el shell IPC generico; detener procesos solo admite el Child activo.
Limites de audio, configs y firmware: [sandbox](execution-sandbox.md).

## Graficos y diagnostico

La GPU se determina por evidencia, no marca de CPU. Cage admite OpenGL; Gamescope
tiene requisitos propios. SVGA3D no es llvmpipe. vmwgfx usa SHM para WebKit, sin
override CPU global; los filtros se adaptan a la GPU virtual.

Los perfiles de RetroArch, PCSX2, DuckStation, Dolphin y PPSSPP parten del
backend operativo seleccionado para la sesion. Vulkan disponible por si solo no
sobrescribe una seleccion OpenGL; con backend `software` o `auto` no se fuerza un
renderer en los archivos nativos del emulador.

**La pantalla negra intermitente NO esta declarada resuelta.** La confirmacion
antigua sobre fondo/onda fue invalidada por recurrencias. [Incidencia](virtualbox-graphics.md).
Los logs incluyen hora, origen, nivel, duracion y recursos. F8 marca incidentes;
la sonda rAF no mide FPS ni pixeles. [Registros](diagnostic-logging.md).

## Pendientes

- Aceptacion visual, audio y mando por equipo; ARM real pendiente.
- Hostings sin conector, fuentes retiradas/403/404/451 y disponibilidad de peers.
- Validacion por juego/emulador; no compatibilidad universal.
- Dependencias opcionales e indexadores configurados explicitamente.
- Aplicacion en caliente de todos los ajustes, OTA e imagen reproducible.

Los contadores de tests y versiones instaladas pertenecen a informes fechados,
no a constantes de arquitectura. Una instalacion historica no certifica que un
binario siga presente o cumpla la politica de seguridad actual.

## Desarrollo

[Backend](backend-contracts.md), [filesystem](filesystem-convention.md),
[arquitectura](refactoring-and-architecture-guidelines.md) y
[requisitos/propuesta futura](../specification/requirements.md).
La base de proveedores de bibliotecas para Steam, Epic y GOG conserva cuentas,
entitlements, enlaces canonicos y estado persistido de sincronizacion sin
credenciales en SQLite. Ajustes incluye conexion, sincronizacion y desconexion:
Steam mediante clave Web API propia, Epic mediante Legendary y GOG mediante gogdl.
Los secretos quedan en el almacenamiento privado de cada proveedor. La presencia
de estos adaptadores no certifica su funcionamiento con cuentas reales; esa
aceptacion funcional sigue pendiente en el equipo objetivo.
[Tiendas](store-library-providers.md).
El despliegue nativo no ejecuta Git ni pruebas de navegador. Para trabajar la UI,
`npm run dev` habilita una previsualizacion de navegador con datos ficticios en
memoria, exclusiva de desarrollo y sin acceso a operaciones nativas. No sustituye
la aceptacion de la appliance ni usa datos reales como fixtures.