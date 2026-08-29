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

## 2. Flujo y Matriz de Decisión de Arranque Adaptativo

```text
                           ARRANQUE ARCH LINUX
                                    │
                                    ▼
                             systemd / tty1
                                    │
                                    ▼
                         Autologin usuario 'emubox'
                                    │
                                    ▼
                         emubox-session-manager
                                    │
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
          Detectar Hardware / VM           Sondear Vulkan Real
          (systemd-detect-virt)           (vulkaninfo --summary)
                    │                               │
                    └───────────────┬───────────────┘
                                    ▼
                         ¿Vulkan HW Operativo?
                                    │
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
            SÍ: MODO NATIVO                NO: MODO COMPATIBILIDAD
        (Hardware PC dedicado)             (VMware / VirtualBox / Legacy)
                    │                               │
                    ▼                               ▼
         Cage Wayland Kiosk              Cage Wayland Kiosk
                    │                               │
                    ▼                               │
          Gamescope Compositor                      │
          (1080p/4K FSR, VSync)                     │
                    │                               │
                    └───────────────┬───────────────┘
                                    ▼
                              EMUBOX TAURI OS
                     (Pantalla física a 60 FPS estables)
```

1. **Arranque del Sistema**: Systemd alcanza el target `graphical.target`.
2. **Autologin Local Único en TTY1**: `getty@tty1` inicia sesión automáticamente con el usuario `emubox` (mediante el archivo único `/etc/systemd/system/getty@tty1.service.d/emubox-autologin.conf`) con sesión PAM válida y asignación de asiento `seat0`.
3. **Inicialización de la Sesión Wayland**:
   - Se crea el directorio de runtime de usuario `$XDG_RUNTIME_DIR` (`/run/user/<uid>`).
   - Se asigna `$XDG_SESSION_TYPE=wayland` y el bus D-Bus de sesión.
4. **Sondeo de Capacidades Gráficas**:
   - `emubox-session` realiza una comprobación activa de DRM (`/dev/dri/card0`) y de Vulkan real mediante `vulkaninfo --summary`.
5. **Selección Dinámica del Compositor**:
   - **Modo Nativo (GPU física con Vulkan OK)**: Lanza `Cage -> Gamescope -> EmuBox`.
   - **Modo Compatibilidad (VM sin Vulkan físico)**: Lanza `Cage -> EmuBox` directamente, evitando de raíz el fallo `vkCreateInstance failed` de Gamescope.
6. **Despliegue Visual**:
   - EmuBox se renderiza a pantalla completa en la salida de vídeo física sin bucles de reinicio.

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
2. Validar override de `getty@tty1` (autologin con `emubox-autologin.conf`).
3. Validar permisos de `/opt/emubox` (`emubox:emubox`).
4. Validar socket y permisos de Wayland.
5. Validar ejecución de Cage.
6. Validar anidación de Gamescope.
7. Validar arranque de NetworkManager.
8. Validar renderizado de EmuBox en local.
9. Reiniciar la VM / máquina física (`sudo reboot`).
10. Observar aparición automática de la interfaz en pantalla física (Cold boot).
11. Cerrar completamente la sesión SSH y verificar que la consola permanece 100% activa.

---

## 7. Administración Remota por SSH y Desacoplamiento de Pseudo-Terminales (`pts/*`)

La administración se realiza de forma no invasiva por SSH sin interactuar con la sesión gráfica física:

```bash
ssh emubox@IP_DE_LA_VM
```

* **Aislamiento de Terminal**: La sesión SSH se asigna a `/dev/pts/X`. El archivo `/home/emubox/.bash_profile` comprueba estrictamente `[ "$(tty)" == "/dev/tty1" ]`, por lo que **conectarse o desconectarse por SSH jamás intentará arrancar una segunda instancia gráfica de EmuBox ni cerrará la que corre en pantalla**.
* **Comprobación de IP**: Consultar mediante `ip addr` o `hostname -I`.

---

## 8. Flujo de Desarrollo y Despliegue con Git

El código fuente principal reside y se compila en `/opt/emubox`, vinculado al repositorio remoto `Fixius50/EmuBox`:

```text
PC de Desarrollo ──────────────► Git Commit & Push ──────────────► GitHub (Fixius50/EmuBox)
                                                                           │
VM / Consola EmuBox ◄─────────── Compilar & Validar ◄──────────── Git Pull (SSH Auth)
```

### Autenticación y Despliegue Seguro:
1. **Autenticación SSH para GitHub**: Verificar con `ssh -T git@github.com` antes de operaciones remotas.
2. **Ciclo de Actualización**:
   ```bash
   cd /opt/emubox
   git pull
   bash scripts/build.sh
   ```
3. **Invariante de Despliegue**: Los ejecutables en `/usr/local/bin` son generados automáticamente por los scripts del repositorio; **prohibido editarlos manualmente**.

---

## 9. Hoja de Ruta de Desarrollo Tras la Validación de Arranque

Con el ciclo de encendido y el stack Wayland 100% blindados, el trabajo pasa directamente a las funcionalidades de consola:

1. **Catálogo y Biblioteca**: Virtualización fluida sobre 10.000 juegos y animaciones Anime.js.
2. **Experiencia 10-Foot UI**: Navegación espacial 2D por mando y mapeo de iconos dinámicos (PS/Xbox/Nintendo).
3. **Gestión de ROMs y Almacenamiento**: Auto-montaje de pendrives USB y categorización automática por extensión.
4. **Enrutamiento de Emuladores**: Selección inteligente de cores Vulkan Standalone vs Libretro.
5. **Firmware y BIOS**: Escaneo y enlace automático por checksums criptográficos MD5/SHA1.
6. **Backend Rust / Tauri IPC**: Conexión de eventos nativos de gamepad vía `gilrs` y telemetría de hardware.
7. **Sistema de Actualizaciones Desacopladas (OTA)**: Descargas atómicas en `/opt/emubox/releases/`.
8. **Pruebas en Hardware Físico**: Validación final del modo NATIVO con GPU dedicada AMD/NVIDIA (`Cage -> Gamescope -> EmuBox`).

