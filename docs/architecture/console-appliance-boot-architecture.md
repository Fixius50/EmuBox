# Arquitectura de Arranque Autónomo de Consola (Appliance Lifecycle)

Este documento define la arquitectura de ciclo de vida y arranque del sistema **EmuBox** como un dispositivo de consola dedicada (*Appliance*) bajo **Arch Linux (Direct DRM/KMS + Gamescope)**.

---

## 1. Principio Fundamental: Desacoplamiento SSH vs Sesión Local

```
                  ┌──────────────────────────────────────────────────┐
                  │                 SESIÓN REMOTA                    │
                  │             SSH (Admin / Tooling)                │
                  │   - Instalación, configuración y diagnósticos    │
                  │   - Cero dependencia de DISPLAY / Wayland local  │
                  │   - Puede cerrarse sin afectar la consola        │
                  └─────────────────────────┬────────────────────────┘
                                            │
                                  Aprovisiona y habilita
                                            │
                                            ▼
                  ┌──────────────────────────────────────────────────┐
                  │                 SISTEMA BASE                     │
                  │         Arch Linux + Systemd (seat0)             │
                  │   - Autologin en consola física (tty1)           │
                  │   - Sesión de usuario emubox independiente       │
                  └─────────────────────────┬────────────────────────┘
                                            │
                                    Lanza en pantalla
                                            │
                                            ▼
                  ┌──────────────────────────────────────────────────┐
                  │                 COMPOSITOR                       │
                  │       Wayland: Gamescope / Cage (DRM/KMS)        │
                  │   - Adquiere el nodo /dev/dri/card0              │
                  │   - Resolución nativa y escalado FSR             │
                  └─────────────────────────┬────────────────────────┘
                                            │
                                       Ejecuta
                                            │
                                            ▼
                  ┌──────────────────────────────────────────────────┐
                  │                 INTERFAZ                         │
                  │      EmuBox OS (Tauri v2 + SolidJS 1.9)          │
                  │   - Pantalla de la VM / Monitor HDMI / TV        │
                  │   - Operación 100% por mando y teclado           │
                  └──────────────────────────────────────────────────┘
```

> 📌 **Regla de Oro**:  
> **SSH es únicamente el mando de administración y mantenimiento remoto.**  
> EmuBox jamás se ejecuta dentro de la sesión SSH ni depende de que exista una conexión activa. Toda la interfaz gráfica reside y se renderiza exclusivamente en la sesión física local de la máquina/pantalla.

---

## 2. Flujo y Cadena de Arranque Autónomo

1. **Arranque del Sistema**: Systemd alcanza el target `graphical.target`.
2. **Autologin Local en TTY1**: `getty@tty1` inicia sesión automáticamente con el usuario del sistema (`emubox`) con sesión PAM válida y asignación de asiento `seat0`.
3. **Inicialización de la Sesión Wayland**:
   - Se crea el directorio de runtime de usuario `$XDG_RUNTIME_DIR` (`/run/user/<uid>`).
   - Se asigna `$XDG_SESSION_TYPE=wayland` y el bus D-Bus de sesión.
4. **Compositor de Consola (Gamescope)**:
   - Gamescope toma el control del nodo DRM/KMS (`/dev/dri/card0`).
   - Expone la variable `$WAYLAND_DISPLAY` para la ventana cliente.
5. **Arranque de EmuBox**:
   - EmuBox se conecta al socket de Wayland expuesto por Gamescope.
   - La interfaz aparece a pantalla completa en la salida de vídeo física (10-foot UI a 60/120 FPS).

---

## 3. Especificaciones del Stack Gráfico

| Componente | Tecnología Seleccionada | Razón Arquitectónica |
| :--- | :--- | :--- |
| **Protocolo de Pantalla** | **Wayland puro** (sin Xorg/X11) | Menor latencia de entrada, sincronización perfecta sin tearing, estándar moderno de Linux. |
| **Compositor Principal** | **Gamescope** | Compositor micro-Wayland de Valve/SteamOS optimizado para juegos, DRM-KMS directo y FSR. |
| **Compositor Kiosko Alternativo** | **Cage** | Compositor Wayland minimalista de ventana única para entornos ligeros o pruebas. |
| **Motor Frontend** | **WebKitGTK 4.1 + Tauri v2** | Renderizado HTML/CSS de ultra-alto rendimiento con reactividad de grano fino en SolidJS. |
| **Pipeline de Renderizado** | **Autónomo (GPU vs CPU)** | Detección automática en arranque; fallback transparente sin blur pesado en entornos sin GPU acelerada. |
| **Sesión & Ciclo de Vida** | **Systemd + getty@tty1** | Persistencia total ante reinicios sin dependencia de conexiones de red externas. |
