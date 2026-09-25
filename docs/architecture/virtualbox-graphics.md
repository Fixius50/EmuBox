# Aceleración 3D de EmuBox en VirtualBox

## Resultado observado

### Captura real y color, sesion de las 22:24

Se obtuvieron capturas nativas de VirtualBox y de Wayland mediante screencopy,
sin navegador ni entrada sintetica. Ambas muestran la interfaz completa pero
clara; el cambio de luminosidad ya esta presente antes de la presentacion del
anfitrion. En la captura Wayland de 1456x817 se midieron, entre otros, los RGB
(24,42,58) en (10,100) y (36,78,86) en (700,100), superiores al fondo oscuro
definido en los estilos empaquetados. No se capturo el episodio negro durante
el juego ni se demostro la causa del cambio temporal de luminosidad.

Las pruebas antiguas con primarios puros no detectaban diferencias de gamma.
`tests/presentation.test.c` incluye ahora tonos intermedios, mezcla alfa y una
textura ARGB importada con el color oscuro #0b131b. GLES2/SVGA3D y Pixman
coinciden con los valores esperados, con tolerancia de un nivel por canal.
Esto comprueba la escena y la subida de esa textura en wlroots; no reproduce
la composicion interna de WebKit ni los buffers generados por el emulador.

El fondo XMB se pinta ahora directamente sobre la raiz opaca en lugar de una
capa con z-index negativo. Se mantienen los colores por categoria. La onda
decorativa es estatica para `data-gpu-kind=virtual`, conservando renderer
acelerado; los equipos fisicos mantienen su animacion. El objetivo es evitar
la composicion separada del fondo y reducir trabajo continuo de WebKit.
No es una correccion verificada del negro del juego: requiere observar la
ventana real tras cargar el nuevo binario. No se desactiva la aceleracion global
ni se compensa el problema cambiando arbitrariamente gamma o brillo.

En reposo se observaron promedios de CPU altos en WebKit (aproximadamente
85 % de un nucleo), no una medicion de latencia de entrada. La sesion solo
registro un retraso libinput de 46 ms al arrancar, insuficiente para explicar
el segundo de retardo referido. No habia mando fisico identificado: js0 era
VirtualBox mouse integration. Audio UI habilitado en los ajustes por defecto
y stream PipeWire de EmuBox sin mute; sonido audible no verificado.

Grim se extrajo temporalmente de un paquete oficial cuya firma se verifico con
el llavero de Arch; no se instalo software adicional en el SO. No se cambiaron
firmware, driver anfitrion, VSync, gamma ni parametros de latencia del juego.

### Partida y sintomas posteriores, 15 de septiembre

El usuario pudo ejecutar un RPG antiguo con Libretro y reporto sonido/controles
funcionales, pero aproximadamente un segundo de retardo con teclado, cambios de
luminosidad azul al hover y negro o imagen muy tenue tras cerca de un minuto.
Escape abre el mensaje de salida del juego y recupera al menos parte de la imagen.
El mando fisico y la continuidad del sonido durante el negro no estan confirmados.

La sesion de las 21:57 registra biblioteca cargada, rerender y scanout directo
desactivado, sin errores GPU nuevos. `consoleblank=0`, no se encontro daemon de
inactividad, y RetroArch tiene menu_screensaver_timeout=0, HDR desactivado, shader
de video desactivado e insercion de frames negros=0. No hay una regla CSS de hover
que ajuste la luminosidad de toda la pantalla. Esas comprobaciones NO descartan
problemas de presentacion, foco, core o hipervisor. No se cambiaron gamma, VSync,
latencia, shaders ni politicas de energia para aparentar una solucion.

Las reglas de teclado/mando de la UI se revisan independientemente de la entrada
nativa del emulador. No hay una espera programada de un segundo en la primera
accion de la UI; las pruebas unitarias no miden latencia extremo a extremo.

