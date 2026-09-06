# Catalogo desde manifiestos

La biblioteca utiliza juegos escaneados o importados en SQLite. No se mezclan
datasets sinteticos, no hay un total fijo de 10.000 y estar en el catalogo no
equivale a tener un juego instalado.

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

Una tarjeta puede reunir distintas distribuciones o versiones de paquete del
mismo titulo y plataforma. La agrupacion visual elimina marcas reconocibles
como Repack/Scene/License, su distribuidor, Build, Free Download, tiendas
(`GOG`, `Steam`, `Epic`) y etiquetas de idioma como `Ru/Multi`. Un sufijo explicito
de paquete `+ DLC` o `- Build <numero> + <nombre> DLC` se conserva en el nombre
original de la variante, pero no genera otra tarjeta del juego base.

Un año desconocido se agrupa con el unico año conocido del mismo titulo y
plataforma, en vez de crear un duplicado. Si existen varios anos conocidos
distintos, permanecen separados y las variantes sin año no se asignan a ninguno
arbitrariamente. Se conservan secuelas, regiones, ediciones y expansiones con
subtitulo propio; nunca se mezclan plataformas.
La identidad se infiere de titulos, no de un identificador universal: algunas
variantes seguiran separadas para evitar mezclar juegos por parecido.

Regresion comprobada con los titulos de la captura del 6 de septiembre de 2026:
las ocho variantes `#DRIVE Rally` y las ocho variantes `#BLUD` forman una tarjeta
por titulo/plataforma, con todos sus paquetes y fuentes accesibles. No se eliminan
filas de SQLite ni se vuelven a descargar manifiestos para aplicar esta correccion.

SQLite conserva cada registro y fuente original. La UI muestra titulos agrupados,
paquetes e instalados por separado. `Fuentes y paquetes` permite consultar las
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
Magnet/torrent se registran para catalogo pero su descarga se rechaza con
un mensaje claro: el motor BitTorrent no esta implementado. Una pagina HTML no
se considera un archivo de juego. Tampoco se resuelven automaticamente portales
con login, captchas o pasos intermedios.

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