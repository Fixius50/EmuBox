# Metodologia de validacion y rendimiento

Revision: 2026-09-16. Es un protocolo, no un informe de resultados.

## Evidencia

Registrar hardware/VM, kernel, Mesa, WebKit, compositor, resolucion, escala,
binario y datos usados, caches, fecha, comando, salida y repeticiones.
No comparar build caliente con compilacion de dependencias desde cero.

Separar latencia IPC/CPU, callbacks UI, presentacion de pixeles y juego real.
requestAnimationFrame no mide scanout ni demuestra ausencia de negro. La sonda
actual de dos callbacks cada cinco segundos no calcula FPS.

Para una serie completa de intervalos medir media, p50/p95/p99, conteos superiores
al presupuesto de refresco y media del 1% mas lento. Publicar muestras crudas y
metodo de percentiles. Medir CPU por proceso, RSS sin sumar hilos duplicados,
memoria disponible, swap y presion E/S. Separar JS/CSS comprimido, ELF y huella instalada.

## Casos

1. Arranque frio/cacheado hasta UI utilizable, con catalogo vacio y grande.
2. Navegacion rapida con mando/teclado, busqueda y retorno de foco.
3. Apertura/cierre de fichas y modales, recursos antes/despues.
4. Sesion prolongada, marca F8 y correlacion con registros.
5. Descarga local de fixture, pausa/reanudacion, verificacion y publicacion.
6. Datos de 500, 1000, 10000 y volumen real del catalogo, sin numero fijo de juegos.

Los tests Node/Rust no certifican partidas ni ARM fisico. El flujo actual no
autoriza pruebas de navegador; las comprobaciones visuales pendientes se declaran.

## Comparativas retiradas

Se consolidaron el informe de benchmarks y la matriz headless. Sus tablas se
presentaban como oficiales sin enlazar muestras crudas ni ejecuciones reproducibles
verificadas en esta revision. No se usan como evidencia ni compromiso de 120 FPS,
bundles de tamano fijo o ausencia de fugas.

La decision vigente es SolidJS + Kobalte + CSS propio. Svelte/Bits UI, Vue/Reka,
React, Next y Astro eran alternativas estudiadas, no dependencias actuales.
TanStack Virtual, gilrs y rutas `shared/styles` citados entonces no describen la
implementacion actual.

Resultados fechados: [diagnosticos](../diagnostic-reports.md).
Referencias: [aceptacion](../architecture/appliance-validation.md) y
[registros](../architecture/diagnostic-logging.md).