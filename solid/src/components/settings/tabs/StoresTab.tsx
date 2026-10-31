import { Component, createResource, createSignal, For, Show } from "solid-js";
import { CloudOff, Link2, RefreshCw, ShieldCheck, Store } from "lucide-solid";
import { ConsoleButton } from "@components/common/ConsoleButton";
import type { IEmuBoxBackend } from "@contracts/backend.types";
import type { StoreAccount, StoreProviderInfo } from "@contracts/store.types";

interface StoresTabProps {
  backend?: IEmuBoxBackend;
  isRowFocused: (row: number) => boolean;
  onSelectContentArea?: () => void;
}

type StoreId = "epic" | "gog" | "steam";
type StoreAction = "authorize" | "connect" | "sync" | "disconnect";

const syncLabel = (provider: StoreProviderInfo) => {
  if (provider.sync.authorizationStatus === "required") return "Cuenta no conectada";
  if (provider.sync.status === "error") return "Error conservado";
  if (provider.sync.status === "syncing") return "Sincronizando";
  if (provider.entitlementCount === 0) return "Pendiente de sincronización";
  return "Biblioteca sincronizada";
};

export const StoresTab: Component<StoresTabProps> = (props) => {
  const [epicBusy, setEpicBusy] = createSignal(false);
  const [epicMessage, setEpicMessage] = createSignal("");
  const [epicCode, setEpicCode] = createSignal("");

  const [gogBusy, setGogBusy] = createSignal(false);
  const [gogMessage, setGogMessage] = createSignal("");
  const [gogCode, setGogCode] = createSignal("");

  const [steamBusy, setSteamBusy] = createSignal(false);
  const [steamMessage, setSteamMessage] = createSignal("");
  const [steamId, setSteamId] = createSignal("");
  const [steamKey, setSteamKey] = createSignal("");

  const [library, { refetch }] = createResource(async () => {
    if (!props.backend) throw new Error("Backend de tiendas no disponible");
    const [providers, accounts] = await Promise.all([
      props.backend.getStoreProviders(),
      props.backend.getStoreAccounts(),
    ]);
    return { providers, accounts };
  });

  const accountsFor = (provider: string): StoreAccount[] =>
    library()?.accounts.filter((account) => account.provider === provider) ?? [];

  const getBusy = (provider: string) => {
    if (provider === "epic") return epicBusy;
    if (provider === "gog") return gogBusy;
    return steamBusy;
  };

  const getMessage = (provider: string) => {
    if (provider === "epic") return epicMessage;
    if (provider === "gog") return gogMessage;
    return steamMessage;
  };

  const runStoreAction = async (provider: StoreId, action: StoreAction) => {
    if (!props.backend) return;
    const isBusy = getBusy(provider);
    if (isBusy()) return;
    const setBusy = provider === "epic" ? setEpicBusy : provider === "gog" ? setGogBusy : setSteamBusy;
    const setMessage = provider === "epic" ? setEpicMessage : provider === "gog" ? setGogMessage : setSteamMessage;

    setBusy(true);
    setMessage("");
    const providerName = provider === "epic" ? "Epic" : provider === "gog" ? "GOG" : "Steam";

    try {
      if (action === "authorize") {
        if (provider === "epic") await props.backend.startEpicAuthorization();
        else if (provider === "gog") await props.backend.startGogAuthorization();
        else await props.backend.startSteamAuthorization();
        setMessage(
          provider === "steam"
            ? "Genera una clave Web API de Steam y completa los datos aquí."
            : `Inicia sesión en ${providerName} y pega aquí el código de autorización.`,
        );
      } else if (action === "connect") {
        if (provider === "epic") {
          await props.backend.completeEpicAuthorization(epicCode());
          setEpicCode("");
        } else if (provider === "gog") {
          await props.backend.completeGogAuthorization(gogCode());
          setGogCode("");
        } else {
          await props.backend.completeSteamAuthorization(steamId(), steamKey());
          setSteamKey("");
        }
        setMessage(`Cuenta de ${providerName} conectada. Ya puedes sincronizar la biblioteca.`);
      } else if (action === "sync") {
        const result =
          provider === "epic"
            ? await props.backend.syncEpicLibrary()
            : provider === "gog"
              ? await props.backend.syncGogLibrary()
              : await props.backend.syncSteamLibrary();
        setMessage(`${result.importedGames} juegos de ${providerName} sincronizados.`);
      } else {
        if (provider === "epic") await props.backend.disconnectEpic();
        else if (provider === "gog") await props.backend.disconnectGog();
        else await props.backend.disconnectSteam();
        setMessage(provider === "steam" ? "Credenciales de Steam eliminadas." : `Sesión de ${providerName} cerrada.`);
      }
      await refetch();
    } catch {
      setMessage(
        provider === "steam"
          ? "La operación Steam no se completó. Revisa los datos y la visibilidad de tu biblioteca."
          : `La operación ${providerName} no se completó. Revisa el código de autorización e inténtalo de nuevo.`,
      );
      await refetch();
    } finally {
      setBusy(false);
    }
  };

  return (
    <div class="settings-tab-panel store-tab-panel">
      <div class="panel-header-block">
        <div class="panel-header-titles">
          <h3 class="panel-section-title">Bibliotecas de Tiendas</h3>
          <p class="panel-section-desc">La biblioteca local permanece disponible aunque una tienda no pueda sincronizarse.</p>
        </div>
        <div class="store-security-indicator"><ShieldCheck size={16} />Sesiones privadas</div>
      </div>

      <Show when={library.loading}>
        <div class="store-empty-state"><RefreshCw size={20} class="store-loading-icon" />Cargando estado de proveedores...</div>
      </Show>
      <Show when={library.error}>
        <div class="store-empty-state store-error-state"><CloudOff size={20} />No se pudo leer el estado local de tiendas.</div>
      </Show>
      <Show when={library()}>
        {(state) => (
          <div class="store-provider-list">
            <For each={state().providers}>
              {(provider, index) => (
                <article
                  class={`store-provider-card ${props.isRowFocused(index()) ? "focused" : ""}`}
                  onClick={() => props.onSelectContentArea?.()}
                >
                  <div class="store-provider-heading">
                    <span class="store-provider-icon"><Store size={19} /></span>
                    <div>
                      <h4>{provider.name}</h4>
                      <p>{syncLabel(provider)}</p>
                    </div>
                    <span class={`store-status-chip ${provider.sync.status}`}>{syncLabel(provider)}</span>
                  </div>
                  <div class="store-provider-metrics">
                    <span>{provider.accountCount} cuentas</span>
                    <span>{provider.entitlementCount} juegos con licencia</span>
                    <span><Link2 size={14} />{provider.linkedGameCount} enlazados</span>
                  </div>
                  <Show when={accountsFor(provider.id).length > 0} fallback={<p class="store-provider-note">Conecta tu cuenta para importar la biblioteca de {provider.name}.</p>}>
                    <div class="store-account-list">
                      <For each={accountsFor(provider.id)}>
                        {(account) => <span class="store-account-chip">{account.displayName}</span>}
                      </For>
                    </div>
                  </Show>
                  <Show when={provider.sync.errorMessage}>
                    <p class="store-provider-error">{provider.sync.errorMessage}</p>
                  </Show>

                  {/* Credential Inputs */}
                  <Show when={provider.id === "epic"}>
                    <div class="store-credential-form">
                      <input aria-label="Código de autorización de Epic" autocomplete="off" placeholder="Código de autorización" type="password" value={epicCode()} onInput={(event) => setEpicCode(event.currentTarget.value)} />
                    </div>
                  </Show>
                  <Show when={provider.id === "gog"}>
                    <div class="store-credential-form">
                      <input aria-label="Código de autorización de GOG" autocomplete="off" placeholder="Código de autorización" type="password" value={gogCode()} onInput={(event) => setGogCode(event.currentTarget.value)} />
                    </div>
                  </Show>
                  <Show when={provider.id === "steam"}>
                    <div class="store-credential-form store-steam-credential-form">
                      <input aria-label="SteamID64" autocomplete="off" inputmode="numeric" placeholder="SteamID64" value={steamId()} onInput={(event) => setSteamId(event.currentTarget.value)} />
                      <input aria-label="Clave Web API de Steam" autocomplete="off" placeholder="Clave Web API" type="password" value={steamKey()} onInput={(event) => setSteamKey(event.currentTarget.value)} />
                    </div>
                  </Show>

                  {/* Unified Action Buttons */}
                  <div class="store-actions">
                    <ConsoleButton label={getBusy(provider.id)() ? "ESPERA..." : `ABRIR ${provider.name.toUpperCase()}`} variant="action" onClick={() => void runStoreAction(provider.id as StoreId, "authorize")} />
                    <ConsoleButton label="CONECTAR" variant="action" onClick={() => void runStoreAction(provider.id as StoreId, "connect")} />
                    <ConsoleButton label="SINCRONIZAR" variant="action" onClick={() => void runStoreAction(provider.id as StoreId, "sync")} />
                    <ConsoleButton label="CERRAR SESIÓN" variant="action" onClick={() => void runStoreAction(provider.id as StoreId, "disconnect")} />
                  </div>
                  <Show when={getMessage(provider.id)()}>
                    <p class="store-provider-note">{getMessage(provider.id)()}</p>
                  </Show>
                </article>
              )}
            </For>
          </div>
        )}
      </Show>
    </div>
  );
};

export default StoresTab;
