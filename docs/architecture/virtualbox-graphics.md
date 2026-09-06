# Aceleración 3D de EmuBox en VirtualBox

## Resultado observado

## Politica de seleccion

EmuBox detecta CPU y GPU por separado. La prioridad automatica es usar la GPU
disponible, incluida aceleracion 3D virtual como SVGA3D/virgl; no se considera
software simplemente por ejecutarse en una VM. Si detecta Vulkan hardware,
DRM y Gamescope disponible, usa Gamescope. Si detecta OpenGL acelerado, Cage
puede usar esa GPU aunque no haya Vulkan. Si no se detecta aceleracion Vulkan
ni OpenGL, el lanzador recurre a CPU con Cage/Pixman y Mesa/WebKit software.

El nombre de la CPU y sus nucleos se registran independientemente. Vulkan y
OpenGL son APIs, no modelos de GPU. Si faltan herramientas/drivers de sondeo,
no se puede certificar aceleracion y puede seleccionarse el fallback conservador.
El modo `software` sigue disponible solo como override explicito de diagnostico.
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

### Prueba reversible si la corrupcion persiste

DRM legacy y cursor software no garantizan estabilidad de SVGA3D. En la VM se
observo recurrencia junto a errores de kernel `vmwgfx: Failed to open channel`.
Para discriminar la ruta 3D, el lanzador admite `/etc/emubox/graphics-mode` con
un unico valor: `auto` (por defecto) o `software`. La variable de entorno
`EMUBOX_RENDER_MODE` tiene prioridad sobre ese archivo.

`software` selecciona Cage/Pixman, `LIBGL_ALWAYS_SOFTWARE=1` y
`WEBKIT_DISABLE_COMPOSITING_MODE=1`. Es una prueba temporal con posible coste de
rendimiento, no una mejora de aceleracion ni una solucion confirmada del driver.
El cambio no requiere build, pero si reiniciar TTY1. El log debe mostrar
`renderMode=software` y `Creating pixman renderer`.

Para volver a la deteccion automatica, cambiar el contenido del archivo a `auto`
y reiniciar TTY1. No se aplica este modo por defecto en otras maquinas. Si la
imagen se estabiliza solo en software, revisar la version de VirtualBox y el
driver del anfitrion antes de volver a activar la ruta 3D.

```bash
systemd-detect-virt
lspci -nnk
eglinfo -B
vulkaninfo --summary
```

Las herramientas `eglinfo` y `glxinfo` pertenecen a `mesa-utils` en Arch;
`vulkaninfo`, a `vulkan-tools`. Son diagnósticos opcionales del instalador.
Por SSH, que fallen las plataformas EGL X11/Wayland no invalida un sondeo GBM
o surfaceless válido. No asumir que todas las aplicaciones usan el renderer
detectado: confirmar también el log de la sesión y del emulador.

Gamescope requiere Vulkan. Si no está disponible, EmuBox utiliza Cage; los
perfiles que implementan selección de API usan OpenGL cuando no hay Vulkan.
Un emulador puede requerir versiones/extensiones no expuestas por VirtualBox;
el rendimiento y la compatibilidad no equivalen a una GPU física dedicada.

Referencias oficiales: [pantalla y VMSVGA](https://www.virtualbox.org/manual/ch03.html#settings-display)
y [aceleración 3D](https://www.virtualbox.org/manual/ch04.html#guestadd-3d).