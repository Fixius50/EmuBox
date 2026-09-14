# Proveedores de descarga

## Separacion de responsabilidades

- Catalogo: normaliza manifiestos y conserva juego, fuente, URI y disponibilidad declarada. Importar no descarga juegos.
- `download_resolver`: clasifica el localizador y comprueba proveedor/conector. Formato de contenido y transporte no son equivalentes.
- `download_connectors`: traduce Pixeldrain `/u/id` a su API publica y enlaces de archivo 1fichier mediante la API de una cuenta autorizada.
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
| 1fichier archivo | API de cuenta + HTTP, desactivada por defecto | Solo opt-in explicito, credencial privada y oferta con permiso de descarga API |
| Otros hostings | Sin conector registrado | Error con dominio concreto; no promesa de descarga |
| Otros protocolos | Sin proveedor registrado | Permanecen identificados como no compatibles en el catalogo |

No existe garantia de obtener cualquier juego por aparecer en un manifiesto.
`available` es disponibilidad declarada; `downloadable` significa que hay una
ruta de proveedor compatible, no que la fuente remota ya haya sido comprobada.

## Configuracion sin registro

EmuBox no exige registro, inicio de sesion ni suscripcion para las descargas
publicas o la preparacion local. HTTP nativo, aria2 (GPL-2.0-or-later) y libarchive
(BSD) son componentes abiertos. No hay que crear cuentas en ellos ni en EmuBox.
Pixeldrain es un servicio de terceros, no se presenta como infraestructura abierta:
su API publica de lectura puede imponer limites o CAPTCHA. BitTorrent publico no
requiere cuenta en el cliente; un tracker privado puede exigir autorizacion.

Los conectores con cuenta estan desactivados salvo `EMUBOX_ALLOW_ACCOUNT_DOWNLOADS=1`.
Sin esa activacion no se lee la credencial de 1fichier ni se consulta su API,
aunque exista un token antiguo. La configuracion recomendada es dejar la variable
sin definir (o en 0). En este equipo aria2 1.37.0 y libarchive 3.8.9 estan instalados;
no hace falta provisionar claves ni instalar un servicio de descargas adicional.

