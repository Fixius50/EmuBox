# EmuBox: informe tecnico para el equipo de IA

Fecha: 13 de septiembre de 2026.

## 1. Que es y que no es

EmuBox es un entorno de consola sobre Linux existente. Combina una aplicacion nativa, una interfaz operable con mando y scripts que convierten una instalacion de Arch en una appliance dedicada. No es un kernel propio, una implementacion de drivers ni un emulador universal.

Conviene separar tres entregables:

1. Runtime: binario Tauri/Rust, frontend SolidJS, IPC, SQLite y servicios de juegos.
2. Appliance: usuario, permisos, directorios, autologin, sesion Wayland, audio y arranque con systemd.
3. Distribucion: imagen instalable reproducible, particionado, bootloader, kernel, actualizaciones de base y recuperacion. Esta tercera capa no esta terminada y no debe presentarse como una ISO de EmuBox validada.

La instalacion manual soportada parte de Arch Linux x86_64 o Arch Linux ARM aarch64. El proyecto contiene configuracion y validaciones para ambas arquitecturas, pero eso no demuestra que una appliance ARM real haya arrancado correctamente.

## 2. Arranque y ciclo de vida

La ruta instalada de consola es:

```text
Firmware y bootloader del equipo
  -> kernel Linux y systemd
  -> getty@tty1 con autologin del usuario emubox
  -> perfil de shell, limitado a TTY1
  -> /usr/local/bin/emubox-session
  -> /opt/emubox/scripts/run.sh
  -> deteccion de aceleracion y seleccion de backend
  -> compositor compatible
  -> binario Tauri con frontend embebido
```

SSH es una via de mantenimiento, no el iniciador de la interfaz fisica. El servicio auxiliar `emubox.service` no debe iniciar otra copia junto a getty. El lanzador instalado delega en el script del repositorio, por lo que actualizar ese script cambia el siguiente arranque de sesion.

El compositor necesita dispositivos DRM accesibles, una salida utilizable, sesion y permisos del usuario. Que un ejecutable este instalado no demuestra que pueda tomar la pantalla. Reiniciar getty interrumpe la interfaz y puede interrumpir juegos: es una accion operativa que no debe ejecutarse sin coordinarla.

## 3. Arquitectura de la aplicacion

SolidJS mantiene el estado reactivo de XMB. La biblioteca ofrece categorias, carpetas, versiones por plataforma, ficha de juego y acceso a fuentes/paquetes. La navegacion se calcula en un servicio puro y un hook mantiene estado, busqueda y ventanas acotadas de elementos; no se renderiza el catalogo completo en el DOM.

Los componentes reciben datos y callbacks. Los hooks controlan entradas, foco y suscripciones. Los stores mantienen catalogo, ajustes y modales. `IEmuBoxBackend` describe el contrato y `TauriBackendService` lo transporta mediante la API oficial de Tauri.

El runtime requiere Tauri. Abrir solamente Vite en un navegador muestra que falta el runtime; no conecta automaticamente con SQLite ni fabrica hardware, juegos, mandos, procesos o actualizaciones. Para desarrollo integrado se utiliza `npm run tauri:dev` en una sesion grafica adecuada.

En Rust, `commands/` adapta IPC, `models/` define contratos y `services/` contiene responsabilidades. La deteccion grafica, su politica, configuracion, entrada, pantalla, audio, almacenamiento, BIOS, logs y energia se separan. Las reglas detalladas estan en `refactoring-and-architecture-guidelines.md`.

Los errores nativos no se convierten en respuestas ficticias. Los datos no medidos se representan como ausentes. Configuracion de fabrica y telemetria son conceptos distintos: una preferencia por defecto no demuestra un estado real del dispositivo.

## 4. Datos persistentes

```text
/opt/emubox         codigo, scripts y binarios
/etc/emubox         configuracion persistente
/var/lib/emubox     SQLite, juegos, partidas, BIOS y entornos de emuladores
/var/cache/emubox   caches y descargas temporales
/var/log/emubox     registros
/run/emubox         estado efimero recreado al arrancar
```

La biblioteca real procede de SQLite, los juegos escaneados y los manifiestos importados. Se han retirado los datasets generados y el backend simulado. Una biblioteca vacia debe seguir vacia; no se sustituye por miles de juegos de demostracion.

La agrupacion de paquetes es visual. Conserva IDs de registros y fuentes, plataformas y ediciones que no se pueden equiparar con seguridad. Seleccionar una fuente debe descargar el paquete elegido, no el ID representativo de otra variante ni todas las alternativas.

La configuracion se lee del sistema. Solo la ausencia del archivo permite usar valores de fabrica. Los errores de permisos y JSON invalido son errores reales. La escritura usa un temporal en el mismo directorio y renombrado para evitar dejar un archivo parcialmente escrito.

