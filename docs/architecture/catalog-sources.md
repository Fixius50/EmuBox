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

`download_game` crea el trabajo cuando el usuario lo solicita y prefiere fuentes
HTTP. Magnet/torrent se registran para catalogo pero su descarga se rechaza con
un mensaje claro: el motor BitTorrent no esta implementado. Una pagina HTML no
se considera un archivo de juego. Tampoco se resuelven automaticamente portales
con login, captchas o pasos intermedios.

La importacion de las 71 URLs del usuario registro 167.901 juegos y 278.460
fuentes unicas el 5 de septiembre de 2026; la cola de descargas siguio vacia.
Son recuentos de ese momento, no constantes ni garantia de descargas funcionales.
El informe local de esa ejecucion esta en `reports/catalog-import.log`.