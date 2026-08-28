# Arquitectura de Arranque Autónomo de Consola (Appliance Lifecycle)

Este documento define la arquitectura de ciclo de vida y arranque del sistema **EmuBox** como un dispositivo de consola dedicada (*Appliance*) bajo **Arch Linux (Direct DRM/KMS + Gamescope)**.

---

## 1. Principio Fundamental: Desacoplamiento SSH vs Sesión Local

```
                    ┌──────────────────────────────────────────────────┐
                    │                 1. SYSTEMD / TTY1                │
                    │   - Autologin del usuario 'emubox' en seat0      │
                    │   - Aislamiento de sesión PAM y D-Bus            │
                    └─────────────────────────┬────────────────────────┘
                                              │
                                              ▼
                    ┌──────────────────────────────────────────────────┐
                    │                 2. CAGE KIOSK                    │
                    │   - Compositor Wayland de sesión dedicada        │
                    │   - Gestión limpia de asiento sin Desktop Env    │
                    └─────────────────────────┬────────────────────────┘
                                              │
                                              ▼
                    ┌──────────────────────────────────────────────────┐
                    │                 3. GAMESCOPE                     │
                    │   - Entorno de composición para videojuegos      │
                    │   - Resolución 1080p/4K, FSR, DRM/KMS, FPS       │
                    └─────────────────────────┬────────────────────────┘
                                              │
                                              ▼
                    ┌──────────────────────────────────────────────────┐
                    │                 4. EMUBOX OS                     │
                    │   - Interfaz nativa Tauri v2 + SolidJS 1.9       │
                    │   - Control 10-Foot UI por mando físico          │
                    └──────────────────────────────────────────────────┘
```

### ¿Por qué la combinación Cage + Gamescope?

Cage y Gamescope no son alternativas excluyentes, sino **capas complementarias con propósitos diferenciados**:

* **Cage (Capa de Sesión Wayland)**: Proporciona una sesión Wayland de kiosko minimalista y ultra-ligera en `tty1`. Su único trabajo es gestionar el asiento de hardware (`seat0`) y ejecutar una única aplicación sin arrastrar gestores de ventanas ni entornos de escritorio pesados (GNOME/KDE/XFCE).
* **Gamescope (Capa de Composición y Rendimiento de Consola)**: Se ejecuta dentro de la sesión de Cage para brindar las capacidades avanzadas de una consola moderna de videojuegos (aislamiento de resolución, sincronización VSync estricta, escalado FSR/Integer scaling y gestión dinámica de ventanas de emuladores).
* **EmuBox (Capa de Aplicación)**: El frontend WebKitGTK/Tauri que renderiza la interfaz cinematográfica Obsidian & Neón.

---

## 2. Flujo y Cadena de Arranque Autónomo

1. **Arranque del Sistema**: Systemd alcanza el target `graphical.target`.
2. **Autologin Local en TTY1**: `getty@tty1` inicia sesión automáticamente con el usuario `emubox` con sesión PAM válida y asignación de asiento `seat0`.
3. **Inicialización de la Sesión Wayland**:
   - Se crea el directorio de runtime de usuario `$XDG_RUNTIME_DIR` (`/run/user/<uid>`).
   - Se asigna `$XDG_SESSION_TYPE=wayland` y el bus D-Bus de sesión.
4. **Lanzamiento de Cage**:
   - Cage inicia como compositor Wayland raíz sobre la TTY1.
5. **Anidación de Gamescope**:
   - Cage invoca Gamescope con los parámetros de consola (`gamescope -f -W 1920 -H 1080 -r 60 -- /opt/emubox/bin/emubox`).
6. **Arranque de EmuBox**:
   - EmuBox se enlaza a Gamescope y se despliega en pantalla física a 60/120 FPS sin dependencia de SSH.

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
