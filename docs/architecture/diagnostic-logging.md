# Controlador de registros

El negro intermitente sigue abierto. Este controlador permite correlacionarlo con
operaciones y procesos; no presupone una causa ni comprueba los pixeles presentados.
No cambia los ajustes graficos ni el aislamiento de los juegos.

## Archivo y formato

Archivo activo: `/var/log/emubox/events.jsonl`. Una linea JSON por evento,
sin volcar el catalogo ni el entorno. Rotacion: 8 MiB y cuatro copias por defecto
(`events.jsonl.1` es la mas reciente), unos 40 MiB en total mas una entrada.
Un bloqueo exclusivo impide que dos instancias roten el mismo archivo. Un fallo
de apertura/escritura se anuncia por stderr y no impide usar EmuBox.

Campos comunes: `timestamp` (UTC ISO), `timestampMs` (epoch), `elapsedMs`
(monotono desde el inicio del controlador), `sessionId`, `bootId`, `sequence`,
`level`, `source`, `event`, `pid`, `thread`, `message` y `data`.
PID/hilo comunes son los del recolector; el journal conserva el PID original,
unidad y horas originales en `data.originPid`, `originTimestampUs` y
`originMonotonicUs`. La UI aporta `clientTimestampMs` y `clientElapsedMs`.
Las lineas legadas de session.log tienen hora de observacion, no una hora de
emision inventada; se marcan como `timestampKind=observed`.

## Responsabilidades

`services/infrastructure/telemetry` centraliza captura, filtros y escritura:

- `mod.rs`: inicializacion de la fachada `log`, filtros, limites, ocultacion y
  funciones `event`, `span` y `operation`.
- `writer.rs`: cola, escritura JSONL completa y rotacion.
- `collectors.rs`: nuevas lineas de sesion, journal y muestras de procesos.

Las macros Rust `log::info!`, `warn!`, `error!`, etc. ya tienen receptor. Conservan
modulo, archivo y linea. Los spans generan `operation.start` y `operation.end`
con el mismo `operationId`; `operation.result` incluye exito/error. El fin de un
span solo significa salida de la funcion, no exito de la operacion.

Estan instrumentados arranque, escaneo, lectura de biblioteca, indice local,
base maestra, manifiestos y lanzamiento. El estado del coordinador incluye tareas,
recursos, avisos y errores. Las trazas antiguas por stderr se siguen recogiendo;
su nivel/origen se infiere del texto y puede ser impreciso o duplicar un evento
estructurado. No se modifica ni rota el session.log legado.

La UI registra clics (boton/tipo de elemento, sin texto), acciones de mando,
foco, visibilidad, redimensionado, errores JavaScript/promesas, console y perdida
de contexto WebGL observable. Cada cinco segundos solicita dos callbacks
requestAnimationFrame y mide su espera; no mantiene un bucle de animacion continuo
ni provoca repintados CSS. `sampling=two-frames` distingue estas muestras de las
antiguas: `frames=2` no significa dos FPS. `probeLatencyMs` mide la espera completa
y `maxGapMs` la mayor espera de la sonda. Si sigue pendiente en el siguiente
intervalo, informa `frames=0` sin superponer otra sonda. Una pausa >1s en una
pagina visible se marca como aviso. Esto mide callbacks, no
presentacion real en pantalla. Los IPC de >=500ms se registran como avisos;
los rapidos son debug. No se guardan argumentos ni resultados IPC.

F8 genera `incident.mark`: pulsarlo al observar el negro ayuda a buscar la ventana
temporal. La captura de input no guarda otras teclas, valores de formularios, nombres
de juegos ni movimientos individuales del raton. Los mensajes legados de consola,
stderr o journal pueden contener titulos o rutas: se aplica ocultacion conservadora,
pero deben revisarse antes de compartirlos. Si la UI se bloquea por completo,
la marca puede llegar tarde o no llegar; los recolectores nativos continuan.

## Configuracion

Se lee al iniciar `/etc/emubox/logging.json`; si no existe, se usa
`/opt/emubox/data/logging.json`. La configuracion se valida; un error activa los
valores por defecto y genera un aviso. Cambiarla requiere reiniciar la sesion.

