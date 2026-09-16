# Aislamiento de ejecucion

Descargado, preparado e instalado no significan confiable. La preparacion conserva
su sandbox y validaciones; todo lanzamiento desde ProcessService utiliza ahora
Bubblewrap. No existe fallback a ejecucion directa.

## Frontera de confianza

`runtime/launch_policy.rs` selecciona el perfil compilado del registro nativo.
La ruta y los argumentos arbitrarios del registro SQLite no determinan el comando.
Las herramientas se resuelven por candidatos absolutos o `/usr/bin`, sin PATH
heredado; binarios y cores deben estar bajo `/usr` o `/opt/emubox/bin`, ser propiedad
de root y tener todos sus ancestros protegidos contra escritura de grupo/otros.
Un AppImage bajo un repositorio modificable por emubox queda bloqueado. Debe ser
desplegado por el administrador en una ubicacion protegida admitida por su perfil;
no se cambian automaticamente propietarios ni permisos del repositorio.

`LaunchGameRequest` conserva campos antiguos para devolver errores explicitos:
rechaza `romPath`, `customArgs` no vacio, slot de estado y fullscreen=false.
Las asociaciones con argumentos o config personalizados tambien se rechazan.
Solo se usa la ruta instalada de SQLite, canonicalizada dentro de la raiz de juegos;
no se aceptan enlaces que escapen de ella. Wine exige marcador, archivos y candidato
PE seleccionado de una instalacion gestionada. Las instalaciones existentes no se
mueven ni se reescriben sus IDs.

`execute_command` y su adaptador frontend han sido eliminados, no ocultados por UI.
`kill_process` solo admite el Child del sandbox activo, nunca PIDs arbitrarios.
La creacion se serializa y la consulta de estado recoge el hijo terminado.
Un puente shell fijo emite una confirmacion desde dentro del sandbox ya montado
y ejecuta el perfil con `exec "$@"`, sin interpolar texto de la UI ni del catalogo.
Si no confirma en diez segundos, se termina y devuelve error. El anuncio JSON
`child-pid` de Bubblewrap no basta: ocurre antes de finalizar los montajes.
Esta confirmacion tampoco certifica que el emulador haya cargado el juego.

## Perfiles y accesos

| Recurso | ROM / emulador nativo | PC / Wine |
| --- | --- | --- |
| Contenido | ROM suelta o directorio del juego/paquete, solo lectura | Paquete preparado seleccionado, solo lectura |
| Estado | Home privado por hash de gameId y emulatorId | Igual, con WINEPREFIX privado |
| Red | Namespace separado, sin conexiones al host ni Internet | Igual; sin excepciones de red por IPC |
| Entorno | Limpiado tanto en Command como en Bubblewrap | Igual; Mono/Gecko desactivados |
| Pantalla | Socket Wayland concreto; nunca X11 del host | Gamescope anidado obligatorio dentro del sandbox |
| GPU | Nodos renderD de DRM, sin card ni acceso a todo /dev | Igual |
| Audio | ALSA: PCM playback y controles, sin captura | Igual |
| Entrada directa | Solo dispositivos udev joystick, excluyendo teclado/raton | Igual |

El estado reside en `/var/lib/emubox/sandbox/<sha256(gameId + NUL + emulatorId)>/`
y aparece como `/home/player`: `.config`, `.local/share`, `.cache`, `saves`,
`states`, `screenshots` y `wine`. Directorios con enlaces se rechazan. RetroArch
recibe rutas explicitas de guardado/estado, BIOS en `/bios`, capturas privadas y
audio ALSA. RPCS3 recibe firmware y juegos preparados en solo lectura, con el resto
de su configuracion privada. No se importan automaticamente configs o partidas
del home real ni de prefijos Wine antiguos.

La raiz contiene `/usr` de solo lectura, librerias via symlinks, fuentes,
cache del cargador, zona horaria, sysfs de dispositivos/DRM, BIOS administradas,
proc del namespace y dispositivos minimos. No se monta `/`, `/home`, `/etc` completo,
el repositorio, la base SQLite, otros juegos, sockets SSH, D-Bus, PipeWire ni PulseAudio.
El ultimo se excluye porque un cliente puede controlar el servidor de audio del host.
Solo se conserva el socket Wayland de la sesion, no todo XDG_RUNTIME_DIR.

Se crean namespaces de usuario, PID, red e IPC, se quitan capabilities, se deshabilita
la creacion de nuevos user namespaces y se desconecta la terminal. El proceso muere
con su supervisor. prlimit impide core dumps, limita archivos individuales a 20 GiB
y descriptores a 2048. Tmpfs: /tmp 512 MiB, /run 16 MiB, /dev/shm 128 MiB.

## Limites y despliegue

Se requieren Bubblewrap con disable-userns, util-linux/prlimit y
namespaces habilitados. Ya son dependencias de preparacion de la appliance.
Wine requiere ademas Gamescope, Xwayland y capacidad grafica compatible.
No se ofrece una ruta X11 sin confinamiento si falta ese soporte.

No se han ejecutado juegos para verificar compatibilidad. Los perfiles con configs,
BIOS o preferencias antiguas pueden necesitar configuracion privada; el contenido
que exige escribir junto al ejecutable puede fallar por ser de solo lectura.
ALSA puede fallar si PipeWire ocupa el dispositivo o el emulador solo admite Pulse.
Las AppImages modificables por el usuario se rechazan deliberadamente.

Esto no es una VM ni una garantia contra escapes: kernel, GPU y compositor Wayland
siguen siendo parte de la frontera de confianza. El socket Wayland permite los
protocolos que ofrezca el compositor, potencialmente portapapeles/captura; debe usarse
en la sesion appliance dedicada, no como proteccion completa de un escritorio
personal con protocolos privilegiados. No hay filtro seccomp especifico ni cuotas
cgroup de CPU, memoria residente, procesos o disco agregado. Los limites por archivo
y tmpfs no impiden todas las formas de denegacion de servicio.

## Verificacion

Tests ordinarios cubren perfiles desconocidos, argumentos/rutas libres, escapes
por symlink, dispositivos mixtos y seleccion Windows gestionada. La prueba nativa
opt-in ejecuta solo un shell fijo con datos artificiales: verifica entorno vacio,
ausencia de home/repositorio/config del host, contenido no escribible, estado privado
escribible y rechazo de conexion al loopback del host. No ejecuta ROMs, Wine ni juegos.

```sh
cargo test --offline --manifest-path src-tauri/Cargo.toml services::runtime::game_sandbox::tests
cargo test --offline --manifest-path src-tauri/Cargo.toml native_sandbox_hides_host_data_and_keeps_only_private_writes -- --ignored
```