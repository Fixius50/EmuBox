# Directorio de capacidades Tauri

Este directorio debe existir: `tauri-build` lo registra como entrada de compilacion.
Su ausencia invalida la cache de Cargo en cada build, aunque no cambie el codigo.
Este archivo no concede permisos IPC ni habilita capacidades adicionales.

`main-events.json` autoriza exclusivamente `core:event:allow-listen` y
`core:event:allow-unlisten` para la ventana local `main`, cuyo label se declara
en `tauri.conf.json`. Permite recibir los estados de startup y las actualizaciones
de biblioteca, y liberar las suscripciones. No concede emision desde JavaScript,
acceso remoto ni el conjunto completo `core:default`.

Tauri descubre el archivo al compilar. Tras cambiar capacidades es necesario
recompilar y cargar el nuevo binario; reiniciar el antiguo no aplica los permisos.
`tests/native-backend.test.ts` comprueba el alcance y la seleccion de la capacidad.