Las pruebas Rust utilizan una base temporal por proceso. Esta separacion es critica: en el pasado, pruebas de escaneo insertaron juegos en la base real. Nunca validar un cambio creando filas de prueba en la biblioteca del usuario.

## 5. Politica grafica y limites

La politica es GPU/3D utilizable, luego API compatible, luego compositor. La deteccion distingue `accelerated`, `software` e `indeterminate`. Un fallo, permiso insuficiente o herramienta ausente no demuestra ausencia de GPU. Ni la arquitectura CPU ni estar dentro de una VM determinan por si solas si hay GPU.

Las APIs actualmente son OpenGL/EGL y Vulkan en el sondeo nativo. Se prefiere OpenGL acelerado para la ruta Cage/WebKitGTK; Vulkan es otra posibilidad acelerada. No hay una promesa de soporte para cualquier API futura simplemente porque el contrato permita ampliarlo.

Si ambos sondeos concluyen mostrando solo software y el inventario es completo sin ambiguedad multigpu se puede elegir CPU. Un inventario fallido o varias GPU sin cobertura individual conservan incertidumbre. Si ninguno confirma aceleracion y alguno no concluye, el backend recomendado es `auto`, el diagnostico sigue indeterminado y se intenta Cage sin forzar renderer software. Si esa sesion falla, se conserva el error: no se relanza silenciosamente por CPU. `EMUBOX_RENDER_MODE=software` (o el archivo de modo de la appliance) permite fallback explicito ante incertidumbre, registrado aparte; nunca invalida aceleracion confirmada.

El inventario agrupa nodos DRM por ruta canonica de dispositivo, con PCI, driver y numeros de render node. Vulkan puede aportar `VK_EXT_physical_device_drm`; EGL puede aportar nodos DRM segun su salida/version. La correlacion utiliza esos identificadores, no nombres de GPU. Cuando faltan, varias GPU quedan sin asociacion; una sola GPU con inventario completo y evidencia no ambigua permite `single_device_inference`. Esa inferencia no es prueba de uso por el proceso.

`devices` y `probes.observations` conservan inventario y capacidades por separado. `selectedDeviceId` solo existe cuando las observaciones de la API preferida permiten una seleccion no ambigua; `activeDeviceId` permanece ausente sin evidencia real del proceso. No se combinan OpenGL de una GPU y Vulkan de otra para habilitar Gamescope. Falta de correlacion no impide usar OpenGL ya observado mediante Cage. No se fuerza una GPU mediante variables de offload ni se promete seleccionar la mas rapida.

El lanzador consulta el mismo servicio Rust mediante `--graphics-session`; `--graphics-info` ofrece el JSON completo sin abrir la UI ni consultar el catalogo. Si el detector nativo no puede ejecutarse, el arranque conserva el estado indeterminado y registra el motivo. Los campos resumidos de HardwareInfo son compatibilidad; `hardware.graphics` contiene la evidencia y el estado autoritativos.

La seleccion de compositor se hace aparte. Cage es automatico cuando puede utilizar el backend y DRM disponibles. Gamescope tiene requisitos propios, incluidos Vulkan acelerado y una salida compatible; detectar Vulkan no obliga a elegirlo. Puede solicitarse por preferencia compatible o servir como alternativa si Cage no soporta la ruta. Sin compositor compatible se informa del problema: no se oculta una GPU detectada pasando silenciosamente a CPU.

Los renderers `llvmpipe`, `softpipe`, `swrast`, `lavapipe` y `SwiftShader` son software. `SVGA3D` puede incluir la palabra `LLVM` sin ser llvmpipe. `virgl`, SVGA3D y otras GPU virtuales aceleradas no deben descartarse por ser virtuales. Mali es una familia de GPU; aarch64 es una arquitectura CPU.

El sondeo consulta dispositivos y respuestas de drivers, no solo paquetes instalados. Aun asi, enumerar dispositivos Vulkan o crear un contexto EGL no valida todos los shaders, extensiones, formatos, permisos KMS ni estabilidad de una sesion completa. Las capacidades son evidencia de disponibilidad, no un benchmark ni una certificacion universal.

La VM de desarrollo observada expone VMware SVGA II/vmwgfx, renderer SVGA3D y OpenGL acelerado, sin Vulkan utilizable. Rust y Shell coinciden en seleccionar OpenGL con Cage. El modelo Ryzen que aparece en la CPU no significa que la VM tenga acceso directo a su GPU Radeon.

EmuBox no puede crear passthrough, extensiones de GPU o memoria de video que el hipervisor no exponga. Activar aceleracion 3D y configurar VMSVGA corresponde al anfitrion. El passthrough requiere otra infraestructura y soporte IOMMU/hardware. Cambiar codigo frontend no convierte una GPU virtual limitada en una GPU fisica dedicada.

