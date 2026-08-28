# Arquitectura de Arranque Autónomo de Consola (Appliance Lifecycle)

Este documento define la arquitectura de ciclo de vida, arranque autónomo y políticas de resiliencia del sistema **EmuBox** como un dispositivo de consola dedicada (*Appliance*) bajo **Arch Linux (Direct DRM/KMS + Gamescope)**.

---

## 1. Principio Fundamental: Desacoplamiento SSH vs Sesión Local

```
                    ┌──────────────────────────────────────────────────┐
                    │                 1. SYSTEMD / TTY1                │
                    │   - Autologin del usuario 'emubox' en seat0      │
                    │   - Aislamiento de sesión PAM y D-Bus de usuario │
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

* **Cage (Capa de Sesión Wayland / Kiosko)**: Proporciona una sesión Wayland minimalista y ultra-ligera en `tty1`. Su único trabajo es gestionar el asiento de hardware (`seat0`) y ejecutar una única aplicación sin arrastrar gestores de ventanas ni entornos de escritorio pesados (GNOME/KDE/XFCE).
* **Gamescope (Capa de Composición y Rendimiento de Consola)**: Se ejecuta dentro de la sesión de Cage para brindar las capacidades avanzadas de una consola moderna de videojuegos (aislamiento de resolución, sincronización VSync estricta, escalado FSR/Integer scaling y gestión dinámica de ventanas de emuladores).
* **EmuBox (Capa de Aplicación)**: El frontend WebKitGTK/Tauri que renderiza la interfaz cinematográfica Obsidian & Neón.

> 📌 **Regla de Oro de Administración**:  
> **SSH es exclusivamente una puerta de mantenimiento, diagnóstico y despliegue remoto.**  
> EmuBox jamás se ejecuta dentro de la sesión SSH ni depende de que exista una conexión activa. La interfaz gráfica se levanta y persiste de forma autónoma en la pantalla física de la máquina.

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

---

## 4. Política de Resiliencia y Recuperación (Anti-Crash Loops)

Para evitar bucles masivos de reinicios (ej. `Restart=on-failure` superando cientos de intentos):
1. **Límites de Reinicio Estrictos**: Configurar `StartLimitIntervalSec=60s` y `StartLimitBurst=3` en systemd.
2. **TTY de Rescate**: Si el entorno gráfico falla en `tty1`, el sistema no se bloquea; se conservan `tty2` y `tty3` para acceso manual directo o administración vía SSH.
3. **Persistencia de Permisos**: Todo el árbol `/opt/emubox` pertenece exclusivamente a `emubox:emubox`, previniendo bloqueos de acceso generados por ejecuciones con `sudo`.

---

## 5. Red y Conectividad Zero-Config (Offline-First)

La consola opera bajo la premisa de **cero fricción de red**:

```text
                           ENCENDER CONSOLA
                                  │
                                  ▼
                         NetworkManager Daemon
                                  │
                   ┌──────────────┴──────────────┐
                   ▼                             ▼
           Ethernet Conectado           Sin Cable Ethernet
                   │                             │
                   ▼                             ▼
             DHCP Automático             ¿Hay Wi-Fi Configurado?
                   │                             │
                   │                     ┌───────┴───────┐
                   │                     ▼               ▼
                   │                   Sí (Auto)       No (Manual)
                   │                     │               │
                   │                     ▼               ▼
                   │                Conectar SSIDs    Elegir red en UI
                   │                     │            e introducir clave
                   └──────────────┬──────┘               │
                                  ▼                      ▼
                            Online Ready         Offline Mode Activo
                       (Scraping / OTA / P2P)    (Juegos locales 100% listos)
```

* **Cero Configuración Manual de IPs**: El usuario no debe introducir IP, máscara de red, gateway ni DNS.
* **Filosofía Offline-First**: EmuBox arranca y ejecuta todos los emuladores, partidas guardadas e interfaz aunque no exista conexión a Internet.

---

## 6. Protocolo de Verificación Canónico (11 Pasos)

1. Validar configuración de unidades systemd (`systemctl`).
2. Validar override de `getty@tty1` (autologin).
3. Validar permisos de `/opt/emubox` (`emubox:emubox`).
4. Validar socket y permisos de Wayland.
5. Validar ejecución de Cage.
6. Validar anidación de Gamescope.
7. Validar arranque de NetworkManager.
8. Validar renderizado de EmuBox en local.
9. Reiniciar la VM / máquina física.
10. Observar aparición automática de la interfaz en pantalla física.
11. Cerrar completamente la sesión SSH y verificar que la consola permanece 100% activa.