Un cliente abierto no puede garantizar disponibilidad ni quitar requisitos de un
hosting. GoFile y 1fichier no son requisitos de EmuBox: se puede seleccionar otra
fuente publica del mismo juego cuando exista; nunca se cambia de fuente sin permiso.
Los archivos locales tampoco requieren red ni cuenta. No se crean cuentas invitadas,
se compran creditos ni se instalan gestores propietarios como sustituto automatico.
Referencias: [aria2](https://aria2.github.io/), [libarchive](https://www.libarchive.org/),
[API Pixeldrain](https://pixeldrain.com/api).

## API autenticada 1fichier (opcional)

Solo se reconocen enlaces `https://1fichier.com/?ID` con ID alfanumerico. Solo si
se decide activar esta integracion con cuenta, el administrador debe proporcionar
su propia clave API en
`/etc/emubox/credentials/1fichier.token`: archivo regular, sin enlaces simbolicos,
permisos 0600 o 0400, legible por el usuario que ejecuta EmuBox. El directorio debe
ser privado y administrado localmente. No se crea ninguna credencial ni se solicita
por chat; no se guarda en el catalogo, IPC o logs. Sin ella la fuente no se anuncia
como descargable. Tener clave no certifica suscripcion ni permisos remotos.

El conector usa POST HTTPS a `/v1/download/get_token.cgi`, Bearer solo hacia la API,
sin redirecciones autenticadas ni reintentos automaticos, con timeout de 20 segundos
y respuesta limitada a 64 KiB. Solo admite enlaces HTTPS del dominio 1fichier.
La solicitud usa `cdn=0`, pero la oferta puede forzar consumo de creditos CDN;
la interfaz advierte esta posibilidad. No resuelve carpetas, contrasenas de archivo
ni acceso gratuito mediante scraping. La cancelacion durante la consulta API puede
tardar hasta el timeout; la transferencia posterior respeta los controles HTTP.
Al renovar una URL temporal cambia la identidad HTTP: el parcial se reinicia en
lugar de anexar contenido cuya identidad no pueda demostrarse.

GoFile sigue sin conector: su API exige token y acceso Premium para listar contenido.
No se crean cuentas invitadas ni se intenta sustituir permisos mediante scraping.
Referencias: [1fichier API](https://1fichier.com/api.html),
[GoFile API](https://gofile.io/api), [libarchive](https://www.libarchive.org/).

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
EXE arbitrarios como instaladores ejecutables: los formatos admitidos se delegan
a preparadores especificos, descritos mas abajo.
7z y RAR4/RAR5 se reconocen por firma y pasan por `compress-tools`/libarchive;
EmuBox escribe los archivos regulares, sin usar extraccion a disco de libarchive.
Se comprueban rutas, tipos, duplicados y tamanos declarados/reales; no se crean
enlaces ni se restauran permisos del paquete. Rigen 100.000 entradas, 100 GiB y
ratio total de expansion 1000. Las llamadas internas de descompresion no son
interrumpibles; el control se comprueba entre entradas/bloques. No hay soporte
explicito para contrasenas, ensamblado multivolumen ni extraccion anidada.
Si falla la preparacion se conservan los originales como `downloaded`.
La dependencia nativa es `libarchive` (Arch), `libarchive-dev` para compilar en
Debian/Ubuntu; `libarchive-tools` aporta bsdtar para las pruebas locales.
PS3 y descriptores CUE/M3U requieren preparacion explicita: no se presupone que
sus archivos dependientes o instalacion esten completos por existir un fichero.
Cancelar elimina solo staging del trabajo; pausa y fallo conservan los parciales.

`completed` significa que la preparacion encontro un candidato de lanzamiento
no ambiguo; no certifica compatibilidad del emulador. `downloaded` significa que
los archivos se obtuvieron pero requieren preparacion/seleccion, sin marcar el
juego como instalado. Los paquetes gestionados no se importan otra vez desde el
watcher. La UI muestra fase, proveedor, controles y ruta del contenido pendiente.
Los trabajos ya publicados como `downloaded` no se reprocesan automaticamente
al actualizar. La accion "Reintentar preparacion local" del selector (o RB con
el mando) reutiliza `resume_download`: verifica el paquete publicado y prepara
sin consultar el proveedor, incluso si la fuente remota ya no esta disponible.
Se conservan originales y checksum disponible. Si hay varios candidatos de formatos
admitidos, la preparacion conserva los archivos extraidos y sigue como `downloaded`.
El selector muestra "Archivo de lanzamiento" y exige confirmar uno. Con mando,
LB enfoca la lista, arriba/abajo cambia de candidato y A confirma; izquierda vuelve
a las fuentes. Puede cambiarse el candidato de un paquete ya preparado.
`get_download_candidates` y `select_download_candidate` validan pertenencia al
trabajo, snapshot de fuente, rutas y existencia de archivos. No aceptan rutas
arbitrarias, enlaces simbolicos finales ni un trabajo activo. Se reutiliza la lista
de formatos de lanzamiento por plataforma: seleccionar no convierte un EXE/PKG,
BIN/CUE/M3U o contenido PS3 en instalacion validada. La comprobacion de seleccion es
estructural; no certifica compatibilidad del emulador ni dispone de hashes individuales
para los archivos extraidos.
El contenido preparado ocupa un directorio nuevo y el marcador se sustituye mediante
rename antes de actualizar SQLite. Pausa/cancelacion no eliminan los originales.
Un fallo entre publicacion y marcador puede dejar un directorio preparado huerfano;
no se borra automaticamente ni se confunde con un paquete listo. La reanudacion de
un trabajo cuyo contenido publicado ha desaparecido falla sin volver a descargar.
## Preparadores de instalacion

`downloads/installers` divide deteccion y candidatos, descubrimiento de firmware,
configuracion de lanzamiento, sandbox, validacion y orquestacion. El marcador
`PublishedDownload.installation` es opcional para leer paquetes anteriores y registra
tipo Inno/PS3 y raiz relativa. La publicacion conserva los originales y exige
seleccion explicita del ejecutable preparado.

EXE con firma MZ se envia a innoextract, que solo admite sus versiones soportadas de
Inno Setup. No ejecuta el instalador ni sus acciones de registro o prerrequisitos.
Los candidatos Windows son archivos PE x86/x64 bajo app, excluyendo setup/unins y DLL.
Wine usa un prefijo por paquete y desactiva la descarga automatica de Mono/Gecko.
No es una promesa de compatibilidad de todos los EXE ni aislamiento de juegos.

PKG con firma PS3 se envia a RPCS3 `--headless --installpkg` en un entorno por trabajo.
El firmware se obtiene de `EMUBOX_PS3_FIRMWARE_DIR` o del dev_flash administrado por
EmuBox. El firmware PS3 oficial 4.93 ya fue instalado en este equipo; no se descarga
firmware ni licencias durante cada trabajo. Se exige mensaje de instalacion completa
en RPCS3.log y estructura instalada; el codigo de salida cero por si solo no basta.
Los candidatos requieren EBOOT.BIN, USRDIR y PARAM.SFO con firmas esperadas.
No se ha probado una instalacion completa con un juego PKG valido.

Los preparadores usan bubblewrap sin red, entrada y herramienta de solo lectura,
staging escribible y prlimit. Limites: 30 minutos, 1800 segundos CPU, 100 GiB por
archivo y comprobacion periodica de 100.000 entradas/100 GiB totales. El limite
de espacio virtual es 8 GiB para Inno y 128 GiB para RPCS3, no memoria residente.
No es una cuota estricta de disco. Se rechazan enlaces y archivos especiales y la
cancelacion termina el proceso aislado. La instalacion desconocida queda pendiente,
sin pedir registro en servicios externos.

Las rutas canonicas de descargas son `services/downloads/service/` (catalogo,
plataforma, repositorio y orquestacion), `manager/` (cola, transferencia y publicacion),
`providers/` e `installers/`. `services/mod.rs` conserva alias compatibles.

## Verificacion y limites pendientes

Las pruebas HTTP utilizan servidores locales y SQLite temporal. Cubren snapshot
de fuente, checksum antes de publicacion, rango/ETag, ZIP y rutas peligrosas.
7z se genera localmente con bsdtar y se prueba mediante el preparador integrado,
incluyendo enlaces y cancelacion. RAR tiene pruebas de firma y rechazo de truncados.
La prueba opcional `extracts_valid_rar5_fixture` verifica la muestra sintetica
`test_read_format_rar5_stored.rar` de libarchive v3.8.1, proporcionada localmente
mediante `EMUBOX_RAR5_FIXTURE`; extrae el texto esperado byte a byte sin red en el test.
RAR5 almacenado y comprimido se han validado. La prueba opcional
`extracts_compressed_rar5_fixture` usa `EMUBOX_RAR5_COMPRESSED_FIXTURE` con
`test_read_format_rar5_compressed.rar` de la misma version y comprueba los 1.200 bytes.
RAR4 y otras variantes (solid, multivolumen, cifrados) no se han validado integralmente.
La prueba del gestor verifica reanudacion sin fuente disponible, checksum fallido,
cancelacion, conservacion de originales y recuperacion del marcador. 1fichier
tiene pruebas locales de contrato, destinos y permisos, no prueba autenticada real. La
prueba opcional BitTorrent usa aria2 real, un torrent privado de cuatro bytes y
webseed localhost, con DHT y descubrimiento desactivados; no consulta juegos reales.

Pendientes: conectores adicionales compatibles con acceso publico, instaladores EXE
distintos de Inno, validacion PKG con juegos, descriptores multidisco y validacion por plataforma. GoFile y pruebas autenticadas
de 1fichier quedan excluidos de la configuracion sin cuentas solicitada.
Tambien deben auditarse politicas de acceso de red para manifiestos no confiables,
cuotas de disco y aislamiento reforzado de procesos antes de exponer esto a usuarios
no confiables. No se evaden CAPTCHA, login, limites ni restricciones de terceros.