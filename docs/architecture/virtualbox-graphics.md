# Aceleración 3D de EmuBox en VirtualBox

## Resultado observado

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

El proveedor de mando combina D-pad y stick en una sola direccion por lectura,
con repeticion tras 350 ms y cada 120 ms al mantenerla. El scroll de la biblioteca
no selecciona la tarjeta bajo un raton inmovil; se exige movimiento real del raton.

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