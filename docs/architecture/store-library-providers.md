# Proveedores de bibliotecas de tiendas

## Limites de responsabilidad

`StoreProvider` representa una tienda, no un emulador ni un juego. Steam, Epic y GOG aportan identidad externa, propiedad, instalacion y metadatos de su propio catalogo. La biblioteca EmuBox conserva la presentacion mediante `canonical_games`.

Una cuenta de EmuBox nunca equivale a una cuenta de tienda. La capa no acepta ni persiste contrasenas. Las credenciales de una integracion autorizada son archivos privados del proveedor, no filas SQLite ni telemetria.

## Persistencia

`/var/lib/emubox/stores/<provider>/` esta reservado para la sesion privada del proveedor. SQLite contiene solo:

- `store_accounts`: identidad y estado de sincronizacion de la cuenta.
- `store_games`: identificador externo y metadatos estructurados de tienda.
- `store_entitlements`: propiedad e instalacion por cuenta.
- `store_game_links`: enlace explicito desde `(provider, external_game_id)` a `canonical_games`.

No hay matching textual automatico desde una tienda a una identidad canonica. Una misma identidad canonica puede tener enlaces de varias tiendas.

## Arranque y aislamiento

La UI usa el estado SQLite persistido antes de cualquier sincronizacion. Una futura sincronizacion de proveedor sera secundaria al arranque y conservara la biblioteca previa ante errores de red o autenticacion.

Los juegos ejecutados con Bubblewrap no reciben `/var/lib/emubox/stores`, la base SQLite de EmuBox ni sesiones de proveedores. Un proveedor puede requerir red y un flujo de login oficial; eso no cambia la politica de red por defecto de los juegos.

## Estado actual

La primera fase registra Steam, Epic y GOG, expone consultas IPC de proveedores/cuentas/entitlements y reserva la persistencia. No implementa login, importacion de sesiones, scraping ni tokens. Cada proveedor debe incorporar despues un flujo autorizado propio sin compartir contrasenas con EmuBox.