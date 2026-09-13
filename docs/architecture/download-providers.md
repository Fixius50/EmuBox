# Proveedores de descarga

## Separacion de responsabilidades

- Catalogo: normaliza manifiestos y conserva juego, fuente, URI y disponibilidad declarada. Importar no descarga juegos.
- `download_resolver`: clasifica el localizador y comprueba proveedor/conector. Formato de contenido y transporte no son equivalentes.
- `download_connectors`: traduce accesos publicos documentados. Por ahora, Pixeldrain `/u/id` a su API publica de archivo.
- `download_manager`: coordina una transferencia activa, cola FIFO, controles, snapshot de fuente, verificacion, publicacion y recuperacion.
- `download_providers`: contrato de transferencia/control/progreso; implementaciones HTTP y BitTorrent separadas.
- `download_preparation`: verificacion SHA-256 y preparacion por contenido, no por tecnologia de transporte.

## Capacidad actual

| Fuente | Proveedor | Limites |
| --- | --- | --- |
| HTTP/HTTPS | HTTP nativo | Sujeto a respuesta real; HTML no se acepta como juego |
| Magnet btih | BitTorrent con aria2 | Necesita motor instalado, metadata y peers/seeds |
| URL torrent directa | HTTP para descriptor, BitTorrent para contenido | Descriptor limitado a 16 MiB; no basta con obtener el torrent |
| Pixeldrain archivo publico | Conector documentado + HTTP | Respeta limites, autorizacion y CAPTCHA; no resuelve listas ni carpetas |
| Otros hostings | Sin conector registrado | Error con dominio concreto; no promesa de descarga |
| Otros protocolos | Sin proveedor registrado | Permanecen identificados como no compatibles en el catalogo |

No existe garantia de obtener cualquier juego por aparecer en un manifiesto.
`available` es disponibilidad declarada; `downloadable` significa que hay una
ruta de proveedor compatible, no que la fuente remota ya haya sido comprobada.

## Motor BitTorrent

La dependencia de ejecucion es el paquete `aria2`, opcional en el instalador:

```bash
sudo pacman -S --needed aria2
```

Se usa una instancia por trabajo, RPC exclusivamente loopback y secreto aleatorio
en archivo privado. No se carga configuracion del usuario ni netrc. Los secretos
y respuestas completas del motor no se imprimen en logs. Se apaga el proceso al
completar/interrumpir y se conservan controles para reanudar.

BitTorrent puede subir piezas durante la descarga (limite configurado: 64 KiB/s).
No hace seeding despues de completar. No se alteran puertos del router/firewall.
Los torrents privados mantienen las restricciones propias del motor. Magnet v2
sin btih no se anuncia como soportado por esta integracion.

## Integridad, staging e instalacion

Cada trabajo conserva fuente JSON y proveedor en `download_execution`. Cambiar un
manifiesto no sustituye la fuente del trabajo. Los estados persistidos se recuperan
como pausados tras reiniciar; no comienzan descargas automaticamente.

HTTP reanuda solo con ETag fuerte y rango coherente. Sin validador se empieza de
nuevo; no se anexan bytes a ciegas. Progreso se actualiza como maximo cada 500 ms.
El descriptor torrent y el contenido tienen directorios distintos.

Staging privado se situa en la plataforma de destino, excluido del watcher y del
escaner. El contenido se verifica antes de publicarse; los nombres/subdirectorios
se conservan. El destino es exclusivo por trabajo y no sobrescribe otros paquetes.
Un marcador de publicacion permite recuperar el paso filesystem/SQLite interrumpido.

ZIP se reconoce por firma, se extrae sin ejecutar codigo y rechaza escapes de ruta,
enlaces simbolicos, duplicados conflictivos y expansiones excesivas. Limites:
100.000 entradas, 100 GiB expandidos y ratio de expansion acotado. No se interpretan
EXE o PKG como instaladores ejecutables automaticamente. 7z/RAR y archivos ambiguos
pueden obtenerse, pero su preparacion automatica no esta implementada.
PS3 y descriptores CUE/M3U requieren preparacion explicita: no se presupone que
sus archivos dependientes o instalacion esten completos por existir un fichero.
Cancelar elimina solo staging del trabajo; pausa y fallo conservan los parciales.

`completed` significa que la preparacion encontro un candidato de lanzamiento
no ambiguo; no certifica compatibilidad del emulador. `downloaded` significa que
los archivos se obtuvieron pero requieren preparacion/seleccion, sin marcar el
juego como instalado. Los paquetes gestionados no se importan otra vez desde el
watcher. La UI muestra fase, proveedor, controles y ruta del contenido pendiente.

## Verificacion y limites pendientes

Las pruebas HTTP utilizan servidores locales y SQLite temporal. Cubren snapshot
de fuente, checksum antes de publicacion, rango/ETag, ZIP y rutas peligrosas. La
prueba opcional BitTorrent usa aria2 real, un torrent privado de cuatro bytes y
webseed localhost, con DHT y descubrimiento desactivados; no consulta juegos reales.

Pendientes: conectores adicionales segun API autorizada, preparacion 7z/RAR/PKG,
seleccion interactiva de archivos de paquetes ambiguos y validacion por plataforma.
Tambien deben auditarse politicas de acceso de red para manifiestos no confiables,
cuotas de disco y aislamiento reforzado de procesos antes de exponer esto a usuarios
no confiables. No se evaden CAPTCHA, login, limites ni restricciones de terceros.