La prueba de presentacion admite como argumento una duracion de 1 a 120 segundos:
anadir `70` al comando de `/tmp/emubox-presentation-test` mantiene la escena mas
alla del minuto, verifica todos los pixels y registra el mayor intervalo entre
frames. Sigue siendo una salida headless de 96x64, no una prueba del juego, WebKit,
KMS fisico ni del mando. Su resultado no permite declarar resuelto el negro.

Resultado de la ejecucion prolongada: 3008 cuadros y 18481152 pixels correctos
en 70033 ms; mayor separacion entre frames de 62,60 ms. La lectura de pixels
forma parte del coste del test, por lo que no es un benchmark del juego ni una
medicion de latencia de teclado/mando. No se reprodujo el negro en esta escena
aislada. DRM informaba DPMS On en la consulta realizada; no se midio durante
el episodio del juego. Brillo, negro y latencia real siguen pendientes.

### Imagen parcial tras cargar la biblioteca

En la sesion de las 21:17 del 15 de septiembre, el adaptador PRIME y
SVGA_NO_LOGGING ya estaban activos: cero errores del compositor y biblioteca
cargada por IPC. Aun asi, el usuario observo pantalla negra parcial que se
actualizaba al mover el raton. Por tanto, eliminar los errores de handles no
resuelve ni certifica la presentacion visual.

El recorrido efectivo tiene responsabilidades distintas:

```text
SQLite / IPC -> SolidJS / WebKit -> buffer de la aplicacion
   -> importacion en Cage -> composicion GLES2 / regiones de actualizacion
   -> buffer de salida -> DRM legacy / vmwgfx -> VMSVGA / GPU del anfitrion
```

Se han comprobado carga de datos, capacidades OpenGL y liberacion de referencias
PRIME. Queda por confirmar en la pantalla fisica que los cuadros se presentan
completos y siguen actualizandose sin depender del movimiento del raton.
El problema observado es compatible con regiones de actualizacion incompletas;
esa es una hipotesis, no una causa demostrada en todas las capas.

`configure_emubox_presentation` configura solo Cage con vmwgfx:
`WLR_SCENE_DEBUG_DAMAGE=rerender` fuerza redibujar todo el cuadro en cada
actualizacion, y `WLR_SCENE_DISABLE_DIRECT_SCANOUT=1` mantiene la composicion
en Cage en lugar de presentar directamente el buffer del cliente. No fuerza
renderizado por CPU, no activa bucles JavaScript ni genera eventos de raton.
Puede aumentar el trabajo de GPU y ancho de banda. Se respetan variables ya
definidas; `WLR_SCENE_DEBUG_DAMAGE=none` y
`WLR_SCENE_DISABLE_DIRECT_SCANOUT=0` recuperan el comportamiento anterior.
Otras GPU y la seleccion de Gamescope no reciben esta politica.

El log `presentation:` registra renderer, modo de regiones, scanout directo y
la compatibilidad DMA-BUF de WebKit, junto con la seleccion del adaptador.
El diagnostico de appliance reconoce tambien Cage ejecutado mediante el cargador
glibc; encontrar un proceso y un socket Wayland no convierte `graphics-functional`
en PASS ni certifica que el contenido sea visible.

Prueba de presentacion nativa sobre wlroots 0.20 (requiere sus cabeceras):

```bash
cc -std=c11 -Wall -Wextra -Werror tests/presentation.test.c $(pkg-config --cflags --libs wlroots-0.20 pixman-1 wayland-server) -o /tmp/emubox-presentation-test
SVGA_NO_LOGGING=1 WLR_RENDERER=gles2 WLR_RENDER_DRM_DEVICE=/dev/dri/renderD128 /lib64/ld-linux-x86-64.so.2 --preload "$PWD/bin/libemubox-vmwgfx.so" /tmp/emubox-presentation-test
```

La prueba usa una salida headless de 96x64, ocho cuadros y un rectangulo de
8x8 que cambia de color sin raton. Comprueba los pixels de todos los cuadros y
la region presentada, no solo que EGL inicialice. No inicia navegador, juegos,
otra instancia de EmuBox ni modifica la pantalla real. Este comando es para el
equipo x86_64 observado; el nodo y cargador dependen de la arquitectura.
No valida el contenido producido por WebKit ni el scanout KMS de la pantalla.
Estos limites deben mantenerse explicitos incluso cuando la prueba pase.

