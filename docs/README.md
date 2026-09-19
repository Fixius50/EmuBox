# Documentacion de EmuBox

Fecha de revision: 2026-09-16. El codigo y las pruebas determinan el comportamiento;
un informe historico o una propuesta no certifican una funcion disponible.

## Mapa de lectura

| Necesidad | Entrada canonica | Complementos |
| --- | --- | --- |
| Instalar, ejecutar o validar el proyecto | [README principal](../README.md) | [Validacion de appliance](architecture/appliance-validation.md), [mediciones](specification/benchmark.md) |
| Conocer que funciona hoy y que sigue pendiente | [Estado actual](architecture/current-state.md) | [requisitos y alcance](specification/requirements.md), [informes historicos](diagnostic-reports.md) |
| Cambiar frontend, IPC o servicios nativos | [Guia de desarrollo](architecture/refactoring-and-architecture-guidelines.md) | [contratos backend](architecture/backend-contracts.md), [filesystem](architecture/filesystem-convention.md) |
| Tocar catalogo, biblioteca o descargas | [Catalogo](architecture/catalog-sources.md) | [proveedores de descarga](architecture/download-providers.md), [sandbox](architecture/execution-sandbox.md) |
| Integrar bibliotecas de Steam, Epic o GOG | [Proveedores de tiendas](architecture/store-library-providers.md) | [Catalogo](architecture/catalog-sources.md), [filesystem](architecture/filesystem-convention.md) |
| Evaluar online o multijugador futuro | [Online y multijugador](specification/online-multiplayer.md) | [requisitos y alcance](specification/requirements.md), [contratos backend](architecture/backend-contracts.md) |
| Diagnosticar arranque, graficos o registros | [Arranque appliance](architecture/console-appliance-boot-architecture.md) | [VirtualBox](architecture/virtualbox-graphics.md), [logs](architecture/diagnostic-logging.md) |

## Estructura canonica

| Carpeta | Responsabilidad |
| --- | --- |
| `architecture/` | Estado tecnico vigente, contratos, limites de runtime, servicios nativos, seguridad y diagnostico operativo. |
| `specification/` | Requisitos de producto, alcance funcional, metodologia de medicion y criterios que no dependen de una implementacion concreta. |
| `diagnostic-reports.md` | Cronologia fechada de pruebas y hallazgos. Es evidencia historica, no fuente normativa. |

Antes de crear un documento nuevo, revisar si encaja en una de estas entradas. Si
describe comportamiento vigente, debe enlazarse desde `architecture/current-state.md`
o desde la guia tecnica responsable. Si describe una aspiracion, requisito o metodo
de medida, pertenece a `specification/`. Si conserva una investigacion fechada,
pertenece a `diagnostic-reports.md`.

## Evidencia historica

[Informes de diagnostico](diagnostic-reports.md) conserva observaciones fechadas.
No constituye una lista de fallos actuales ni una promesa de compatibilidad.
El negro intermitente sigue pendiente de confirmacion visual; builds y tests
correctos no certifican que se haya corregido.

## Consolidacion

Se retiraron los duplicados de IPC y PS3, el informe general para IA, la propuesta
OS separada y las comparativas/diseno preliminares. Sus responsabilidades quedan
en backend, descargas, estado actual, requisitos y metodologia respectivamente.
Las cifras de benchmark sin evidencia enlazada no se presentan como resultados
actuales. Las reglas del agente y notas de capacidades Tauri se mantienen en sus
ubicaciones tecnicas. `npm run arch:check` comprueba los enlaces locales Markdown.