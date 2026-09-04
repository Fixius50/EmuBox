# Arquitectura de Arranque Autónomo de Consola (Appliance Lifecycle)

Este documento define la arquitectura de ciclo de vida, arranque autónomo y políticas de resiliencia del sistema **EmuBox** como un dispositivo de consola dedicada (*Appliance*) bajo **Arch Linux (Direct DRM/KMS + Gamescope)**.

---

## 1. Principio Fundamental: Desacoplamiento SSH vs Sesión Local

```
                    ┌──────────────────────────────────────────────────┐
                    │                 1. SYSTEMD / TTY1                │
                    │   - Autologin de la appliance en seat0           │
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

* **Gamescope (Capa de Composición y Rendimiento para GPU Acelerada)**: Se ejecuta directamente para brindar las capacidades avanzadas de una consola moderna de videojuegos (DRM/KMS directo, sincronización VSync estricta, escalado FSR y gestión dinámica de ventanas de emuladores en GPUs AMD/Intel/NVIDIA).
* **Cage (Capa de Sesión de Emergencia / Fallback CPU)**: Proporciona una sesión Wayland minimalista y ultra-ligera en `tty1` cuando no existe aceleración gráfica física por hardware (`llvmpipe`, `softpipe` o entornos de virtualización sin aceleración funcional).
* **EmuBox (Capa de Aplicación)**: El frontend WebKitGTK/Tauri que renderiza la interfaz cinematográfica Obsidian & Neón con SolidJS 1.9.

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
                         Autologin de la appliance
                                    │
                                    ▼
                         emubox-session-manager
                                    │
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
          Detectar Hardware / Driver       Sondear Renderer Real
         (amdgpu / i915 / xe / nvidia)     (Descartar llvmpipe/soft)
                    │                               │
                    └───────────────┬───────────────┘
                                    ▼
                      ¿Aceleración GPU por Hardware?
                                    │
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
            SÍ: GPU ACELERADA               NO: FALLBACK CPU
        (AMD, Intel, NVIDIA nativo)     (llvmpipe / Software / VM)
                    │                               │
                    ▼                               ▼
          Gamescope Compositor              Cage Wayland Kiosk
        (DRM/KMS directo, FSR)              (Wayland Kiosk liviano)
                    │                               │
                    └───────────────┬───────────────┘
                                    ▼
                        emubox-drm-sync (udev 0% CPU)
                                    │
                                    ▼
                             EMUBOX TAURI OS
                   (Resolución y Viewport 100% Dinámicos)
```

1. **Arranque del Sistema**: Systemd alcanza el target `graphical.target`.
2. **Autologin Local Único en TTY1**: `getty@tty1` inicia la sesión de la appliance mediante `/etc/systemd/system/getty@tty1.service.d/emubox-autologin.conf`, con sesión PAM válida y asignación de asiento `seat0`. La configuración actual puede ejecutar la sesión como `root` para el acceso directo requerido a DRM y rutas del sistema; SSH permanece aislado.
3. **Inicialización de la Sesión Wayland**:
   - Se crea el directorio de runtime de usuario `$XDG_RUNTIME_DIR` (`/run/user/<uid>`).
   - Se asigna `$XDG_SESSION_TYPE=wayland` y el bus D-Bus de sesión.
4. **Sondeo de Capacidades Gráficas y Renderer**:
   - `emubox-session` realiza una comprobación activa de PCI/DRM y analiza el renderer OpenGL/Vulkan, descartando expresamente rasterizadores software como `llvmpipe` o `softpipe`.
5. **Selección del Pipeline**:
   - **GPU Acelerada (Hardware Real)**: Lanza `Gamescope -> EmuBox`.
   - **Fallback CPU (Emergencia)**: Lanza `Cage -> EmuBox` directamente cuando no hay aceleración física.
6. **Sincronización Dinámica de Resolución (`emubox-drm-sync`)**:
   - Permanece en segundo plano bloqueado en el socket de `udevadm monitor` (0% CPU).
   - Ante eventos `HOTPLUG=1` del kernel (redimensionado de ventana en hipervisores o cambio de TV/monitor en hardware), sincroniza `wlr-randr` en caliente hacia la UI SolidJS sin resoluciones fijas ni reinicios de la aplicación.

