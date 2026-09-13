# Especificacion de producto EmuBox OS

Estado: propuesta para revision. Fecha: 13 de septiembre de 2026.
Se ha aprobado redactar esta especificacion, no construir una imagen ni declarar
hardware oficialmente soportado. Las decisiones pendientes requieren confirmacion.

## Alcance aprobado

- Mantener Runtime, Appliance y Distribution como capas separadas.
- EmuBox OS puede basarse en Arch, Linux, Mesa y systemd; no requiere kernel ni drivers propios.
- Usar aceleracion fisica o virtual utilizable, sin convertir Vulkan en requisito universal.
- No construir ISO, modificar particiones, instalar bootloaders ni cambiar el sistema anfitrion en esta tarea.
- No ejecutar Git, navegador, apagados o reinicios como parte de esta especificacion.

## Producto propuesto

Una consola dedicada que arranca sin SSH, ofrece biblioteca real y ajustes
operables con mando/teclado, conserva datos del usuario y puede recuperarse de una
actualizacion fallida. El runtime es reutilizable tanto en una appliance manual
como en una futura imagen de distribucion.

## Niveles de soporte

| Estado | Significado | Evidencia necesaria |
| --- | --- | --- |
| Soportado | Hardware objetivo oficialmente validado para una version concreta | Equipo/configuracion identificados y bateria de aceptacion completada |
| Compatible | Hay compatibilidad tecnica, pero no compromiso de soporte oficial | Pruebas parciales reproducibles y limites documentados |
| No soportado | Fuera del alcance oficial; no se garantiza funcionamiento | Restriccion explicita y motivo, no inferencia por fabricante |
| No determinado | Evidencia insuficiente | Pendiente de sondeo o validacion |

Estos estados son por equipo, version y configuracion, no por nombre generico de
CPU. Un ejecutable aarch64 correcto no convierte todas las placas ARM en soportadas.
Una GPU virtual acelerada tampoco valida todos los anfitriones e hipervisores.

## Matriz inicial para revision

| Familia | Situacion conocida | Propuesta, aun no compromiso |
| --- | --- | --- |
| PC x86_64 | Runtime nativo compilado; faltan equipos fisicos de referencia completos | Primera familia objetivo: PC UEFI, modelos por acordar |
| VM x86_64 | SVGA3D/OpenGL detectado; estabilidad visual aun pendiente | Entorno de desarrollo/aceptacion separado del hardware fisico |
| ARM64 | Contratos y flujo nativo preparados; sin aceptacion fisica completa | Seleccionar placas o plataforma UEFI concretas antes de incluirlas |
| ARM32 | Runtime excluido por la arquitectura actual | No soportado |
| Otras configuraciones | Sin evidencia suficiente | No determinado |

No se declara ninguna familia completa como oficialmente soportada en este documento.

## Responsabilidades

| Capa | Control de EmuBox | Delegado al stack base |
| --- | --- | --- |
| Runtime | UI, IPC, catalogo, fuentes, configuracion, lanzamiento y diagnosticos | Emulacion de consolas a los motores instalados |
| Appliance | Sesion, usuario, directorios, permisos previstos y configuracion de servicios | Kernel, DRM, drivers, compositor y audio a sus implementaciones |
| Distribution | Composicion/versionado de imagen, pruebas, entrega y recuperacion | Paquetes y actualizaciones upstream, integrados y validados por EmuBox |
| Anfitrion VM | Diagnosticar lo visible en el invitado | Passthrough, GPU expuesta, firmware virtual y aceleracion del hipervisor |

EmuBox no puede prometer extensiones graficas ausentes, compatibilidad universal
de juegos ni acceso a una GPU no expuesta. Una imagen controla versiones del stack,
pero no elimina limites del hardware ni errores upstream por si sola.

## Instalacion y almacenamiento

Estado actual: configuracion sobre Arch x86_64 o Arch Linux ARM aarch64 existente.
Propuesta: conservar esa ruta durante la validacion y evaluar despues una imagen
instalable para los equipos objetivo. Falta decidir si sera imagen de disco,
instalador interactivo o ambas; no se ha elegido archiso, mkosi ni otra herramienta.

Se mantienen las rutas de configuracion y datos de la appliance. Una futura
instalacion debe separar datos persistentes de componentes reemplazables, informar
antes de cualquier borrado y definir copia/restauracion. Esquema de particiones,
root mutable/inmutable, cifrado y dual boot quedan pendientes.

## Versiones, actualizacion y recuperacion

Propuesta: versionar por separado runtime, esquema de datos e imagen base;
publicar un manifiesto de paquetes y hashes con cada entrega. Fijar versiones y
repositorios o snapshots para reproducibilidad; un script contra repositorios
rolling actuales no garantiza reconstruir una imagen anterior.

Antes de implementar se deben decidir canales, firmas y custodia de claves,
politica de parches de seguridad, estrategia de rollback (por paquetes, snapshots
o imagen A/B), migraciones de datos y recuperacion offline. La OTA nativa actual
no esta implementada; no debe presentarse como base de una actualizacion atomica
del SO ya terminada.

## Aceptacion por equipo y version

1. Arranque en frio sin SSH, una unica sesion, salida a recuperacion documentada.
2. UI e IPC reales, sin dependencia de servicios de desarrollo.
3. Deteccion de GPU/API con motivos e identidad cuando sea verificable; ausencia
   de evidencia no se presenta como CPU confirmada.
4. Navegacion con mando y teclado, reconexion, audio audible y pantalla estable.
5. Persistencia de ajustes y partidas tras reinicio y actualizacion.
6. Funcionamiento de biblioteca y ajustes sin emuladores instalados.
7. Instalacion y recuperacion ante fallo controlado, solo cuando exista esa capa.
8. Resultados registrados por hardware, firmware, kernel, Mesa, compositor y build.

Los casos 7 y de actualizacion del SO no se marcan aprobados con pruebas del runtime.

## Decisiones para cerrar esta especificacion

| Decision | Recomendacion inicial | Pendiente de confirmar |
| --- | --- | --- |
| Primer objetivo | PC x86_64 UEFI; VM como validacion separada | Modelos y minimos CPU/RAM/GPU |
| ARM64 | Equipos concretos, no soporte generico ARM | Placas/firmware y prioridad |
| Entrega inicial | Appliance manual mientras se valida | Fecha y requisitos para introducir imagen |
| Root y actualizacion | Elegir segun recuperacion y capacidad de mantenimiento | Mutable, snapshots o A/B; firmas y canales |
| Datos | Preservar biblioteca/partidas fuera del componente reemplazable | Particiones, backup y migraciones |
| Compromiso de soporte | Versiones/equipos con evidencias publicadas | Responsable, duracion y politica de incidencias |

La aceptacion de esta tarea es una especificacion acordada. Hasta confirmar estas
decisiones, el documento sigue siendo una propuesta y no autoriza una cuarta
tarea de implementacion de distribucion.