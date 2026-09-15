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
        +-- sondear GPU fisica/virtual -> OpenGL / Vulkan / software
        |
        +-- seleccionar compositor compatible -> EmuBox
```

Cage y Gamescope son alternativas en este flujo, no capas anidadas. Cage puede
usar aceleración OpenGL: la falta de Vulkan no prueba renderizado por CPU.
Gamescope puede usar XWayland para clientes que lo requieran; no se promete una
cadena sin X11 para todos los emuladores.

SSH sirve para mantenimiento. Conectar o cerrar SSH no debe iniciar ni terminar
la sesión física. El servicio auxiliar `emubox.service` queda deshabilitado para
evitar un segundo arranque junto a getty. TTY2/TTY3 permiten recuperación local.

La proteccion contra bucles se configura en el propio `getty@tty1`: tres inicios
en 60 segundos y cinco segundos entre reinicios. Los limites del servicio auxiliar
deshabilitado no protegen la sesion fisica. Alcanzar el limite detiene los reintentos;
desde SSH o TTY2, tras reparar la causa, se puede ejecutar
`sudo systemctl reset-failed getty@tty1` y reiniciarlo deliberadamente.

## Detección y permisos

`installer/lib/architecture.sh` normaliza CPU y valida ELF. El runtime Rust usa
`std::env::consts::ARCH` y comunica `uname -m` por separado como `kernelArchitecture`.
`installer/lib/graphics.sh` consume `bin/emubox --graphics-session`: el mismo
servicio Rust que alimenta IPC sondea DRM/sysfs, EGL/OpenGL y Vulkan. Incluye drivers virtuales, AMD, Intel, NVIDIA, Broadcom,
Mali, Adreno y Apple; los dispositivos desconocidos no se clasifican por CPU.

OpenGL acelerado tiene preferencia para Cage/WebKitGTK; sin OpenGL acelerado se
considera Vulkan acelerado. Solo evidencia software suficiente permite diagnosticar
software; fallos y cobertura incompleta conservan indeterminado. Cage es automatico
con OpenGL/DRM o como intento sin renderer forzado ante incertidumbre. Un fallback
software explicito queda registrado sin cambiar ese diagnostico. Gamescope exige Vulkan acelerado, DRM y
ejecutable: se usa por preferencia compatible (`EMUBOX_COMPOSITOR_PREFERENCE=gamescope`)
o como alternativa si Cage no soporta el backend disponible. Vulkan por si solo
no fuerza Gamescope. Esto es una recomendación por capacidades, no una prueba de que una sesión KMS
pueda iniciarse: permisos, drivers y pantalla deben verificarse en el equipo.
Si falta Cage en la ruta alternativa, el lanzador falla con un mensaje explícito.
No se exportan optimizaciones RADV globales.

El inventario y las observaciones no se mezclan entre GPU. Correlacion no resuelta
no bloquea OpenGL observado; el dispositivo utilizado no se afirma sin evidencia
del proceso. `bin/emubox --graphics-info` muestra el contrato completo en JSON.

La sesión utiliza el usuario `emubox`, su runtime XDG y D-Bus. Los permisos DRM,
video/input/seat y los datos de la appliance se preparan durante la instalación.
Código, Git y compilación pertenecen a `emubox:emubox`; solo la configuración
del sistema requiere root. No cambiar recursivamente los permisos de `/`.

Los instaladores recargan systemd tras generar unidades del sistema o de usuario.
La regla tmpfiles recrea `/run/emubox` como `emubox:emubox` en cada arranque.
La base habilita PipeWire/WirePlumber de usuario y no requiere emuladores;
los motores se instalan aparte mediante `installer/setup/emulator-packages.sh`.
El log de sesión se conserva en `/var/log/emubox/session.log`.
Si no existe configuracion de GameMode del usuario, el setup instala el perfil
conservador de `installer/config/gamemode.ini`: no cambia mitigaciones del kernel
ni solicita afinidad automatica de CPU o salvapantallas de escritorio. Las
preferencias existentes se conservan. RetroArch desactiva GameMode en VM mediante
su perfil nativo; no se conceden privilegios polkit para aparentar soporte cpufreq.
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

### Mantenimiento de FAT y permisos de arranque

Solo para una particion FAT separada montada en `/boot`, con copia de seguridad
externa de los datos importantes y sin actualizaciones de kernel/bootloader en
curso. No ejecutar mientras otro administrador modifica montajes o el arranque.
El mantenimiento no instala kernels, cambia particiones ni toca la raiz ext4.

```bash
sudo pacman -S --needed dosfstools rtkit
sudo bash scripts/repair-boot.sh --apply
sudo bash scripts/setup-autostart.sh
```

Ejecutar cada paso solo si el anterior termina correctamente. El reparador valida
que fstab y el montaje coincidan, rechaza submontajes/dispositivos compartidos,
guarda fstab y una imagen de la FAT desmontada en un directorio privado bajo
`/var/backups/emubox`, ejecuta `fsck.fat -a` y verifica con `fsck.fat -n` antes
de remontar. Conserva UUID y opciones ajenas a las mascaras, aplica
`fmask=0077,dmask=0077` y mantiene passno=2 para fsck en futuros arranques.
Un volumen ocupado no se desmonta a la fuerza. Si la reparacion falla, intenta
montarlo de solo lectura; ante fallo posterior de montaje recupera el fstab previo.
Revisar siempre el resultado antes de reiniciar o actualizar el kernel.

La imagen local no sustituye un respaldo fuera de la VM ni verifica el disco del
anfitrion. Un corte durante el mantenimiento sigue requiriendo recuperacion manual.
Si `/boot` no se puede remontar, utilizar consola/rescate y el respaldo; no reiniciar
el sistema a ciegas. Para otros esquemas de arranque hace falta otro procedimiento.

Tras aplicar: `node scripts/check-appliance.mjs` comprueba herramientas, privacidad
de `/boot` y limites de getty. RTKit instalado no certifica sonido audible. Los
avisos historicos del journal no desaparecen al reparar; comprobar el siguiente
arranque ordenado. No desactivar fsck ni reducir el loglevel para ocultarlos.

Cuando termine todo el mantenimiento, `sudo systemctl restart getty@tty1` carga
la sesion actualizada sin reiniciar el SO. Para validar un arranque completo hace
falta un reinicio ordenado posterior, decidido por el usuario. Nunca usar reset o
corte de corriente virtual como sustituto de `systemctl poweroff` desde el invitado.

Verificar en cada CPU admitida:

1. `file bin/emubox` y arquitectura/target nativos.
2. Autologin `emubox`, permisos DRM, runtime XDG y bus de sesión.
3. Diagnóstico real de CPU, RAM, GPU, Vulkan, DRM, Gamescope y modelo del equipo.
4. Backend acelerado si existe, con compositor compatible; software solo sin aceleracion.
5. UI visible y manejable con mando, sin dependencia de SSH.
6. UI, IPC, persistencia SQLite, mando y audio sin emuladores instalados.
7. Arranque en frío y reconexión de pantalla cuando sea compatible.

Las pruebas unitarias no sustituyen estas comprobaciones físicas. La CI sobre
Ubuntu ARM valida compilación nativa, no instalación de paquetes en Arch Linux ARM.
El lanzamiento de motores/cores nativos es una fase posterior independiente.