7. **Recarga systemd**: los instaladores ejecutan `systemctl daemon-reload` tras crear unidades del sistema y `systemctl --user daemon-reload` tras crear unidades de usuario.

8. **Catálogo de juegos**: Tauri escanea primero `/var/lib/emubox/games` y SQLite. Si no hay ROMs, SolidJS carga `data/games-10000.json` como catálogo visible con estado no instalado.

---

## 3. Especificaciones del Stack Gráfico

| Componente | Tecnología Seleccionada | Razón Arquitectónica |
| :--- | :--- | :--- |
| **Protocolo de Pantalla** | **Wayland puro** (sin Xorg/X11) | Menor latencia de entrada, sincronización perfecta sin tearing, estándar moderno de Linux. |
| **Pipeline GPU Principal** | **Gamescope** | Compositor micro-Wayland de Valve/SteamOS optimizado para juegos, DRM-KMS directo y FSR para GPUs AMD/Intel/NVIDIA. |
| **Pipeline CPU de Emergencia** | **Cage** | Compositor Wayland minimalista de ventana única para renderizado software por CPU (`llvmpipe`) o entornos de virtualización. |
| **Sincronización de Resolución** | **emubox-drm-sync (udev)** | Listener reactivo puro de eventos `SUBSYSTEM=drm, HOTPLUG=1` con 0% de uso de CPU y propagación en caliente al viewport de SolidJS. |
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

* **Conexión Local (VirtualBox NAT Port Forwarding `2222 -> 22`)**:
  ```bash
  ssh -p 2222 emubox@127.0.0.1
  ```
* **Conexión en Red Local / Hardware Físico**:
  ```bash
  ssh emubox@<IP_DE_LA_CONSOLA>
  ```
* **Credenciales por defecto**: Usuario `emubox`, contraseña `1234`.
* **Aislamiento de Terminal**: La sesión SSH se asigna a `/dev/pts/X`. El archivo `/home/emubox/.bash_profile` comprueba estrictamente `[ "$(tty)" == "/dev/tty1" ]`, por lo que **conectarse o desconectarse por SSH jamás intentará arrancar una segunda instancia gráfica de EmuBox ni cerrará la que corre en pantalla**.

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

---

## 10. Consolidación Técnica de los Puntos Críticos del Sistema

| Punto | Área | Mecanismo de Implementación | Comportamiento en VM vs Hardware Físico |
| :--- | :--- | :--- | :--- |
| **Punto 1** | **Arranque / Appliance** | `getty@tty1` + `emubox-autologin.conf` + `.bash_profile` + `/usr/local/bin/emubox-session` | Cero dependencia de X11 / GDM. Arranque autónomo e invisible hacia pantalla completa. |
| **Punto 6** | **Actualización del Sistema** | `scripts/update-emubox.sh` (orquestado vía `./script.sh` [Opción 2]) | Pull `--ff-only`, delega build a `scripts/build.sh`, auto-stash, protección atómica ante fallos y reinicio de TTY1. |
| **Punto 7** | **Gestión Gráfica Adaptativa** | `vulkaninfo --summary` activo + `systemd-detect-virt` + `/dev/dri` | **VMware / VirtualBox**: `Cage -> EmuBox` (Vulkan evitado). **HW Físico**: `Cage -> Gamescope -> EmuBox`. |
| **Punto 8** | **Resolución Dinámica** | Sondeo de conectores DRM (`/sys/class/drm/*/modes`) | Ajuste automático a resolución nativa detectada (1080p/1440p/4K); fallback predeterminado: `1920x1080@60Hz`. |
| **Punto 9** | **Entrada y Gamepad** | Permisos en grupos `input`, `uinput`, `video`, `seat` + eventos `gilrs` | Mando como entrada primaria, teclado como soporte, aislamiento total de sesiones remotas SSH (`pts/*`). |
| **Punto 10** | **Recuperación y Robustez** | `StartLimitBurst=3` / `StartLimitIntervalSec=60s` + logging en `/var/log/emubox/` | Prevención de crash-loops; preservación de binario funcional si el build falla; consolas TTY2/TTY3 de rescate. |


