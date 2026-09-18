# Requisitos y alcance

Revision: 2026-09-16. Consolida requisitos, diseno y propuesta de producto;
no sustituye la evidencia de aceptacion de hardware.

## Producto

EmuBox es un launcher nativo y una appliance sobre Linux existente, no un kernel,
driver ni emulador universal. Runtime, appliance y distribucion son entregables
distintos. Tauri/Rust y SolidJS estan implementados; hay scripts para Arch
x86_64/aarch64. La ISO, particionado, bootloader y OTA del SO siguen siendo una
propuesta no implementada ni autorizada por esta especificacion.

Online, cuentas, matchmaking, servidores dedicados y multijugador quedan fuera de
la fase vigente. La investigacion se conserva aparte para no condicionar el
runtime single-player ni anadir librerias remotas antes de tener especificacion.

El soporte se declara por equipo, version y configuracion. x86_64 tiene build
local; ARM64 requiere aceptacion fisica. ARM32 queda fuera del contrato actual.
Soportado requiere aceptacion completa; compatible, evidencia tecnica parcial;
no soportado exige restriccion expresa; no determinado significa evidencia insuficiente.
Una GPU virtual acelerada no certifica el hipervisor ni sus drivers.

## Interfaz

- SolidJS controla reactividad; Kobalte, comportamiento accesible y foco; CSS propio,
  presentacion. Logica en hooks/stores y tipos centralizados en `solid/src/types`.
- Mando, teclado y raton, foco visible, retorno al cerrar modales, filas acotadas
  y tarjetas de versiones bajo demanda. Dimensiones estables y unidades relativas.
- Juego canonico -> versiones -> fuentes -> descarga -> preparacion -> instalacion
  -> emulador autorizado -> sandbox -> jugar.
- Sin datos de demostracion en produccion, metadatos inventados, exito simulado
  ni descarga automatica al abrir una ficha.
- Los tokens reales estan en `solid/src/styles`; no se replica aqui su paleta.
  Animaciones y filtros dependen de capacidades. No se garantizan 60/120 FPS,
  ausencia de repintados o memoria constante por elegir un framework.

## Datos y seguridad

SQLite y archivos gestionados conservan IDs y estado. Las pruebas usan datos
aislados, nunca la base real. Configuracion invalida y fallos de permisos son errores.
HTTP funciona directamente; qBittorrent-nox gestiona magnet/torrent. Jackett busca
fuentes explicitamente en indexadores elegidos por el usuario. No se requieren
cuentas externas de EmuBox, pago ni suscripciones; las claves locales no salen a UI/logs.
Se respetan restricciones de terceros y no se configuran indexadores automaticamente.

Descargado, preparado e instalado siguen siendo no confiables. Preparacion valida
rutas y limites; ejecucion usa perfiles nativos y Bubblewrap, sin shell IPC generico,
argumentos libres ni home personal. Ver contratos especializados.

## Distribucion pendiente

Antes de construir una imagen se deben acordar equipos, minimos, particionado,
cifrado/dual boot, root mutable o inmutable, snapshots/A-B, firmas y custodia de
claves, canales, migraciones, copia y recuperacion offline. No se ha elegido
archiso, mkosi ni un mecanismo OTA completo. No implica permiso para tocar discos.

## Referencias

- [Estado y limites](../architecture/current-state.md)
- [Aceptacion](../architecture/appliance-validation.md)
- [Backend](../architecture/backend-contracts.md)
- [Catalogo](../architecture/catalog-sources.md)
- [Descargas](../architecture/download-providers.md)
- [Sandbox](../architecture/execution-sandbox.md)
- [Online y multijugador futuro](online-multiplayer.md)
- [Metodo de medicion](benchmark.md)