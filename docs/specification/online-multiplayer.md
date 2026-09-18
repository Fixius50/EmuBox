# Online y multijugador

Revision: 2026-09-18. Este documento conserva investigacion de arquitectura
futura. No autoriza implementar cuentas, servidores online, matchmaking ni
multijugador en la fase actual.

## Decision vigente

EmuBox se centra en single-player, appliance local, biblioteca, descargas,
preparacion e inicio seguro de juegos. La UI Tauri no debe convertirse en un
servidor multijugador ni cargar librerias online por anticipado.

Los cambios actuales deben optimizar el flujo local: descargas en segundo plano,
prioridad del emulador activo, control de CPU/E/S/red, cache, preparacion y
aislamiento. El multijugador se considera una linea futura separada.

## Descargas mientras se juega

El objetivo real no es solo mas velocidad: es que una descarga grande no degrade
la partida. Cualquier motor de descarga debe poder vivir fuera del proceso de UI,
tener controles de pausa/cancelacion, limites de subida/bajada, confinamiento de
rutas, reanudacion verificable y prioridad baja de CPU/E/S cuando haya juego activo.

`qBittorrent-nox` es la implementacion BitTorrent actual. Se eligio como motor
externo para evitar integrar `libtorrent-rasterbar` directamente en el binario
Rust/Tauri. Si se sustituye, el candidato preferido es `rqbit`: tiene binario,
HTTP API, servidor persistente, fastresume, limites de ratio/velocidad,
metricas Prometheus, streaming con Range y puede usarse como libreria Rust.

La sustitucion por `rqbit` debe hacerse con un proveedor experimental aislado,
no cambiando el contrato publico del catalogo ni los IDs persistidos. Validacion
minima antes de promoverlo:

1. Magnet `btih` con metadata real.
2. `.torrent` privado servido por HTTP local.
3. Pausa, reanudacion y cancelacion sin publicar archivos incompletos.
4. Limite de subida agresivo mientras hay juego activo.
5. Reanudacion sin rehash costoso cuando el motor lo permita.
6. Rechazo de rutas fuera del staging y enlaces simbolicos.
7. Medicion de CPU, RSS, escrituras y latencia del emulador durante descarga.

No se recomienda volver a `libtorrent-rasterbar` salvo que `qBittorrent-nox` y
`rqbit` fallen en requisitos medidos: introduce bindings C++/ABI, complejidad de
empaquetado y mas riesgo para ARM. `aria2c` puede estudiarse solo para HTTP
segmentado masivo, no como requisito actual ni como sustituto de BitTorrent.

## Multijugador futuro

Si EmuBox llegara a tener online, la frontera debe estar fuera de Tauri:

```text
EmuBox appliance local
  -> autenticacion opcional / lobby / matchmaking remoto
  -> asignacion de sesion
  -> juego o emulador conectado a un servidor dedicado externo

Servidor online separado
  -> cuentas, presencia, matchmaking, rankings y almacenamiento
  -> realtime y/o servidores dedicados
  -> observabilidad, cuotas, abuso y despliegue multi-region
```

La investigacion externa dejo estas opciones como referencia, no como decision de
producto:

| Area | Opcion investigada | Lectura |
| --- | --- | --- |
| Backend social/matchmaking | Nakama / `@heroiclabs/nakama-js` | Rapido para cuentas, presencia, chat, rankings y matchmaking; implica operar servidor externo. |
| Servidores dedicados a escala | Agones | Orquesta fleets de game servers en Kubernetes; util solo si existen servidores dedicados propios. |
| Backend Rust propio | Axum + Tokio + Redis/PostgreSQL | Control alto para APIs, lobbies y presencia; exige disenar auth, abuso, despliegue y observabilidad. |
| Realtime Rust | Quinn o Renet | Interesante para gameplay de baja latencia; no sustituye cuentas, matchmaking ni persistencia. |
| Rooms TypeScript | Colyseus | Productivo para prototipos de rooms; anade stack Node separado. |

Para la fase actual, ninguna de estas librerias debe anadirse al proyecto. Si en
el futuro se abre online, primero se debe escribir una especificacion separada:
tipo de juego, autoridad del servidor, persistencia, numero esperado de usuarios,
regiones, presupuesto de latencia, moderacion, privacidad y coste operativo.