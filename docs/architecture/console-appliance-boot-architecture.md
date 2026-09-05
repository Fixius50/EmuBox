# Arquitectura de arranque de EmuBox

## Alcance y estado

La prioridad actual es Runtime + Appliance en ambas CPU. La distribucion/imagen
EmuBox OS es posterior y no bloquea esta validacion. El instalador es la ruta
manual/de desarrollo, no el producto de distribucion final.
Ver [criterios de aceptacion](appliance-validation.md).

El instalador configura un sistema existente: Arch Linux en x86_64 o Arch Linux
ARM en aarch64. No instala ARM32, particiona discos ni crea una ISO. La CPU no
determina el compositor ni garantiza compatibilidad con todos los emuladores.

Se ha compilado y validado el ELF x86_64. El workflow incluye un runner ARM64
nativo, pero su ejecución y el arranque sobre hardware ARM real siguen pendientes.
No se considera terminada la aceptación ARM64 hasta verificar ambos.

## Sesión local y SSH

```text
systemd / getty@tty1
        |
autologin usuario emubox (PAM, seat0)
        |
/home/emubox/.bash_profile (solo tty1)
        |
/usr/local/bin/emubox-session
        |
/opt/emubox/scripts/run.sh
        |
        +-- Vulkan hardware + DRM + Gamescope --> Gamescope --> EmuBox
        |
        +-- resto de capacidades --------------> Cage -------> EmuBox
```

Cage y Gamescope son alternativas en este flujo, no capas anidadas. Cage puede
usar aceleración OpenGL: la falta de Vulkan no prueba renderizado por CPU.
Gamescope puede usar XWayland para clientes que lo requieran; no se promete una
cadena sin X11 para todos los emuladores.

SSH sirve para mantenimiento. Conectar o cerrar SSH no debe iniciar ni terminar
la sesión física. El servicio auxiliar `emubox.service` queda deshabilitado para
evitar un segundo arranque junto a getty. TTY2/TTY3 permiten recuperación local.

## Detección y permisos

`installer/lib/architecture.sh` normaliza CPU y valida ELF. El runtime Rust usa
`std::env::consts::ARCH` y comunica `uname -m` por separado como `kernelArchitecture`.
`installer/lib/graphics.sh` sondea DRM/sysfs, dispositivos DRI y Vulkan, con PCI
como apoyo opcional. Incluye drivers virtuales, AMD, Intel, NVIDIA, Broadcom,
Mali, Adreno y Apple; los dispositivos desconocidos no se clasifican por CPU.

Gamescope se selecciona si se detectan Vulkan hardware, DRM y su ejecutable.
Esto es una recomendación por capacidades, no una prueba de que una sesión KMS
pueda iniciarse: permisos, drivers y pantalla deben verificarse en el equipo.
Si falta Cage en la ruta alternativa, el lanzador falla con un mensaje explícito.
No se exportan optimizaciones RADV globales.

La sesión utiliza el usuario `emubox`, su runtime XDG y D-Bus. Los permisos DRM,
video/input/seat y los datos de la appliance se preparan durante la instalación.
Código, Git y compilación pertenecen a `emubox:emubox`; solo la configuración
del sistema requiere root. No cambiar recursivamente los permisos de `/`.

Los instaladores recargan systemd tras generar unidades del sistema o de usuario.
La regla tmpfiles recrea `/run/emubox` como `emubox:emubox` en cada arranque.
La base habilita PipeWire/WirePlumber de usuario y no requiere emuladores;
los motores se instalan aparte mediante `installer/setup/emulator-packages.sh`.
El log de sesión se conserva en `/var/log/emubox/session.log`.
`emubox-drm-sync` escucha eventos DRM para ajustar salidas compatibles; su
funcionamiento requiere comprobación en cada compositor y pantalla.

## Build y actualización

`scripts/build.sh` compila como usuario sin privilegios, selecciona `BUILD_ARCH`
y `TARGET` nativos, ejecuta frontend y Tauri `--no-bundle` (con alternativa Cargo),
y valida el ELF antes de instalar `bin/emubox`. Genera además el artefacto local
`bin/emubox-linux-x86_64` o `bin/emubox-linux-aarch64`.

`scripts/update-emubox.sh` exige un árbol limpio, obtiene `origin/main` y aplica
`pull --ff-only`. Conserva el ejecutable anterior antes del pull, lo restaura
durante el build y ante errores, y valida el ELF final antes de reiniciar TTY1.
No hace auto-stash ni reset de código. Los builds y actualizaciones usan logs
temporales si no pueden escribir en el directorio de logs del sistema.

Los emuladores y cores se validan de nuevo antes del lanzamiento. Las ROMs,
partidas, BIOS, IDs y manifiestos no dependen de la CPU; los binarios gestionados
sí se separan por arquitectura. RPCS3 sigue visible aun si no puede ejecutarse.

## Aplicar y verificar

Tras revisar y compilar los cambios, un administrador puede regenerar el arranque:

```bash
sudo bash scripts/setup-autostart.sh
sudo systemctl restart getty@tty1
```

El reinicio interrumpe la sesión gráfica y cualquier juego activo. Estos comandos
no se han ejecutado durante la integración ARM64. Modificar los scripts del
repositorio no cambia automáticamente el autologin previamente instalado.

Verificar en cada CPU admitida:

1. `file bin/emubox` y arquitectura/target nativos.
2. Autologin `emubox`, permisos DRM, runtime XDG y bus de sesión.
3. Diagnóstico real de CPU, RAM, GPU, Vulkan, DRM, Gamescope y modelo del equipo.
4. Gamescope con Vulkan hardware y Cage cuando falta Vulkan.
5. UI visible y manejable con mando, sin dependencia de SSH.
6. UI, IPC, persistencia SQLite, mando y audio sin emuladores instalados.
7. Arranque en frío y reconexión de pantalla cuando sea compatible.

Las pruebas shell y mocks no sustituyen estas comprobaciones físicas. La CI sobre
Ubuntu ARM valida compilación nativa, no instalación de paquetes en Arch Linux ARM.
El lanzamiento de motores/cores nativos es una fase posterior independiente.