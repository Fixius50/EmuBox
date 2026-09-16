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
| Magnet btih | qBittorrent-nox | Necesita motor instalado, metadata y peers/seeds |
| URL torrent directa | HTTP para descriptor, BitTorrent para contenido | Descriptor limitado a 16 MiB; no basta con obtener el torrent |
| Pixeldrain archivo publico | Conector documentado + HTTP | Respeta limites, autorizacion y CAPTCHA; no resuelve listas ni carpetas |
| 1fichier archivo | API de cuenta + HTTP, desactivada por defecto | Solo opt-in explicito, credencial privada y oferta con permiso de descarga API |
| Otros hostings | Sin conector registrado | Error con dominio concreto; no promesa de descarga |
| Otros protocolos | Sin proveedor registrado | Permanecen identificados como no compatibles en el catalogo |

No existe garantia de obtener cualquier juego por aparecer en un manifiesto.
`available` es disponibilidad declarada; `downloadable` significa que hay una
ruta de proveedor compatible, no que la fuente remota ya haya sido comprobada.

## Auditoria de cobertura, 15 de septiembre de 2026

Contrastada contra `services/downloads/resolver.rs`, `connectors.rs`,
`providers/http.rs`, `providers/qbittorrent.rs` y las fuentes persistidas.
No se consultaron servidores remotos, cuentas, juegos ni torrents publicos.
`DownloadService` sigue existiendo como fachada; la implementacion ya esta
separada. El README y los requisitos que negaban BitTorrent o daban la fase
por completada se han sincronizado con este contrato.

| Tipo de fuente | Deteccion actual | Conector / proveedor | Cuenta | Resultado o fallo |
| --- | --- | --- | --- | --- |
| Fuente `available=false` | Primera guarda del resolver | Ninguno | No se consulta | No descargable, aunque la URI tenga transporte compatible |
| HTTP(S) con extension reconocida | Path termina en zip, 7z, rar, iso, chd, pkg, exe, bin, gz, xz, rvz, gba o sfc; fuera de hosts bloqueados | HTTP nativo | No exige cuenta local | 403/404, timeout, TLS, HTML o identidad incorrecta fallan; no prueban falta de proveedor |
| HTTP(S) dinamico/desconocido | HTTP(S) restante, `unverified_http` | HTTP nativo, sin conector | No exige cuenta local | No descubre enlaces dentro de paginas; HTML se rechaza. Un dominio desconocido NO se anuncia como hosting integrado |
| `.torrent` directo | Path termina en `.torrent`, despues de comprobar hosts conocidos | HTTP descriptor (16 MiB) + qBittorrent | Solo API local; tracker puede exigir permisos | Sin motor, descriptor valido, metadata o peers no termina |
| Magnet btih | `magnet:` con `xt=urn:btih:` valido (40 hex o 32 base32) | qBittorrent | Solo API local; restricciones del tracker se conservan | Magnet invalido, solo btmh/v2 o motor ausente no descargable |
| Pixeldrain `/u/ID` | Dominio exacto, ID alfanumerico, sin credenciales/puerto no estandar | `pixeldrain_public_file` -> HTTP `/api/file/ID` | No exigida por EmuBox | Cuota, CAPTCHA, 403/404 o red fallan sin eludir limites |
| Pixeldrain `/api/file/ID` | Path API en dominio exacto | HTTP directo | No exigida por EmuBox | Respuesta real decide; no se valida existencia al listar |
| Pixeldrain listas/carpetas/otras rutas | `host_page`, sin conector para esas formas | Ninguno | No se consulta | No descargable; no se convierte una lista en un archivo |
| 1fichier `/?ID` | Dominio exacto y query alfanumerica | `1fichier_account_api` -> HTTP | Si, opt-in y credencial privada | Desactivado por defecto; cuenta/cuota/API pueden fallar aun con token local valido |
| Otras rutas 1fichier | `host_page` sin conector reconocido | Ninguno | No se consulta | No descarga automatica ni scraping |
| GoFile, MediaFire, Mega, MegaDB, Datanodes, Buzzheavier/bzzhr, Vikingfile, files.fm, akirabox, filekeeper | Dominio o subdominio reconocido como `host_page` | Sin conector implementado | Desconocida, no se consulta | Motivo explicito con dominio; no se inicia transferencia |
| 1337x, rutor, tapochek, t.me | Dominio o subdominio reconocido como `host_page` | Sin conector implementado | Desconocida, no se consulta | No se extraen magnets, torrents ni enlaces de las paginas |
| Otros protocolos | URI valida fuera de HTTP(S)/magnet | Sin proveedor | No se consulta | Conservados como no compatibles; motivo por protocolo |
| file, data, javascript, blob o URI invalida | Rechazados por normalizador/clasificador | Ninguno | No | No se importan como fuente remota ejecutable |