Los ajustes vmwgfx de cursor, DRM legacy y DMA-BUF son compatibilidad localizada; no certifican que desaparezca la corrupcion de imagen. Si falla el driver, puede ser necesario actualizar VirtualBox, Guest Additions, kernel/Mesa o el driver del anfitrion. No se puede prometer una solucion solo modificando EmuBox.

El backend seleccionado tampoco obliga a que todos los emuladores usen esa misma API. Cada motor tiene configuracion y requisitos propios. WebKitGTK y el compositor poseen sus mecanismos internos; se debe observar el renderer del proceso cuando se valida hardware real.

## 6. Emuladores, BIOS y ejecucion

EmuBox localiza ejecutables y cores, valida su arquitectura, consulta capacidades y prepara el lanzamiento. No implementa la emulacion de PS1, PS2, PS3, Nintendo o Sega. Esa responsabilidad es de DuckStation, PCSX2, RPCS3, RetroArch y los demas motores instalados.

Un binario x86_64 no se vuelve ARM por modificar una etiqueta. Hace falta una compilacion nativa compatible o una capa de traduccion independiente, con sus costes y limites. La disponibilidad de paquetes difiere entre Arch y Arch Linux ARM, y entre fechas/repositorios.

La matriz de capacidades es conservadora, pero no sustituye las comprobaciones del emulador. Un ELF correcto no garantiza las bibliotecas, extensiones graficas, firmware, BIOS o rendimiento necesarios. RPCS3 tiene renderer OpenGL y no debe bloquearse solo por falta de Vulkan, pero exige configurar una ruta que cumpla sus extensiones. EmuBox no reescribe automaticamente su YAML nativo.

El escaner BIOS usa el manifiesto de requisitos y archivos presentes; valida hashes cuando estan definidos. No descarga BIOS, no concede derechos de uso y no puede reconstruir legal o tecnicamente un firmware ausente. El manifiesto tampoco equivale a la lista exhaustiva de alternativas aceptadas por todos los motores.

El lanzamiento directo es la opcion por defecto. Un envoltorio Gamescope requiere solicitud y capacidades compatibles; no debe anidar otro compositor por el simple hecho de haber detectado Vulkan.

## 7. Catalogos y descargas

La importacion registra metadatos y fuentes, no inicia automaticamente descargas. El servicio normaliza distintos formatos y conserva variantes. La seleccion explicita de paquete es la frontera para crear un trabajo de descarga.

Existe cache HTTP persistente con validadores, huellas y pertenencia por manifiesto. Si el servidor permite respuestas condicionales se evita retransmitir contenido sin cambios. Sin ETag/Last-Modified o protocolo delta, el cliente no puede inventar una sincronizacion incremental de red: puede necesitar descargar el documento completo antes de comparar huellas.

El transporte esta separado por proveedores HTTP y BitTorrent (aria2 externo). Una URL torrent obtiene primero el descriptor mediante HTTP; magnet obtiene metadata mediante el motor. Pixeldrain cuenta con un conector de archivo publico; el resto de hostings sin conector se indica explicitamente. No hay un conector universal ni evasion de CAPTCHA o login. HTML no es un archivo de juego. Ver [download-providers.md](download-providers.md).

El tamaño del catalogo no es el numero de juegos instalados ni una garantia de que las fuentes sigan disponibles. HTTP 403/404, TLS, DNS, limites del servidor, firmas caducadas y contenido retirado no se solucionan inventando exito. No se debe desactivar la validacion TLS para ocultar errores.

Autorizacion, licencias y procedencia de juegos/BIOS corresponden a las fuentes y al usuario. Un comentario en un manifiesto no demuestra disponibilidad, seguridad o derechos.

## 8. Telemetria, controles y seguridad

Los mandos nativos se enumeran desde input/sysfs, no mediante un dispositivo Xbox fijo. Las propiedades no medidas quedan ausentes. La disponibilidad de un dispositivo no prueba que su mapeo, vibracion y permisos funcionen en una partida real. La interfaz utiliza tambien la Gamepad API del WebView, una fuente distinta del inventario nativo.

Pantalla se consulta mediante `wlr-randr --json` cuando el compositor implementa el protocolo; audio, mediante consultas estructuradas de `pactl` a la sesion PipeWire/Pulse. Estas herramientas y el acceso al bus/sesion pueden faltar por SSH. En ese caso no se inventan monitor, volumen, frecuencia o latencia. Profundidad de color y HDR siguen sin medirse.

Almacenamiento consulta discos y recorre directorios; inspeccionar no repara permisos. BIOS y logs son consultas reales. Acciones de energia usan systemd/logind sin pedir claves al modelo; pueden ser rechazadas por autorizacion del sistema. No se han ejecutado para probarlas.

