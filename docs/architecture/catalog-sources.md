# Catalogo y base de juegos

La biblioteca utiliza juegos escaneados o importados en SQLite. No se mezclan
datasets sinteticos, no hay un total fijo de 10.000 y estar en el catalogo no
equivale a tener un juego instalado.

## Identidad canonica

La tarjeta y carpeta de la UI representan `canonical_games`. Sus versiones son
filas concretas de `games` enlazadas mediante `catalog_game_matches`; cada version
conserva sus propias `download_sources`, instalacion y asociaciones de emulador.
Descargar siempre usa el `gameId` de la version elegida. El favorito pertenece a
la identidad canonica, mientras que el emulador sigue perteneciendo a la variante
instalada porque dos versiones pueden requerir motores o argumentos distintos.

Abrir la carpeta canonica presenta directamente sus variantes, con contador de
version y estado instalado/descargando. Ya no hay una tarjeta intermedia duplicada
que se llame "version de plataforma". Las tarjetas se materializan al acceder a
la carpeta, no para todas las variantes del catalogo al arrancar. La ficha muestra
el titulo canonico y la version seleccionada por separado; identifica si procede
de Libretro Database o del catalogo local. No se fusionan identidades diferentes
por parecido ni se divide una identidad canonica por anos del paquete.

`Fuentes de esta version` abre el selector en el `catalogGameId` elegido, incluso
si el backend ordena primero otra variante instalada. Se conservan los nombres
originales de las fuentes, sin sustituirlos por el nombre del juego. Con varias
versiones, LB/RB cambia version en el panel de fuentes aunque exista historial;
los controles de descarga siguen disponibles en sus botones. La consulta IPC
de opciones se ejecuta en un worker bloqueante, no en el hilo de la interfaz.

En la comprobacion de solo lectura del 16 de septiembre habia 8.078 variantes
enlazadas a Libretro y 253.664 a identidades locales; 10.966 carpetas tenian varias
variantes. Estos datos explican por que muchos titulos siguen conservando nombres
de manifiesto: disponer de la capa canonica no equivale a haber identificado
oficialmente todo el catalogo. No se ocultan las variantes locales ni se anuncian
como coincidencias maestras.