La deteccion se basa en la URI, no solo en `source_type` persistido. Los hosts
conocidos se comprueban antes del sufijo `.torrent`: una pagina de hosting con
ese sufijo no se convierte en torrent directo. Una URL dinamica que devuelve
un descriptor sin sufijo `.torrent` no se redirige automaticamente al motor torrent.
El soporte de transporte no implica soporte de todos los formatos recibidos.
`Content-Disposition` admite `filename=`, no un parser completo de `filename*`.
La deteccion de HTML usa Content-Type y el prefijo del primer bloque; no es una
inspeccion universal ni una certificacion de seguridad del contenido.

### Inventario local reproducido

Lectura de `download_sources` mediante SQLite en modo solo lectura, sin modificar
la biblioteca. Los dominios se agruparon con un parser URL sin publicar rutas,
parametros ni credenciales. Es una fotografia local, no una lista fija del producto.

| Tipo persistido | `available=1` | `available=0` |
| --- | ---: | ---: |
| HTTP | 262883 | 181807 |
| Magnet | 213756 | 8407 |
| Torrent | 12 | 0 |
| Total fuentes | 476651 | 190214 |

Son 666865 fuentes, no juegos unicos. Variantes y mirrors pueden corresponder
al mismo juego. Ni los totales ni el flag certifican disponibilidad remota.

Principales dominios, contando solo fuentes marcadas disponibles:

| Dominio | Fuentes | Interpretacion de cobertura |
| --- | ---: | --- |
| pixeldrain.com | 54834 | Hay conector para archivos; el recuento de dominio no garantiza que todas las rutas sean `/u/ID` |
| gofile.io | 38186 | Sin conector |
| www.mediafire.com | 35924 | Sin conector |
| datanodes.to | 27609 | Sin conector |
| 1fichier.com | 27261 | Conector restringido a formato valido y cuenta opt-in, desactivado |
| archive.org | 19866 | HTTP segun URI/respuesta; no conector de login ni resolucion de paginas |
| buzzheavier.com | 6232 | Sin conector |
| pastefg.hermietkreeft.site | 5995 | Dominio no reconocido: HTTP sin verificacion, no resolucion de pagina |
| 1337x.to (HTTP y HTTPS) | 5866 | Sin conector |
| tapochek.net (HTTP y HTTPS) | 6110 | Sin conector |
| vikingfile.com | 4899 | Sin conector |
| akirabox.com | 4785 | Sin conector |
| qiwi.gg | 3460 | Dominio no reconocido: HTTP sin verificacion |
| rutor.info (HTTP y HTTPS) | 3290 | Sin conector |
| megadb.net | 3037 | Sin conector |
| files.fm | 2802 | Sin conector |
| filekeeper.net | 1627 | Sin conector |

No se deduce un porcentaje de juegos descargables sumando estos grupos.
La siguiente ampliacion debe priorizar formatos de URL realmente presentes y
APIs publicas compatibles, con prueba de contrato y errores explicitos; no
activar cuentas ni integrar hostings a ciegas. Los errores de importacion de
manifiestos (403/404/451, red, limite 32 MiB) son otra etapa: no equivalen a un
fallo de descarga del juego ni autorizan borrar el catalogo previo.

