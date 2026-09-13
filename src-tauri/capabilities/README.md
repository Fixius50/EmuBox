# Directorio de capacidades Tauri

Este directorio debe existir: `tauri-build` lo registra como entrada de compilacion.
Su ausencia invalida la cache de Cargo en cada build, aunque no cambie el codigo.
Este archivo no concede permisos IPC ni habilita capacidades adicionales.