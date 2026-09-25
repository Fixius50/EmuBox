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
- `store_provider_states`: estado separado de autorizacion y sincronizacion, ultimo intento y error seguro de presentar.

No hay matching textual automatico desde una tienda a una identidad canonica. Una misma identidad canonica puede tener enlaces de varias tiendas.

## Arranque y aislamiento

La UI usa el estado SQLite persistido antes de cualquier sincronizacion. Los adaptadores ofrecen sincronizacion explicita desde Ajustes; la autorizacion de tiendas no es un requisito del arranque.

Los juegos ejecutados con Bubblewrap no reciben `/var/lib/emubox/stores`, la base SQLite de EmuBox ni sesiones de proveedores. Un proveedor puede requerir red y un flujo de login oficial; eso no cambia la politica de red por defecto de los juegos.

## Estado actual

Steam, Epic y GOG exponen consultas IPC de proveedores/cuentas/entitlements y reservan persistencia privada. Ajustes muestra cuentas, licencias y enlaces canonicos ya persistidos. La autorizacion (`required` o `connected`) es independiente de la sincronizacion (`authorization_required`, `syncing`, `ready` o `error`): una cuenta conectada puede seguir pendiente de su primera importacion.

Los flujos de conexion, sincronizacion y desconexion estan implementados por proveedor. Su aceptacion con cuentas reales y los adaptadores instalados debe verificarse en el equipo objetivo; las pruebas locales no la certifican.

Los botones Abrir Epic, GOG y Steam de Ajustes abren ventanas WebView nativas con
origen HTTPS restringido a la tienda correspondiente, sin capacidades IPC y sin
popups externos. GOG sigue obteniendo su URL de autorizacion desde el runtime
local de gogdl, ahora sin invocar un navegador externo. Redirecciones a servicios
de identidad de terceros fuera de los dominios permitidos no se admiten; no se
han probado sesiones reales ni completado CAPTCHA/MFA automaticamente.

### Epic mediante Legendary

`scripts/setup-store-adapters.sh epic` instala `legendary-gl` versionada en un venv bajo `/var/lib/emubox/stores/epic/runtime`. EmuBox abre el inicio oficial de Epic y recibe una vez el codigo de autorizacion a traves del formulario de Tiendas. El codigo no se registra, no se incluye en argumentos de proceso y no se persiste en SQLite.

Tras el login, EmuBox ejecuta `legendary status --json`, `legendary list --json` y `legendary list-installed --json`. Solo persiste identidad de cuenta, nombre de juego, identificador externo, estado de instalacion y marca temporal. El cierre usa `legendary auth --delete`. La opcion `legendary auth --import` queda excluida porque consume una sesion de Epic Games Launcher ajena y puede cerrarla.

### GOG mediante gogdl

`scripts/setup-store-adapters.sh gog` instala la etiqueta `v1.3.0` de gogdl con sus submodulos en `/var/lib/emubox/stores/gog/runtime`. La distribucion ZIP no sirve para este componente porque omite su extension xdelta3. EmuBox abre el inicio oficial de GOG y recibe una vez el codigo por el formulario de Tiendas; el codigo no se registra, no se incluye en argumentos de proceso y no se persiste en SQLite. gogdl guarda y renueva `auth.json` bajo el mismo directorio privado.

La sincronizacion invoca `gogdl auth` solo dentro del backend para renovar la sesion y consulta la biblioteca paginada de Galaxy. El token se mantiene en memoria durante esa consulta; SQLite recibe solo cuenta, identificador externo, titulo disponible y entitlement. El cierre elimina `auth.json` y marca la cuenta como desconectada.

### Steam mediante clave Web API propia

Steam no usa el WebView de EmuBox para extraer cookies, `webapi_token` u otros secretos de pagina. EmuBox abre la pagina oficial de claves Web API; el usuario completa Steam Guard, MFA o CAPTCHA directamente en Steam e introduce despues su SteamID64 y la clave generada en Tiendas. EmuBox guarda el par solo en `stores/steam/credentials.json` con modo `0600`.

La sincronizacion usa `GetPlayerSummaries` y `GetOwnedGames` sobre HTTPS. La clave entra una vez por IPC local desde el formulario protegido, pero nunca aparece en telemetria, logs, argumentos de proceso, resultados IPC ni SQLite. El cierre elimina el archivo privado. La disponibilidad de resultados depende de la clave y de la visibilidad que Steam permita para la biblioteca de esa cuenta.

## Investigacion de autenticacion

No existe un flujo de autenticacion o de biblioteca comun para estas tres tiendas. EmuBox puede integrar componentes mantenidos que ya resuelven un flujo propio de cada tienda, siempre que se ejecuten aislados con un directorio de datos de EmuBox. No debe tratar una sesion de navegador, una cookie o un token observado en otro cliente como un contrato estable ni importarlo desde ese cliente.

| Proveedor | Patron investigado | Limite para EmuBox | Decision |
| --- | --- | --- | --- |
| Steam | Playnite usa el formulario web de Steam y conserva su propia sesion. `IPlayerService/GetOwnedGames` requiere `steamid` y una clave Web API de usuario. | Una sesion de otro cliente no es una API publica de biblioteca. | Se usa SteamID64 y una clave Web API propia introducidos por IPC local y almacenados de forma privada. No se extraen `webapi_token`, cookies ni configuracion de otro cliente. |
| Epic | Legendary ofrece `auth`, renovacion local y `list --json`; puede recibir un codigo de autorizacion. | `--import` reutiliza y revoca una sesion de Epic Games Launcher. | Se integra Legendary como proceso separado bajo `stores/epic`. El codigo se recibe una vez por IPC local desde Tiendas, sin registrarlo ni pasarlo en argumentos de proceso; no se importa la sesion de otro cliente. |
| GOG | gogdl esta pensado para ser llamado por otra aplicacion y mantiene su archivo de autenticacion en una ruta indicada. | El componente incorpora configuracion de cliente propia, cuya validez puede cambiar; su CLI no constituye una biblioteca completa documentada. | Se usa gogdl con autenticacion privada bajo `stores/gog` y un adaptador que consulta la biblioteca de Galaxy. EmuBox no copia ni fija las credenciales de cliente del componente. |

Las fuentes consultadas incluyen la documentacion de Steamworks sobre [claves Web API](https://partner.steamgames.com/doc/webapi_overview/auth) y [GetOwnedGames](https://partner.steamgames.com/doc/webapi/IPlayerService), [Legendary](https://github.com/legendary-gl/legendary), [gogdl](https://github.com/Heroic-Games-Launcher/heroic-gogdl), y los limites observados en [gogdl #72](https://github.com/Heroic-Games-Launcher/heroic-gogdl/issues/72).

## Requisitos previos para implementar un proveedor

Antes de anadir metodos como `login`, `refresh`, `logout` o `sync_library`, cada proveedor debe documentar el componente y el formato que consume: como abre la autorizacion interactiva, artefactos que persisten, forma de renovar y revocar la sesion, alcance de biblioteca permitido y procedimiento de eliminacion. El adaptador debe escribir secretos solo despues de crear su directorio `0700`, nunca registrarlos y mantenerlos fuera de SQLite. Las interacciones que requieren secreto pasan una sola vez de la UI local al backend mediante IPC local; no se incluyen en argumentos de proceso, resultados IPC, telemetria, logs, SQLite ni archivos temporales.