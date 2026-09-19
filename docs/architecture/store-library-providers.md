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

La primera fase registra Steam, Epic y GOG, expone consultas IPC de proveedores/cuentas/entitlements y reserva la persistencia. Ajustes muestra cuentas, licencias y enlaces canonicos ya persistidos; los proveedores sin integracion aprobada aparecen como `authorization_required`. No implementa login, importacion de sesiones, scraping ni tokens. Cada proveedor debe incorporar despues un flujo autorizado propio sin compartir contrasenas con EmuBox.

## Investigacion de autenticacion

No existe un flujo de autenticacion o de biblioteca oficial y comun para estas tres tiendas. EmuBox no debe tratar una sesion de navegador, una cookie o un token observado en otro cliente como un contrato estable.

| Proveedor | Mecanismo oficial investigado | Limite para EmuBox | Decision |
| --- | --- | --- | --- |
| Steam | OpenID identifica una cuenta; `IPlayerService/GetOwnedGames` requiere `steamid` y una clave Web API de usuario. | OpenID no concede la biblioteca. Las claves de editor son para servidores seguros y no se distribuyen en clientes. | No implementar login ni sincronizacion de propiedad hasta disponer de una integracion autorizada y revisada. |
| Epic | EOS permite login y el Epic Games Launcher entrega exchange codes a launchers de productos integrados. | El flujo esta ligado a un producto de Epic autorizado; su refresh token no es un permiso generico para importar una biblioteca de consumidor. | No reutilizar el flujo EOS ni sus tokens para una biblioteca global sin acuerdo o documentacion explicita de Epic. |
| GOG | Galaxy SDK ofrece OpenID y tickets para juegos con credenciales y ambitos asignados en GOG Developer Portal. | Los tickets y claves privadas son por titulo; la API de integraciones de Galaxy es para extensiones ejecutadas por Galaxy, no para un cliente EmuBox independiente. | No implementar login ni biblioteca GOG hasta obtener un flujo de cliente autorizado por GOG. |

Las fuentes oficiales consultadas son la documentacion de Steamworks sobre [claves Web API](https://partner.steamgames.com/doc/webapi_overview/auth) y [GetOwnedGames](https://partner.steamgames.com/doc/webapi/IPlayerService), la guia de Epic sobre [third-party launchers](https://dev.epicgames.com/docs/epic-online-services/accounts-and-social/eos-epic-account-services/auth-interface/integrate-a-third-party-launcher-with-egs), y la documentacion de GOG sobre [OpenID](https://docs.gog.com/sdk-openid/) y [tickets cifrados](https://docs.gog.com/sdk-encrypted-tickets/).

## Requisitos previos para implementar un proveedor

Antes de anadir metodos como `login`, `refresh`, `logout` o `sync_library`, cada proveedor debe documentar un flujo aprobado que indique: cliente y permisos registrados, redireccion o intercambio de codigo, artefactos que persisten, forma de renovar y revocar la sesion, alcance de biblioteca permitido y procedimiento de eliminacion. El adaptador debe escribir secretos solo despues de crear su directorio `0700`, nunca registrarlos y mantenerlos fuera de SQLite. Hasta entonces, `StoreProvider` permanece como un registro de capacidad, no una interfaz de autenticacion ficticia.