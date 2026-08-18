# EmuBox Benchmarking Protocol & Methodology

## 1. Definición Matemática y Rigurosa de Métricas

Para evitar ambigüedad en los resultados, todas las métricas de rendimiento se calculan a partir del registro completo de frame times crudos (`deltaMs` en cada `requestAnimationFrame`):

### 1.1 Métricas de Frame Time y Fluidez
Sea $T = \{t_1, t_2, \dots, t_N\}$ la serie temporal de duraciones de frame en milisegundos tomadas con `performance.now()` durante una prueba:
1. **Promedio de Frame Time ($\bar{t}$)**:
   $$\bar{t} = \frac{1}{N} \sum_{i=1}^{N} t_i$$
2. **FPS Promedio**:
   $$\text{FPS}_{\text{avg}} = \frac{1000}{\bar{t}}$$
3. **Percentiles de Frame Time ($p_{50}, p_{95}, p_{99}$)**:
   Ordenando $T$ en orden ascendente $T_{\text{sorted}}$, $p_k$ representa el valor en la posición $\lfloor \frac{k}{100} \times N \rfloor$.
4. **1% Low Frame Time ($t_{1\%\text{-slowest}}$) y 1% Low FPS**:
   Se define **inequívocamente** como la media aritmética del 1% de los frames más lentos (los percentiles del 99 al 100 en duración):
   $$t_{1\%\text{-slowest}} = \frac{1}{\lceil 0.01 \cdot N \rceil} \sum_{i = N - \lceil 0.01 \cdot N \rceil + 1}^{N} T_{\text{sorted}}[i]$$
   $$\text{FPS}_{1\%\text{ Low}} = \frac{1000}{t_{1\%\text{-slowest}}}$$
5. **Conteo de Frames con Caída de Presupuesto (Jank Count)**:
   * $N_{>16.67\text{ms}}$: Frames con duración superior a 16.67 ms (por debajo de 60 Hz).
   * $N_{>33.33\text{ms}}$: Frames con duración superior a 33.33 ms (por debajo de 30 Hz).
   * $N_{>50.00\text{ms}}$: Frames con duración superior a 50.00 ms (micro-congelación severa).

---

## 2. Comparativa de Arquitecturas de Entrada (Gamepad API vs Tauri IPC)

El laboratorio evalúa dos flujos de entrada:
1. **Flujo A (Web Gamepad API Directo)**:
   `Gamepad Hardware -> Web Gamepad API (navigator.getGamepads) -> Polling RAF Loop -> Spatial Nav Engine -> Frontend State`
2. **Flujo B (Simulación de Tauri IPC Event Bridge)**:
   `Gamepad Hardware (gilrs/Rust) -> Tauri IPC Event Dispatch -> Frontend Listener -> Spatial Nav Engine -> Frontend State`

---

## 3. Escalas de Datos Evaluadas

* **Dataset de Renderizado Visual**:
  * Colección Pequeña: `20` juegos
  * Colección Mediana: `100` juegos
  * Colección Grande: `500` juegos
* **Dataset de Operaciones en Memoria (Filtrado, Búsqueda, Ordenación, Metadatos)**:
  * `500` registros
  * `1.000` registros
  * `5.000` registros
  * `10.000` registros

---

## 4. Desglose de Dimensiones de Distribución y Empaquetado

En los resultados finales se separan explícitamente:
1. **Frontend Payload (Crudo & Comprimido)**:
   * JS crudo, JS gzip, JS brotli.
   * CSS crudo, CSS gzip, CSS brotli.
2. **Tauri Standalone Executable**: Tamaño del binario compilado en modo release (`target/release/emubox`).
3. **Installer / Package**: Tamaño del paquete comprimido (.deb / .AppImage / .tar.gz en Linux; .msi/.exe en Windows).
4. **Total Distribution Footprint**: Tamaño total en disco instalado.

---

## 5. Métrica de Coste de Desarrollo y Mantenibilidad

Para cada candidato se registran cuantitativa y cualitativamente:
* **LOC (Lines of Code)**: Código fuente efectivo (sin comentarios ni imports triviales).
* **Número de Archivos**: Cantidad de archivos requeridos para el launcher completo.
* **Complejidad Ciclomática / Cognitiva**: Grado de anidamiento y reactividad.
* **Prueba Práctica de Extensibilidad**:
  * Tarea: *"Añadir filtros simultáneos por plataforma, género, año y favoritos, completamente navegables con gamepad"*.
  * Métrica: Tiempo de implementación, líneas añadidas, archivos tocados y puntos de fricción/errores encontrados.

---

## 6. Batería de Suites de Prueba Automatizadas

1. **Suite 1: Cold Start & TTFI**: Medición de tiempo de montaje inicial con 500 juegos.
2. **Suite 2: D-Pad Fast Stress Run**: 200 pulsaciones direccionales consecutivas en 10 segundos registrando distribución de frame times.
3. **Suite 3: Modal & View Leak Test**: 500 aperturas y cierres automatizados de la vista de detalles y ajustes para verificar limpieza de memoria y event listeners.
4. **Suite 4: Soak Test (30 Minutos)**: Navegación continua en bucle para detectar degradación progresiva de FPS, crecimiento de RSS del SO y pérdidas de foco.
5. **Suite 5: Data Operations Stress**: Filtrado y ordenación sobre 10.000 registros en memoria.