### Correccion PRIME de Cage y trazas de VMware, 15 de septiembre

La prueba nativa con GBM sobre un descriptor privado de render reprodujo
`drmCloseBufferHandle: EINVAL` fuera de Cage y EmuBox. PRIME devolvia un handle
de superficie TTM de vmwgfx, no un handle de buffer GEM. El cierre generico
`DRM_IOCTL_GEM_CLOSE` y `DRM_VMW_HANDLE_CLOSE` fallaban; la operacion oficial
`DRM_VMW_UNREF_SURFACE` libera esa referencia. Se verificaron 64 ciclos de
liberacion, rechazo del doble cierre y reimportacion manteniendo al exportador.

`scripts/vmwgfx-compat.c` implementa un adaptador local: intenta primero GEM_CLOSE;
solo tras EINVAL y un driver exactamente `vmwgfx` usa UNREF_SURFACE. No simula
exito ni descarta errores. El build genera `bin/libemubox-vmwgfx.so`, sin sustituir
paquetes del sistema. Cage la carga con `ld.so --preload`, no con LD_PRELOAD:
el adaptador no se hereda al ejecutar EmuBox, WebKit ni emuladores. Afecta solo
al proceso Cage, y queda desactivado con `EMUBOX_VMWGFX_COMPAT=0`. Sin biblioteca
o cargador compatible se avisa y se usa Cage normal. Otras GPU conservan su ruta.
La prueba C del adaptador compilado paso 128 ciclos con pixels conservados,
reimportacion y doble cierre rechazado. Los fallos iniciales del sondeo C se
debieron a reutilizar el descriptor de mapeo despues de unmap; se corrigio
inicializandolo en cada mapeo, manteniendo las comprobaciones de integridad.
Cage con salida headless y EGL/Wayland tambien mantiene SVGA3D. Se verifico
que `ld.so --preload` no hereda la biblioteca al siguiente exec. El adaptador
queda habilitado solo para Cage/vmwgfx; la comprobacion de salida fisica y
estabilidad prolongada sigue pendiente de reiniciar TTY1 y observar la sesion.
No es una reparacion del kernel ni una certificacion general de estabilidad.

El sondeo strace de EGL tambien aislo `Failed to open channel`: Mesa enviaba
trazas al canal de VMware, ausente en VirtualBox. Con la opcion oficial
`SVGA_NO_LOGGING=1`, las llamadas DRM_VMW_MSG pasaron de dos a cero manteniendo
SVGA3D en OpenGL core, compatibilidad y ES. El lanzador configura esa opcion
antes de los sondeos solo cuando systemd-detect-virt identifica `oracle` y el
usuario no la ha definido. No cambia loglevel del kernel ni filtra mensajes
de Cage: evita la operacion no soportada de envio de trazas al anfitrion VMware.

Prueba reproducible del adaptador, tras compilarlo:

```bash
cc -std=c11 -Wall -Wextra -Werror tests/vmwgfx.test.c $(pkg-config --cflags --libs gbm libdrm) -ldl -o /tmp/emubox-vmwgfx-test
SVGA_NO_LOGGING=1 /tmp/emubox-vmwgfx-test /opt/emubox/bin/libemubox-vmwgfx.so /dev/dri/renderD128
```

El nodo de render debe corresponder a vmwgfx; no se usa DRM master ni se cambia
la salida fisica. Sin el ultimo argumento solo se prueban errores de descriptores.
La prueba de hardware comprueba 128 ciclos, doble cierre y pixels conservados.
El aviso inicial del kernel sobre hipervisor no soportado puede permanecer;
TDX, microcodigo y reloj del anfitrion son independientes de estos dos fallos.
No se desactivan protecciones para quitarlos. La sesion existente requiere un
reinicio de TTY1 elegido por el usuario para cargar los cambios del lanzador.

### Arranque del 15 de septiembre de 2026

