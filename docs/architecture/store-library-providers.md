# Proveedores de bibliotecas de tiendas

## Limites de responsabilidad

`StoreProvider` representa una tienda, no un emulador ni un juego. Steam, Epic y GOG aportan identidad externa, propiedad, instalacion y metadatos de su propio catalogo. La biblioteca EmuBox conserva la presentacion mediante `canonical_games`.

Una cuenta de EmuBox nunca equivale a una cuenta de tienda. La capa no acepta ni persiste contrasenas. Las credenciales de una integracion autorizada son archivos privados del proveedor, no filas SQLite ni telemetria.

## Persistencia

`/var/lib/emubox/stores/<provider>/` esta reservado para la sesion privada del proveedor. El directorio raiz y cada subdirectorio de proveedor usan propietario `emubox` y modo `0700`. SQLite contiene solo:

- `store_accounts`: identidad y estado de sincronizacion de la cuenta.
- `store_games`: identificador externo y metadatos estructurados de tienda.
- `store_entitlements`: propiedad e instalacion por cuenta.
- `store_game_links`: enlace explicito desde `(provider, external_game_id)` a `canonical_games`.
- `store_provider_states`: estado local de sincronizacion, ultimo intento y error seguro de presentar.

No hay matching textual automatico desde una tienda a una identidad canonica. Una misma identidad canonica puede tener enlaces de varias tiendas.

## Arranque y aislamiento

La UI usa el estado SQLite persistido antes de cualquier sincronizacion. Una futura sincronizacion de proveedor sera secundaria al arranque y conservara la biblioteca previa ante errores de red o autenticacion.

Los juegos ejecutados con Bubblewrap no reciben `/var/lib/emubox/stores`, la base SQLite de EmuBox ni sesiones de proveedores. Un proveedor puede requerir red y un flujo de login oficial; eso no cambia la politica de red por defecto de los juegos.

## Estado actual

Steam, Epic y GOG exponen consultas IPC de proveedores/cuentas/entitlements y reservan persistencia privada. Ajustes muestra cuentas, licencias y enlaces canonicos ya persistidos. Epic incorpora un adaptador Legendary; Steam y GOG conservan el registro y la persistencia a la espera de sus adaptadores especificos.

### Epic mediante Legendary

`scripts/setup-store-adapters.sh epic` instala `legendary-gl` versionada en un venv bajo `/var/lib/emubox/stores/epic/runtime`. El login se abre con `footclient` y ejecuta `legendary auth` dentro del directorio privado. No existe una casilla de contrasena, codigo, token ni cookie en la UI o IPC de EmuBox.

Tras el login, EmuBox ejecuta `legendary status --json`, `legendary list --json` y `legendary list-installed --json`. Solo persiste identidad de cuenta, nombre de juego, identificador externo, estado de instalacion y marca temporal. El cierre usa `legendary auth --delete`. La opcion `legendary auth --import` queda excluida porque consume una sesion de Epic Games Launcher ajena y puede cerrarla.

### GOG mediante gogdl

`scripts/setup-store-adapters.sh gog` instala gogdl en `/var/lib/emubox/stores/gog/runtime`. La terminal de GOG abre el navegador, recibe el codigo por entrada estandar y llama al componente sin incluir el codigo como argumento o mensaje IPC. gogdl guarda y renueva `auth.json` bajo el mismo directorio privado.

La sincronizacion invoca `gogdl auth` solo dentro del backend para renovar la sesion y consulta la biblioteca paginada de Galaxy. El token se mantiene en memoria durante esa consulta; SQLite recibe solo cuenta, identificador externo, titulo disponible y entitlement. El cierre elimina `auth.json` y marca la cuenta como desconectada.

## Investigacion de autenticacion

No existe un flujo de autenticacion o de biblioteca comun para estas tres tiendas. EmuBox puede integrar componentes mantenidos que ya resuelven un flujo propio de cada tienda, siempre que se ejecuten aislados con un directorio de datos de EmuBox. No debe tratar una sesion de navegador, una cookie o un token observado en otro cliente como un contrato estable ni importarlo desde ese cliente.

| Proveedor | Patron investigado | Limite para EmuBox | Decision |
| --- | --- | --- | --- |
| Steam | Playnite usa el formulario web de Steam y conserva su propia sesion. `IPlayerService/GetOwnedGames` requiere `steamid` y una clave Web API de usuario. | El issue de Playnite citado solo describe un fallo de autenticacion; no documenta una API publica para capturar un token de biblioteca. | No extraer `webapi_token`, cookies ni configuracion de otro cliente. Una futura integracion debe tener un flujo propio verificable y no distribuir claves de editor. |
| Epic | Legendary ofrece `auth`, renovacion local y `list --json`; puede ejecutar un navegador/WebView o recibir un codigo introducido en su propio proceso. | `--import` reutiliza y revoca una sesion de Epic Games Launcher. | Se puede integrar Legendary como proceso separado, con configuracion bajo `stores/epic`, siempre que el login ocurra en su terminal/WebView y EmuBox no reciba ni registre los codigos o tokens. |
| GOG | gogdl esta pensado para ser llamado por otra aplicacion y mantiene su archivo de autenticacion en una ruta indicada. | El componente incorpora configuracion de cliente propia, cuya validez puede cambiar; su CLI no constituye una biblioteca completa documentada. | Se puede usar un gogdl mantenido como componente aislado, pero EmuBox no copiara ni fijara sus credenciales de cliente y necesitara un adaptador de biblioteca verificable antes de sincronizar entitlements. |

Las fuentes consultadas incluyen la documentacion de Steamworks sobre [claves Web API](https://partner.steamgames.com/doc/webapi_overview/auth) y [GetOwnedGames](https://partner.steamgames.com/doc/webapi/IPlayerService), [Legendary](https://github.com/legendary-gl/legendary), [gogdl](https://github.com/Heroic-Games-Launcher/heroic-gogdl), y los limites observados en [gogdl #72](https://github.com/Heroic-Games-Launcher/heroic-gogdl/issues/72).

## Requisitos previos para implementar un proveedor

Antes de anadir metodos como `login`, `refresh`, `logout` o `sync_library`, cada proveedor debe documentar el componente y el formato que consume: como abre la autorizacion interactiva, artefactos que persisten, forma de renovar y revocar la sesion, alcance de biblioteca permitido y procedimiento de eliminacion. El adaptador debe escribir secretos solo despues de crear su directorio `0700`, nunca registrarlos y mantenerlos fuera de SQLite. Las interacciones que requieren secreto deben ocurrir dentro del proceso del componente o de una terminal/WebView visible al usuario, nunca como argumentos, resultado IPC o telemetria de EmuBox.