## Configuracion sin registro

EmuBox no exige registro, inicio de sesion ni suscripcion para las descargas
publicas o la preparacion local. HTTP nativo, qBittorrent (GPL) y libarchive
(BSD) son componentes abiertos. No hay que crear cuentas en ellos ni en EmuBox.
Pixeldrain es un servicio de terceros, no se presenta como infraestructura abierta:
su API publica de lectura puede imponer limites o CAPTCHA. BitTorrent publico no
requiere cuenta en el cliente; un tracker privado puede exigir autorizacion.

Los conectores con cuenta estan desactivados salvo `EMUBOX_ALLOW_ACCOUNT_DOWNLOADS=1`.
Sin esa activacion no se lee la credencial de 1fichier ni se consulta su API,
aunque exista un token antiguo. La configuracion recomendada es dejar la variable
sin definir (o en 0). qBittorrent y Jackett usan credenciales locales generadas;
no son cuentas externas ni deben copiarse a la UI o al chat.

Un cliente abierto no puede garantizar disponibilidad ni quitar requisitos de un
hosting. GoFile y 1fichier no son requisitos de EmuBox: se puede seleccionar otra
fuente publica del mismo juego cuando exista; nunca se cambia de fuente sin permiso.
Los archivos locales tampoco requieren red ni cuenta. No se crean cuentas invitadas,
se compran creditos ni se instalan gestores propietarios como sustituto automatico.
Referencias: [qBittorrent](https://www.qbittorrent.org/), [libarchive](https://www.libarchive.org/),
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

## qBittorrent y Jackett locales

qBittorrent-nox >=5.0 sustituye a aria2. Magnet sigue usando BitTorrent: se cambia
el cliente, no el protocolo. El valor SQLite `bittorrent` y los IDs de trabajo
siguen siendo validos. HTTP no cambia. Para instalar el motor en Arch:

```bash
sudo pacman -S --needed qbittorrent-nox
```

Cada trabajo tiene su perfil en `.emubox-staging/<job-id>/qbittorrent/profile` y
payload separado. API v2 en puerto efimero 127.0.0.1, credencial aleatoria PBKDF2
SHA512, autenticacion local, validacion de Host y CSRF activas. No se usa la
instancia personal. Se admiten cookies SID (5.0/5.1) y QBT_SID_* (5.2).
El proceso termina con su supervisor; pausa/cancelacion solicitan parada y cierre.
Un cierre forzado conserva datos para revalidacion, no promete reanudacion exacta.

La bajada tiene `dl_limit=0`: sin tope fijo. La subida mantiene 64 KiB/s y se
solicita parada al completar (puede haber intercambio de piezas mientras se detecta
la finalizacion). Prioridad nice=5 e ionice best-effort 7, sin cuota de CPU.
No se alteran puertos del router ni firewall. Estado cada 500ms; lista de archivos
solo al terminar. Progreso reutiliza conexion SQLite/sentencia preparada.
Los archivos se validan antes de publicarlos: rutas relativas, regulares, completas
y confinadas. Las comprobaciones posteriores de checksum/preparacion se conservan.

Parciales de aria2: no se borran, renombran ni anexan. Una reanudacion posterior
usara el subdirectorio qBittorrent y puede volver a descargar contenido; comprueba
espacio libre antes. Instalaciones ya publicadas no necesitan transferirse de nuevo.
No se desinstala el paquete aria2 del SO automaticamente. Pausa los trabajos antes
de reiniciar la sesion para aplicar la migracion.

### Jackett

Jackett no es un motor de descarga. La ficha ofrece busqueda explicita para el
juego/version y categorias consola/PC; devuelve hasta 50 candidatos magnet de los
indexadores configurados. Se revisa y anade una fuente antes de pulsar Descargar.
No se importan automaticamente resultados, ni se confunde busqueda textual con
identificacion canonica. Resultados caducan a diez minutos y pertenecen a una version.
Los enlaces privados sin magnet no se exponen como una via de descarga alternativa.

Para instalar el servicio gestionado:

1. Ejecuta `sudo bash scripts/setup-search-services.sh`. Instala qBittorrent si
	falta y obtiene la distribucion oficial de [Jackett](https://github.com/Jackett/Jackett/releases)
	x86_64/ARM64, exigiendo el digest SHA-256 de la publicacion GitHub por HTTPS.
	Sin digest no instala; comprueba rutas y rechaza enlaces antes de publicar.
	La huella verifica integridad del artefacto publicado, no una firma independiente.
	Un HTTP 403 de GitHub detiene la instalacion, sin pedir tokens. Alternativa:
	proporcionar `JACKETT_ARCHIVE` y `JACKETT_SHA256` verificado al instalador mediante
	`sudo env JACKETT_ARCHIVE=/ruta/archivo.tar.gz JACKETT_SHA256=<digest> bash scripts/setup-search-services.sh`.
2. Jackett queda en `/opt/jackett`, propiedad root, sin escritura de grupo/otros.
	Una instalacion existente no se sobrescribe. No se ejecutan scripts remotos.
3. Se configura `emubox-jackett.service`, sin reiniciar TTY1.
4. Accede localmente a `http://127.0.0.1:9117` y configura indexadores autorizados.
	Desde otra maquina usa un tunel SSH local, no expongas 9117 en la LAN/Internet.

El instalador guarda la contraseña administrativa en
`/var/lib/emubox/jackett/admin-password` y la clave API en
`/var/lib/emubox/jackett/Jackett/ServerConfig.json`. Consulta la contraseña
directamente en tu terminal; no la pegues en el chat. EmuBox solo lee el archivo
local privado. No acepta endpoints remotos ni claves por IPC. Jackett se ejecuta
con usuario dedicado, sin home personal, filesystem protegido y escritura solo en
su directorio. Las actualizaciones automaticas y acceso externo quedan desactivados.

No se configuran trackers, FlareSolverr, proxies ni servicios de pago de forma
automatica. No tener indexadores devuelve lista vacia; errores de servicio o API
se muestran como errores, no como resultados vacios fabricados. API timeout 30s,
respuesta 2 MiB; una consulta simultanea como maximo. Configurar Jackett requiere
el servicio instalado: la presencia del boton no demuestra disponibilidad.

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
sandbox de preparacion, validacion y orquestacion. La ejecucion pertenece solo a
`runtime/game_sandbox`, no al preparador. El marcador
`PublishedDownload.installation` es opcional para leer paquetes anteriores y registra
tipo Inno/PS3 y raiz relativa. La publicacion conserva los originales y exige
seleccion explicita del ejecutable preparado.

EXE con firma MZ se envia a innoextract, que solo admite sus versiones soportadas de
Inno Setup. No ejecuta el instalador ni sus acciones de registro o prerrequisitos.
Los candidatos Windows son archivos PE x86/x64 bajo app, excluyendo setup/unins y DLL.
Wine usa un prefijo privado por juego/emulador dentro del sandbox y desactiva la
descarga automatica de Mono/Gecko. No se crea un prefijo junto al paquete ni se
reutiliza el home del usuario. No es una promesa de compatibilidad de todos los EXE.
Ver [aislamiento de ejecucion](execution-sandbox.md).

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
prueba opcional qBittorrent usa el motor real, un torrent privado de cuatro bytes y
webseed localhost, con DHT y descubrimiento desactivados; no consulta juegos reales.

Pendientes: conectores adicionales compatibles con acceso publico, instaladores EXE
distintos de Inno, validacion PKG con juegos, descriptores multidisco y validacion por plataforma. GoFile y pruebas autenticadas
de 1fichier quedan excluidos de la configuracion sin cuentas solicitada.
Tambien deben auditarse politicas de acceso de red para manifiestos no confiables,
cuotas de disco y aislamiento reforzado de procesos antes de exponer esto a usuarios
no confiables. No se evaden CAPTCHA, login, limites ni restricciones de terceros.