Para NES, SNES, GBA, N64, Mega Drive, 3DS, NDS, PSP, PS1, PS2, PS3, GameCube y
Dreamcast, el proveedor maestro es [Libretro Database](https://github.com/libretro/libretro-database),
publicado bajo CC BY-SA 4.0. Se importan sus DAT No-Intro/Redump sin credenciales.
Solo se enlazan coincidencias exactas y unicas de titulo de release (confianza 100)
o titulo canonico (90); nunca se hace fuzzy matching automatico. Plataformas no
cubiertas y nombres sin coincidencia reciben una identidad local derivada (60).

La sincronizacion se ejecuta despues de que la UI quede lista. Una fuente correcta
se comprueba como maximo una vez al dia y usa ETag/Last-Modified; un fallo se
reintenta tras una hora. Un error parcial conserva las plataformas importadas y
la biblioteca local. Los 13 cuerpos admitidos tienen un limite individual de
16 MiB. La base canonica no crea descargas ni reescribe IDs de catalogo existentes.

## Organizacion Rust

`services/library/game_database` mantiene una fachada en `mod.rs` con los mismos
puntos de entrada para comandos, arranque e importadores. Las responsabilidades
internas quedan separadas:

- `dat.rs`: lectura DAT y normalizacion de identidad, sin SQLite ni red.
- `repository.rs`: indice local, importacion transaccional, matching y consulta
	de versiones/fuentes. `CanonicalMatch` nombra identidad, release, metodo y
	confianza; ambos importadores comparten la prioridad y escritura del enlace.
- `sync.rs`: fuentes remotas, limites HTTP, validadores y cadencia de consulta.
- `tests.rs`: regresiones de parser y migracion con IDs operativos conservados.

Una mejora de confianza conserva el favorito de la identidad anterior en la
misma transaccion que cambia el enlace. Un juego todavia sin indice canonico
puede alternar su favorito local. Las pruebas de matching en memoria comprueban
prioridad de release, rechazo de ambiguedades y aislamiento por plataforma.
El indice local usa transaccion SQLite inmediata: reserva el escritor antes de
leer filas pendientes, evitando fallos de promocion de lectura a escritura cuando
coinciden importadores. Se cubre con seis indexadores concurrentes y variantes
compartidas en la base aislada de pruebas; el timeout del SO no se modifica.

## Archivo activo

El backend utiliza `/etc/emubox/download-links.txt` si contiene URLs HTTP/HTTPS
activas. Si solo contiene comentarios o no existe, utiliza
`/opt/emubox/data/download-links.txt`. No se sobrescribe configuracion existente.
Los comentarios de la lista no certifican la seguridad de ninguna fuente.

Se consultan hasta cuatro manifiestos simultaneamente, con conexion limitada a
5 segundos, peticion a 25 segundos y contenido a 32 MiB por manifiesto. SQLite
guarda cada manifiesto secuencialmente en una transaccion, sin crear trabajos
ni iniciar descargas. Los fallos se registran por numero de linea en stderr,
que en la appliance llega a `/var/log/emubox/session.log`.
El lote devuelve un error parcial si alguna linea falla, despues de procesar las
demas; las importaciones correctas ya persistidas y sus notificaciones se conservan.
Una sincronizacion concurrente se omite; un mutex invalidado devuelve error.

La sincronizacion ocurre al arrancar y cada seis horas. Cada manifiesto guardado
emite `library-updated`; la UI escucha mediante `@tauri-apps/api/event`, sin exigir
la variable global `window.__TAURI__`.

## Cache persistente e importacion incremental

SQLite conserva los metadatos entre sesiones. La UI carga esa copia local sin
esperar a la red. `manifest_http_cache` guarda URL, ETag, Last-Modified, huella
SHA-256 y fecha de la ultima comprobacion correcta, en la misma base que el catalogo.
Durante seis horas no se vuelve a consultar una fuente importada. Despues se
envian If-None-Match/If-Modified-Since: un HTTP 304 evita descargar el cuerpo JSON.
Si el servidor devuelve 200, se compara la huella antes de normalizar o importar.
La version de cache HTTP 2 invalida la version anterior para reevaluar la inferencia
de plataformas incluso con contenido remoto identico. No inicia descargas de juegos.

La plataforma explicita del elemento precede a la del manifiesto; ambas admiten
PS4 y 3DS. Despues se consideran etiquetas del titulo, palabras completas y nombre
de fuente, y finalmente extensiones no ambiguas del ultimo segmento HTTP o del
nombre `dn` de un magnet. No se usan dominios ni queries HTTP como evidencia.
`.pkg`, `.pbp`, `.rvz` y `.ciso` no determinan por si solos una plataforma.
Ante ausencia de evidencia se mantiene `pc` por compatibilidad del contrato;
no significa que se haya identificado el contenido ni comprobado su ejecucion.

Los servidores sin validadores requieren descargar su manifiesto cuando vence
la cache: no existe un protocolo universal de deltas que permita pedir solo filas.
La escritura en SQLite si es incremental: `manifest_entry_cache` compara entradas
normalizadas y solo actualiza las modificadas/nuevas. `manifest_sources` conserva
la pertenencia de cada URL a sus manifiestos. Las URLs retiradas se desactivan
solo si ningun otro manifiesto las mantiene. No se borran juegos, favoritos,
instalaciones ni historial de descargas.

La huella HTTP se guarda en la misma transaccion que las entradas, solo tras una
importacion correcta. Un error de red/JSON no destruye el catalogo anterior.
Los fallos HTTP/conexion tienen una espera de 15 minutos entre reintentos.
La primera sincronizacion con esta version necesita llenar las tablas de cache;
las siguientes reutilizan los datos. El comando `--import-catalog` respeta esos
plazos y devuelve solo fuentes modificadas, no el tamano total de la biblioteca.

## Titulos y paquetes

Cuando aun no existe una identidad canonica, una tarjeta puede reunir distintas
distribuciones o versiones de paquete del mismo titulo y plataforma. La agrupacion visual elimina marcas reconocibles
como Repack/Scene/License, su distribuidor, Build, Free Download, tiendas
(`GOG`, `Steam`, `Epic`) y etiquetas de idioma como `Ru/Multi`. Un sufijo explicito
de paquete `+ DLC` o `- Build <numero> + <nombre> DLC` se conserva en el nombre
original de la variante, pero no genera otra tarjeta del juego base.

Un año desconocido se agrupa con el unico año conocido del mismo titulo y
plataforma, en vez de crear un duplicado. Si existen varios anos conocidos
distintos, permanecen separados y las variantes sin año no se asignan a ninguno
arbitrariamente. Se conservan secuelas, regiones, ediciones y expansiones con
subtitulo propio; nunca se mezclan plataformas.
La inferencia visual es solo el fallback previo al indice persistente; una vez
asignado `canonicalId`, ese identificador decide la tarjeta aunque los titulos
difieran. Identidades canonicas distintas nunca se fusionan solo por compartir titulo.

Regresion comprobada con los titulos de la captura del 6 de septiembre de 2026:
las ocho variantes `#DRIVE Rally` y las ocho variantes `#BLUD` forman una tarjeta
por titulo/plataforma, con todos sus paquetes y fuentes accesibles. No se eliminan
filas de SQLite ni se vuelven a descargar manifiestos para aplicar esta correccion.

SQLite conserva cada registro y fuente original. La UI muestra titulos agrupados,
versiones e instalados por separado. `Fuentes de esta version` permite consultar las
alternativas incluso cuando una variante ya esta instalada. El selector conserva
los titulos originales y confirma con `gameId` y `sourceId` de la variante elegida,
no con el ID de la tarjeta representativa. No descarga todos los paquetes juntos.

Sin abrir la interfaz ni descargar juegos:

```bash
bin/emubox --import-catalog
```

## Datos y descarga

Se admiten `downloads[]` con `title`/`uris` y `games[]` con plataforma/URL, o arrays
equivalentes. Los metadatos opcionales se conservan cuando existen; `uploadDate`
no se convierte en fecha de lanzamiento. La ausencia de portada, valoracion o
desarrollador no se disfraza con valores inventados. Los SVG de categoria sirven
como identificacion en filtros y tarjetas y como imagen alternativa sin portada.

Los comandos IPC `import_download_links`, `import_downloads_from_json` y
`import_downloads_from_url` devuelven `DownloadSource[]`, no `DownloadJob[]`.
El nombre legado `import_and_start_downloads` se conserva por compatibilidad,
pero ya no arranca descargas: devuelve los trabajos existentes despues de importar.

`get_download_sources(gameId)` devuelve todas las fuentes guardadas junto a
`access`, `downloadable` y `reason`. La UI abre un selector antes de descargar.
`download_game(gameId, sourceId)` valida que la fuente pertenezca a ese juego;
sin `sourceId`, rechaza juegos con multiples fuentes en lugar de elegir una.
Las URLs pueden ser partes o versiones distintas, no se agrupan automaticamente
como espejos ni se descargan todas juntas.

Se distinguen candidatos HTTP por extension, HTTP sin verificar, paginas de
alojamientos conocidos y BitTorrent. Esta clasificacion es heuristica, no una
comprobacion de disponibilidad ni de seguridad. Las paginas de alojamiento conocidas
se bloquean hasta tener un conector. Un candidato HTTP aun puede fallar, redirigir
o devolver HTML; el descargador mantiene sus comprobaciones HTTP y TLS.
Magnet/torrent disponen de proveedor BitTorrent con aria2. Si falta el ejecutable,
se indica el paquete necesario sin anunciar descarga disponible. Pixeldrain tiene
conector publico de archivo; 1fichier dispone de conector API opcional, desactivado
por defecto para mantener el uso sin registro. Solo se activa mediante opt-in
explicito y cuenta propia autorizada; no es requisito de EmuBox. Otros hostings
siguen bloqueados por conector ausente. Elegir un candidato local no consulta al
hosting ni altera la identidad de la fuente seleccionada.
Una pagina HTML no se considera un juego y no se eluden login, CAPTCHA o limites.
Ver [proveedores y preparacion](download-providers.md).

La importacion de las 71 URLs del usuario registro 167.901 juegos y 278.460
fuentes unicas el 5 de septiembre de 2026; la cola de descargas siguio vacia.
Son recuentos de ese momento, no constantes ni garantia de descargas funcionales.
El informe local de esa ejecucion esta en `reports/catalog-import.log`.

## Normalizacion y auditoria

La auditoria de 71 fuentes pudo analizar 29 manifiestos con raiz `name`/`downloads`.
Los otros fallaron por HTTP, DNS/TLS/conexion o limite de 32 MiB; no se desactiva
TLS ni se aumenta indiscriminadamente ese limite.

`manifest_service` normaliza tambien `games[]` y arrays: titulo/nombre,
`genre`/`genres[]`, `releaseYear`/`year` numerico o textual y `coverImage`/`cover`.
El texto literal `null` o `undefined` se convierte en ausencia de dato.
`descriptionHtml` se convierte a texto mediante scraper/html5ever, sin scripts,
estilos ni insercion HTML en la UI. No se usa uploadDate como ano del juego.

Se conserva cada URI valida distinta por juego con identificador estable, sin
perder alternativas en cada importacion. El tipo torrent se determina por el
path `.torrent`, no por encontrar la palabra torrent en el dominio. La plataforma
continua siendo inferida cuando el manifiesto no la declara; no se ha verificado
manualmente la clasificacion de todos los juegos.

La reimportacion normalizada mantuvo 167.901 juegos y registro 543.961 fuentes;
elimino 6.804 descripciones con texto `null`. Durante la operacion un proceso de
produccion antiguo creo otro intento fallido a MegaDB (0 bytes). El importador
no crea trabajos; sus tests lo verifican en una base aislada.
Informe de esta ejecucion: `reports/catalog-normalization.log`.