VirtualBox anfitrion y Guest Additions informan 7.2.16r174877. La sesion usa Cage,
Mesa 26.2.2 y SVGA3D/OpenGL, pero registra `drmCloseBufferHandle failed` y el kernel
`vmwgfx: Failed to open channel` y `unsupported hypervisor`. Las versiones coinciden;
no se ha demostrado que reinstalar Guest Additions solucione el problema. Se mantienen
los ajustes existentes de DRM legacy, cursor compuesto y compatibilidad DMA-BUF,
sin forzar Vulkan, desactivar vmwgfx ni ocultar errores del kernel.

La interfaz de paravirtualizacion KVM expuesta por VirtualBox explica el mensaje
`Hypervisor detected: KVM`; no prueba que la VM este ejecutandose en otro producto.
TDX es una capacidad de virtualizacion confidencial Intel no requerida por EmuBox.
Su ausencia en esta VM con CPU AMD no impide iniciar Cage ni la UI.

SRSO informa `Vulnerable: Safe RET, no microcode`, y TSA informa ausencia de
microcodigo. Deben contrastarse BIOS/UEFI, actualizaciones de seguridad del anfitrion
y capacidades expuestas por VirtualBox. Instalar `amd-ucode` en el invitado no
certifica la mitigacion del anfitrion. No se desactivan mitigaciones ni se fuerzan
flags de CPU para hacer desaparecer avisos. La actualizacion de firmware y los
cambios de VirtualBox requieren una intervencion separada en el anfitrion, con
la VM apagada ordenadamente cuando corresponda. No se han aplicado desde Linux.