Guardar ajustes no significa que todo se aplique en caliente. Varias preferencias se persisten sin una integracion completa con el compositor o PipeWire. Cualquier equipo que amplie estas funciones debe definir por separado guardado, aplicacion, confirmacion y rollback.

La API OTA del runtime no esta implementada: rechaza operaciones. Los scripts externos de actualizacion/recuperacion no son equivalentes a una OTA con firma, verificacion remota, transacciones de paquetes y rollback completo del SO.

El PID y estado del juego se consultan realmente; CPU y memoria del proceso aun no se muestrean y se devuelven como ausentes, no como cero medido.

La ejecucion de comandos de mantenimiento es una superficie sensible. El IPC permite acciones con los permisos del usuario; no es una frontera para ejecutar codigo no confiable. Hace falta una auditoria especifica de permisos Tauri, CSP, contenido remoto y autorizacion antes de presentar la appliance como entorno endurecido o multiusuario.

## 9. Compilacion

El script central valida CPU/target, prepara dependencias, construye SolidJS y genera el ELF nativo Tauri con frontend embebido. Instala el artefacto solo despues de validar su arquitectura. El ejecutable en marcha no cambia hasta reiniciar la sesion.

Las optimizaciones actuales eliminan trabajo repetido, no comprobaciones de seguridad: no hay limpieza Cargo incondicional, Vite no se ejecuta dos veces y la toolchain global no se cambia durante el build.

Las dependencias se reutilizan solo con huella coherente de manifiestos/Node/arquitectura, arbol npm valido y esbuild ejecutable. El frontend se reutiliza con huella de fuentes/configuracion/variables VITE y tambien de salidas, para detectar artefactos borrados o alterados. Cargo vigila el directorio de assets embebidos.

La primera compilacion y los cambios en dependencias nativas seguiran siendo costosos. CPU, RAM, disco, procesos concurrentes y enlazado ponen limites reales. Aumentar hilos sin memoria suficiente puede empeorar el tiempo o provocar OOM. No se habilitan optimizaciones de tamaño/LTO agresivas solo para declarar un build mas rapido.

`EMUBOX_REINSTALL_DEPS=1` fuerza reinstalacion y `EMUBOX_REBUILD_FRONTEND=1` fuerza regenerar assets. Los tiempos deben medirse en builds equivalentes; un build caliente no se compara honestamente con una compilacion limpia de dependencias.

Mediciones locales de esta tarea: primer build con preparacion de caches, 137,6 s; build repetido que aun invalidaba Cargo, 49,7 s; build repetido final, 2,9 s, con etapa Cargo de 0,35 s. Son medidas de esta VM, no un compromiso de tiempo para otros equipos. `src-tauri/capabilities` debe existir aunque no agregue permisos: su ausencia provocaba invalidacion permanente en `tauri-build`. El script evita ademas usar `CARGO_LOG` como variable local de ruta, porque ese nombre pertenece al diagnostico de Cargo.

## 10. Criterios de aceptacion y trabajo pendiente

Las pruebas unitarias y de contratos validan reglas, errores, parsers y aislamiento. No certifican la experiencia fisica. Los recursos temporales de pruebas no se distribuyen ni se insertan en la base real. Se retiraron suites que dependian de respuestas fabricadas; hay que cubrir los recorridos exitosos mediante integracion nativa controlada, no volver a inventar backend para dar una cifra alta de tests.

Quedan pendientes la aceptacion en hardware ARM real, arranque en frio sin SSH, navegacion fisica, audio audible, persistencia tras reinicio y estabilidad grafica en cada equipo. El sondeo SVGA3D positivo no cierra la incidencia visual del hipervisor.

La integracion BitTorrent requiere aria2 instalado y fuentes con peers disponibles; faltan conectores de alojamiento adicionales y preparadores de formatos distintos de ZIP. OTA nativa completa, aplicacion de todos los ajustes al sistema y una distribucion reproducible siguen pendientes. No deben declararse terminados por tener un tipo o un boton.

## 11. Reglas de colaboracion

La definicion de producto se mantiene aparte en [emubox-os-specification.md](emubox-os-specification.md), con estados de soporte y decisiones aun pendientes. No hay aprobacion de ISO, particiones ni bootloader.

No ejecutar comandos de Git ni pruebas de navegador en el flujo actual. No reiniciar TTY1 automaticamente ni ejecutar acciones de energia como pruebas. No revertir cambios del usuario. No tocar ROMs, BIOS, partidas, configuracion de produccion o catalogo real para generar datos de prueba.

Trabajar por responsabilidad, con una hipotesis local y una comprobacion discriminante. Documentar cambios de contrato y medir resultados. Ante una limitacion externa, explicar que capa la controla y que evidencia falta en vez de seguir modificando la UI sin fundamento.