# Documentacion de EmuBox

Fecha de revision: 2026-09-16. El codigo y las pruebas determinan el comportamiento;
un informe historico o una propuesta no certifican una funcion disponible.

## Referencias vigentes

| Tema | Referencia |
| --- | --- |
| Instalacion y uso | [README principal](../README.md) |
| Estado y limites | [Estado actual](architecture/current-state.md) |
| Contratos Tauri y tipos | [Backend](architecture/backend-contracts.md) |
| Catalogo, identidades y versiones | [Catalogo](architecture/catalog-sources.md) |
| Fuentes, descarga y preparacion | [Descargas](architecture/download-providers.md) |
| Seguridad de ejecucion | [Sandbox](architecture/execution-sandbox.md) |
| Arranque y servicios | [Arranque appliance](architecture/console-appliance-boot-architecture.md) |
| Directorios del sistema | [Filesystem](architecture/filesystem-convention.md) |
| Diagnostico de pantalla negra | [VirtualBox](architecture/virtualbox-graphics.md) |
| Registros y filtros | [Logs](architecture/diagnostic-logging.md) |
| Arquitectura y estilo | [Guia de desarrollo](architecture/refactoring-and-architecture-guidelines.md) |
| Criterios de aceptacion | [Validacion](architecture/appliance-validation.md) |
| Requisitos, diseno y propuesta de distribucion | [Alcance](specification/requirements.md) |
| Metodologia y limites de benchmarks | [Mediciones](specification/benchmark.md) |

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