Referencias: [TDX del kernel](https://docs.kernel.org/arch/x86/tdx.html) y
[estado de mitigacion SRSO](https://docs.kernel.org/admin-guide/hw-vuln/srso.html).

### Revision del anfitrion y GameMode

Consulta de solo lectura del anfitrion Windows: GPU AMD Radeon RX 6600,
controlador 32.0.21045.1000, estado informado OK. VirtualBox tiene VMSVGA,
aceleracion 3D activada, 256 MiB de VRAM, un monitor y cuatro vCPU; la interfaz
efectiva de paravirtualizacion es KVM. No faltaba activar 3D ni asignar VRAM.
El registro del anfitrion contiene entradas NEM y un error de GuestPropSvc
`VERR_HGCM_SERVICE_NOT_FOUND`; no demuestra por si solo la causa de los errores
DRM del invitado. No se cambiaron Hyper-V/VBS, firmware, mitigaciones, controlador
de Windows ni opciones de la VM. Los paquetes GameMode/libdrm/wlroots verificados
no tienen archivos alterados, y las versiones de los repositorios locales
coinciden con las instaladas. Eso no certifica ausencia de bugs del driver.

El cliente que activo GameMode en el arranque era RetroArch (PID 816).
La VM no expone gobernadores cpufreq. GameMode 1.8.2 devuelve un estado vacio
cuando no los encuentra y aun intenta `cpugovctl` mediante pkexec; su configuracion
no ofrece un interruptor para omitir ese cambio. Conceder permisos no crearia
un gobernador fisico en el invitado.

El perfil RetroArch desactiva `gamemode_enable` cuando HardwareInfo identifica
una VM, tanto en la configuracion gestionada como en la ubicacion XDG nativa
que RetroArch usa por defecto. En hardware fisico no modifica esta preferencia.
Los archivos de configuracion especificados explicitamente por el usuario con
`--config`/`--appendconfig` conservan su responsabilidad y pueden cambiar la opcion.
RPCS3 ya tenia `Enable GameMode: false` en este equipo. No se envuelven los
sondeos ni juegos automaticamente con gamemoderun.

`installer/config/gamemode.ini` es el perfil conservador para nuevas cuentas
appliance sin configuracion propia: mantiene split_lock_mitigate, no solicita
renice/SCHED_ISO, evita pinning/parking automaticos y no pide un salvapantallas
D-Bus que Cage no ofrece. El instalador conserva archivos existentes; los valores
de GameMode pueden ser sobreescritos por configuraciones de mayor prioridad.
El perfil no convierte GameMode en un motor utilizable sin cpufreq.

En esta sesion se instalo el perfil de usuario, se reinicio solo gamemoded sin
clientes activos y se desactivo GameMode en el archivo nativo de RetroArch.
`split_lock_mitigate` permanecio en 1. Una prueba nativa con dos frames, video
null, sin juego y con configuracion temporal termino sin errores GameMode; no
valida sonido, juegos ni estabilidad de SVGA3D. El primer sondeo fallido habia
intentado inicializar Qt sin pantalla; se corrigio su configuracion, no la UI.
La sesion Cage/EmuBox no se reinicio. Los errores de vmwgfx siguen pendientes;
no se ocultan ni se afirma que este perfil los repare.

Referencia de opciones: [GameMode 1.8.2](https://github.com/FeralInteractive/gamemode/blob/1.8.2/example/gamemode.ini).

## Politica de seleccion

EmuBox detecta CPU y GPU por separado. La prioridad automatica es usar la GPU
disponible, incluida aceleracion 3D virtual como SVGA3D/virgl; no se considera
software simplemente por ejecutarse en una VM. Primero elige OpenGL acelerado
para Cage/WebKitGTK, o Vulkan acelerado si falta esa ruta. Cage es automatico
con OpenGL/DRM; Gamescope solo por preferencia compatible o cuando Cage no
soporta el backend disponible. Si ambos sondeos concluyen mostrando solo software,
el lanzador puede usar CPU. Sondeos incompletos mantienen diagnostico indeterminado
y se intenta Cage automatico, sin forzar Pixman ni Mesa software.

El nombre de la CPU y sus nucleos se registran independientemente. Vulkan y
OpenGL son APIs, no modelos de GPU. Si faltan herramientas/drivers de sondeo,
no se puede certificar aceleracion ni ausencia de ella. El fallback software ante
incertidumbre requiere una eleccion explicita y se registra separado del diagnostico.
Un valor legado `software` ya no desactiva aceleracion utilizable detectada.
La UI nativa recibe las capacidades Rust; la deteccion WebGL se usa en navegador.

La VM vuelve a modo `auto` el 6 de septiembre de 2026 por peticion del usuario.
Esto restablece la prioridad GPU, no demuestra que el fallo grafico previo del
driver virtual este resuelto. El cambio local necesita reiniciar TTY1.

El 5 de septiembre de 2026, la VM de desarrollo expone:

- CPU: AMD Ryzen 5 5600G, x86_64, cuatro vCPU.
- GPU PCI: VMware SVGA II (`15ad:0405`), driver `vmwgfx`.
- EGL/OpenGL: `SVGA3D; build: RELEASE; LLVM;`, OpenGL 4.1 y OpenGL ES 3.0.
- Vulkan: sin dispositivos válidos.
- Selección EmuBox: Cage, con OpenGL acelerado disponible.

`LLVM` en el nombre SVGA3D no significa `llvmpipe`. El detector distingue
explícitamente los renderers software. No debe inventar una GPU Radeon a partir
del texto del modelo de CPU ni considerar Vulkan un requisito universal.

## GPU virtual frente a GPU física

La aceleración 3D de VirtualBox permite que la GPU virtual use los servicios
gráficos del anfitrión. El invitado sigue viendo VMSVGA, no el modelo físico
AMD/NVIDIA/Intel. Instalar el driver de la GPU del anfitrión dentro de la VM
no cambia ese dispositivo ni hace aparecer soporte Vulkan.

Para acceso directo al dispositivo físico hace falta una solución de passthrough
compatible con el anfitrión, GPU e IOMMU, por ejemplo KVM/VFIO sobre Linux,
o ejecutar EmuBox en hardware físico. No es lo mismo que activar la casilla 3D
de VirtualBox. El código EmuBox no puede concederse ese acceso desde el invitado.

## Comprobar el anfitrión

Solo si hace falta cambiar la configuración, apagar la VM de forma ordenada
(no dejarla en estado guardado) y abrir Configuración > Pantalla en VirtualBox:

1. Controlador gráfico VMSVGA para el invitado Linux.
2. Aceleración 3D habilitada.
3. Memoria de vídeo suficiente para la resolución y monitores; 128 MB es un
   punto de partida habitual, sujeto a lo que permita la versión instalada.
4. Driver gráfico del anfitrión actualizado y Guest Additions compatibles con
   su versión de VirtualBox. En Arch, usar paquetes de la distribución sin
   mezclar arbitrariamente instalaciones desde ISO.

En anfitriones Windows con varias GPU, la preferencia gráfica para
`VirtualBoxVM.exe` se configura en Windows; no desde Arch invitado. Para saber
qué GPU física está usando VirtualBox hay que comprobar el proceso en el anfitrión.

La VM observada ya ofrece SVGA3D: no se necesita cambiar drivers ni forzar Vulkan
para reconocer su aceleración OpenGL. Las opciones del anfitrión no se han tocado.

## Verificación en el invitado

### Cursor invisible en Cage

Si el tema nativo existe pero el puntero sigue invisible, el plano de cursor DRM
de la GPU virtual puede ser el causante. El lanzador usa
`WLR_NO_HARDWARE_CURSORS=1` solo con Cage y el driver `vmwgfx`, salvo que la
variable ya este definida. Esto compone el cursor dentro de la imagen y no
desactiva la aceleracion 3D de OpenGL. Es necesario reiniciar TTY1 para aplicarlo.
La visibilidad debe confirmarse en la pantalla de la VM; las pruebas automaticas
solo verifican la seleccion del ajuste y el respeto a preferencias explicitas.

Si la imagen se corrompe y Cage registra `Atomic commit failed: Device or resource
busy`, el lanzador configura tambien `WLR_DRM_NO_ATOMIC=1` para `vmwgfx`.
wlroots usa entonces la interfaz DRM legacy, manteniendo el renderizado OpenGL.
No se cambia este ajuste en otras GPU ni se sobrescribe una variable ya definida.
Tras reiniciar TTY1, el log debe mostrar `forcing legacy DRM interface`.
Esto evita la ruta que estaba fallando, pero no certifica que todos los fallos
visuales de VirtualBox hayan desaparecido: comprobar la imagen al interactuar.

El proveedor de mando combina D-pad y stick en una sola direccion por lectura,
con repeticion tras 350 ms y cada 120 ms al mantenerla. El scroll de la biblioteca
no selecciona la tarjeta bajo un raton inmovil; se exige movimiento real del raton.

### Si la corrupcion persiste

El 16 de septiembre el usuario concreto un caso recurrente: navegar parece normal,
pero hacer clic cambia el brillo y, pasado un tiempo, las zonas bajo el movimiento
del raton se repintan oscuras. La correccion anterior de fondo opaco/onda no resuelve
ese caso. No se considera cerrada la incidencia por tener logs o tests limpios.

La sesion afectada tenia Cage/GLES2, `damage=rerender`, direct scanout desactivado,
cursor software y `WEBKIT_DISABLE_DMABUF_RENDERER=1`; los procesos EmuBox y WebKit
habian heredado esas variables. Repintar toda la salida de Cage no garantiza que
el cuadro recibido desde WebKit sea correcto. No se encontro un filtro CSS
`brightness()` que explicase el cambio global al hacer clic.

El usuario volvio a observar un aumento repentino del brillo y una pagina local
de Jackett mal presentada tras abrir otro WebView. Ya habia persistido el defecto
con `WEBKIT_DISABLE_DMABUF_RENDERER=1`, por lo que repetir solo ese cambio no
constituye una solucion. Se probo tambien desactivar la composicion WebKit con
`WEBKIT_DISABLE_COMPOSITING_MODE=1` solo para vmwgfx; el usuario confirmo que
el problema persistia, por lo que se retiro para no anadir coste de CPU.
La politica vuelve a `WEBKIT_DISABLE_DMABUF_RENDERER=0` y
`WEBKIT_DMABUF_RENDERER_FORCE_SHM=1`, conservando Cage/GLES2 y los overrides
explicitos del transporte. La RAM libre y las muestras de proceso no muestran
presion de memoria ni crecimiento continuo de WebKit. Sigue sin determinarse
la causa del iluminado; no marcarlo como resuelto.

Se corrigio tambien la politica de filtros: una GPU virtual acelerada ya no
recomienda blur, y `data-blur-mode="software"` ahora controla las variables CSS.
Antes el atributo no tenia consumidor y los filtros seguian activos salvo en
pipeline CPU/indeterminado. Este ajuste no cambia la deteccion de aceleracion.

Referencia de implementacion: WebKit
[AcceleratedBackingStore](https://github.com/WebKit/WebKit/blob/main/Source/WebKit/UIProcess/gtk/AcceleratedBackingStore.cpp)
y [AcceleratedSurface](https://github.com/WebKit/WebKit/blob/main/Source/WebKit/WebProcess/WebPage/CoordinatedGraphics/AcceleratedSurface.cpp).
Se comprobaron las opciones disponibles en la biblioteca instalada 2.52.6; la
consulta de codigo upstream no equivale a una prueba visual de esa compilacion.

Pendiente tras reiniciar TTY1: repetir clics y navegacion, esperar al menos el
tiempo que antes disparaba el oscurecimiento y mover el raton por zonas quietas.
La validacion automatica cubre seleccion de transporte, CSS, sintaxis y overrides;
no reproduce el WebView ni certifica la desaparicion del defecto. No se han hecho
pruebas de navegador ni se ha reiniciado la sesion automaticamente.

DRM legacy y cursor software no garantizan estabilidad de SVGA3D. En la VM se
observo recurrencia junto a errores de kernel `vmwgfx: Failed to open channel`.
La politica actual elimina el override CPU cuando hay aceleracion: los valores
legados de `/etc/emubox/graphics-mode` o `EMUBOX_RENDER_MODE` no anulan una GPU
detectada. El arranque limpia variables heredadas que forzaban Mesa/software,
Pixman o desactivaban composicion WebKit antes de sondear. Software confirmado o
fallback explicito con diagnostico indeterminado permite Cage/Pixman,
`LIBGL_ALWAYS_SOFTWARE=1` y `WEBKIT_DISABLE_COMPOSITING_MODE=1`.
Ni `WEBKIT_DISABLE_DMABUF_RENDERER=1` ni la desactivacion de composicion WebKit
cerraron el fallo. Se mantiene el transporte SHM previo para `vmwgfx`.
Revisar VirtualBox, Guest Additions y driver del
anfitrion si persiste la corrupcion; detectar capacidad no certifica estabilidad.

```bash
systemd-detect-virt
lspci -nnk
eglinfo -B
vulkaninfo --summary
```

Las herramientas `eglinfo` y `glxinfo` pertenecen a `mesa-utils` en Arch;
`vulkaninfo`, a `vulkan-tools`. `mesa-utils` forma parte del sondeo requerido;
Gamescope y las herramientas Vulkan son opcionales.
Por SSH, que fallen las plataformas EGL X11/Wayland no invalida un sondeo GBM
o surfaceless válido. No asumir que todas las aplicaciones usan el renderer
detectado: confirmar también el log de la sesión y del emulador.

Gamescope tiene requisitos Vulkan propios, no globales de EmuBox. Cage usa
OpenGL acelerado aunque falte Vulkan y no se sustituye solo porque este exista; los
perfiles que implementan selección de API usan OpenGL cuando no hay Vulkan.
Un emulador puede requerir versiones/extensiones no expuestas por VirtualBox;
el rendimiento y la compatibilidad no equivalen a una GPU física dedicada.
RPCS3 dispone de renderer OpenGL y no se bloquea globalmente por falta de Vulkan.
Su configuracion nativa debe seleccionar una API compatible; EmuBox no certifica
sus extensiones ni modifica automaticamente su YAML de configuracion.

Referencias oficiales: [pantalla y VMSVGA](https://www.virtualbox.org/manual/ch03.html#settings-display)
y [aceleración 3D](https://www.virtualbox.org/manual/ch04.html#guestadd-3d).