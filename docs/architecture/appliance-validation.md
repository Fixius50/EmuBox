# Runtime, appliance y distribucion

## Alcance actual

EmuBox tiene tres capas con evidencias distintas:

| Capa | Contenido | Evidencia necesaria |
| --- | --- | --- |
| Runtime | Rust/Tauri, SolidJS, IPC, SQLite y servicios | Compilacion nativa y tests |
| Appliance | Runtime, systemd, usuario emubox, filesystem, Wayland, input y audio | Arranque y pruebas funcionales en el equipo |
| Distribucion | Base, kernel, boot y empaquetado de la appliance | Imagen reproducible y pruebas de arranque |

El objetivo actual es Runtime + Appliance en `x86_64` y `aarch64`. La imagen
EmuBox OS de cada CPU es una fase posterior: no se introduce archiso, mkosi,
kernel propio, bootloader ni particionado para cumplir este hito. ARM32 no se admite.
Generic AArch64 no equivale a una placa concreta ni garantiza el firmware de todas
las placas; el entorno Linux de base debe arrancar primero en el dispositivo.

`scripts/setup-arch.sh` es la ruta manual/de desarrollo para convertir Arch Linux
x86_64 o Arch Linux ARM aarch64 en una appliance. `installer/install.sh` delega
en ella. No es un constructor de distribucion ni prueba que ARM este validado.

## Instalacion minima

La instalacion base no solicita RetroArch, cores ni emuladores standalone. Una
lista de emuladores vacia o con perfiles no instalados es un estado valido para
cargar ajustes, plataformas, catalogo y servicios. El lanzamiento de juegos tiene
su comprobacion de capacidades independiente.

La sesion es unica: getty@tty1, autologin `emubox`, perfil de TTY1,
`emubox-session` y `scripts/run.sh`. El servicio auxiliar queda deshabilitado.
PipeWire/WirePlumber se habilitan como servicios de usuario y systemd-tmpfiles
recrea `/run/emubox` despues de reiniciar. Los directorios persistentes son
`/etc/emubox`, `/var/lib/emubox`, `/var/cache/emubox` y `/var/log/emubox`.

Gamescope requiere Vulkan hardware, DRM y el ejecutable disponible; de lo contrario
se usa Cage. Cage puede usar OpenGL acelerado. La CPU no determina la GPU ni el
driver; disponibilidad detectada no garantiza renderizado correcto en cada programa.

Un administrador puede solicitar motores opcionales por separado:

```bash
sudo bash installer/setup/emulator-packages.sh
```

Este comando consulta los repositorios configurados con pacman. No promete todos
los motores en todas las CPU ni habilita emulacion/traduccion x86 sobre ARM.
No es necesario para la aceptacion de la appliance.

## Pruebas automaticas

```bash
npm run test:appliance
npm run test:architecture
```

La primera suite simula pacman para comprobar que la base no instala motores,
valida la configuracion generada de sesion/systemd y comprueba el bootstrap de
stores sin motores en ambas CPU simuladas. El evaluador de requisitos se prueba
con fixtures; no arranca systemd ni instala servicios durante los tests.

El workflow `Runtime Linux (native)` ejecuta estas pruebas junto al runtime sobre
runners x86_64 y ARM64 nativos. Ubuntu CI no valida los paquetes de Arch Linux ARM
ni una sesion Wayland fisica. No hay todavia pipeline de imagen o boot testing.

## Inspeccion de solo lectura

Ejecutar como `emubox`, sin sudo, en el equipo objetivo:

```bash
npm run check:appliance
node scripts/check-appliance.mjs --json
```

El comprobador lee arquitectura, ELF, bibliotecas, permisos, configuracion systemd,
runtime de usuario y procesos. Solo comprueba la cabecera de SQLite: no abre ni
modifica la base de datos. No instala paquetes, no reinicia servicios y no lanza
descargas, juegos ni emuladores.

- `PASS`: el requisito inspeccionado se cumple, no toda la funcion.
- `FAIL`: hay un requisito comprobable incumplido, como permisos o autologin root.
- `PENDING`: la funcion necesita una sesion activa o una comprobacion manual.
- Salida `1`: requisitos fallidos. Salida `2`: sin esos fallos, pero aceptacion
  funcional pendiente. Nunca devuelve exito total ni `applianceValidated: true`.
- `distributionValidated` siempre es `false`; no se inspecciona una imagen del SO.

Es normal que un proceso o dispositivo no se pueda comprobar desde una sesion
inadecuada. No ejecutar como root para ocultar un fallo de permisos de `emubox`.

## Aplicar configuracion pendiente

Reiniciar la VM no aplica los scripts editados del repositorio. En una instalacion
con dependencias ya presentes, un administrador puede regenerar la sesion:

```bash
sudo bash scripts/setup-autostart.sh &&
sudo systemctl restart getty@tty1
```

El primer comando cambia propietarios de los datos de EmuBox, configura audio,
tmpfiles, grupos, lanzador y autologin. No recompila. El segundo interrumpe la
sesion grafica y cualquier juego activo; coordinarlo antes de ejecutarlo.
Si faltan dependencias, usar primero la ruta de instalacion manual documentada.
Si el setup indica que falta `pipewire.socket`, instalar la pila de audio antes
de reintentarlo:

```bash
sudo pacman -Syu --needed pipewire pipewire-audio pipewire-pulse wireplumber &&
sudo bash scripts/setup-autostart.sh &&
sudo systemctl restart getty@tty1
```

`pacman -Syu` actualiza el sistema y puede pedir resolver conflictos con otro
servidor de audio. Revisar la transaccion; no forzar reemplazos ni ignorar errores.
El setup comprueba las unidades de usuario antes de modificar permisos o servicios.
No ejecutar el reinicio si la instalacion o el setup fallan. Despues, ejecutar
`npm run check:appliance` como `emubox` sin sudo.

Si `/var/lib/emubox/roms` es un directorio real, el setup se detiene para permitir
su migracion a `games`, sin borrarlo.

## Aceptacion funcional por equipo

Registrar CPU/arquitectura, distro, kernel, version del runtime, GPU/driver,
OpenGL/Vulkan, compositor, fecha y evidencia para cada punto:

| Prueba | Criterio |
| --- | --- |
| Arranque | Inicio en frio, sin SSH, una sesion TTY1 como emubox |
| UI | Tauri/SolidJS visible y navegable, no frontend mock del navegador |
| IPC | Diagnostico y operaciones reales del backend desde la UI |
| SQLite/filesystem | Guardar un favorito de un juego propio y comprobar persistencia al reiniciar |
| Input | Navegacion real con mando y teclado, permisos sin root |
| Audio | Salida audible correcta con PipeWire/WirePlumber y dispositivo elegido |
| Graficos | Renderer del proceso confirmado en logs; Gamescope donde sea viable y Cage como alternativa |
| Sin emuladores | Ajustes y biblioteca cargan sin motores instalados |

Conservar resultados `PASS`, `FAIL`, `PENDING` o `NO APLICA` con justificacion.
Un mock o un binario ELF correcto no cierra estas filas. La validacion de ROMs,
cores y motores se registra despues, separada del soporte de la appliance.

No se ha validado aqui una appliance AArch64 real ni una distribucion reproducible
en ninguna de las dos CPU. Esas comprobaciones requieren el equipo/entorno adecuado.