- `enabled`: interruptor de captura nativa.
- `level`: umbral global `trace`, `debug`, `info`, `warn`, `error` u `off`.
- `sources`: umbrales por prefijo; gana el mas largo. Ejemplos: `ui.input`,
  `ui.render`, `ui.ipc`, `startup`, `catalog.manifests`, `os.kernel`, `system.resources`.
- `collectSession`, `collectJournal`: recolectores opcionales.
- `journalPriority`: prioridad maxima de journalctl; por defecto `warning`,
  incluyendo errores. `info` incluye tambien informacion del SO accesible.
- `processIntervalSeconds`: 2 a 60, por defecto 5. CPU por proceso puede superar
  100% en multicore; la primera muestra no tiene intervalo previo significativo.
- `eventsPerSecond`: 10 a 1000, por defecto 200.
- `maxFileBytes`: 64 KiB a 64 MiB; `retainedFiles`: 1 a 8 copias.

La cola nativa tiene 1024 entradas y no bloquea productores. La UI tiene 64
entradas y envia hasta 32 por segundo, sin IPC superpuestos. Las perdidas se
anuncian como `events.dropped` o `droppedEvents`; el filtro descarta deliberadamente
lo que no cumple el umbral. Los ultimos eventos pueden perderse por apagado brusco;
no se hace fsync por evento. Nunca se promete captura sin perdida.

Se reemplazan URLs, rutas del home y mensajes/campos reconocibles de credenciales.
Se acotan mensajes y datos anidados. Es ocultacion conservadora, no deteccion
perfecta de secretos; no publiques los archivos completos sin revisarlos. Los
archivos nuevos se crean con permisos 0600. La UI no puede elegir destino, cambiar
filtros ni atribuirse origen de kernel a traves del IPC de logs.

## Cobertura del sistema

La captura comienza al arrancar Tauri, no antes del SO. El journal importa hasta
200 avisos recientes del boot actual y despues sigue su cursor cada cinco segundos.
Solo lee lo permitido al usuario actual, sin sudo ni cambio de grupos. Lecturas
fallidas o recortadas generan avisos de cobertura. Mensajes de firmware/hipervisor
no publicados en el journal del invitado no estan disponibles aqui.

session.log se sigue desde el final al arrancar, comprobando nuevas lineas cada
500ms y leyendo como maximo 256 KiB por pasada. Las muestras de procesos registran
nombre, PID/padre, CPU, RSS, estado, memoria disponible y presion CPU/memoria/E/S.
Incluyen los seis procesos de mayor CPU y procesos graficos/audio conocidos hasta
30 filas. Se refrescan solo PIDs principales enumerados en /proc, pidiendo CPU y
memoria; no se leen estadisticas de los hilos como procesos adicionales.
`processScope=process-leaders` identifica el muestreo corregido y
`sampleDurationMs` muestra su coste. Las capturas antiguas incluian filas de hilos
con el mismo RSS: no deben sumarse para estimar RAM total. No se recogen command
lines ni variables de entorno.

Los recolectores tambien consumen recursos. Los limites reducen su impacto pero
pueden cambiar el timing del fallo; para comparar puede desactivarse captura o
aumentarse el intervalo. No se ha automatizado ningun navegador para validarlo.

## Consultar un incidente

El lector recorre las copias y el archivo activo en streaming, manteniendo solo
las ultimas coincidencias. `--level` es nivel minimo, `--source` es prefijo y
`--event`/`--session` son coincidencias exactas. Tiempo se refiere a captura nativa;
consulta la hora original del journal en data para mensajes historicos.

```sh
node scripts/logs.mjs --minutes 10 --limit 200
node scripts/logs.mjs --source ui --event incident.mark
node scripts/logs.mjs --minutes 10 --level warn
node scripts/logs.mjs --source catalog --minutes 10 --json
node scripts/logs.mjs --source os.kernel --since 2026-09-16T18:00:00Z
```

No hay eventos retroactivos de UI para incidentes anteriores al despliegue. Tras
compilar y reiniciar TTY1, reproducir el fallo y marcarlo con F8. La ausencia de
errores registrados no demuestra ausencia de corrupcion visual.