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

const syncLabel = (provider: StoreProviderInfo) => {
  if (provider.sync.status === "error") return "Error conservado";
  if (provider.sync.status === "syncing") return "Sincronizando";
  if (provider.sync.status === "ready") return "Preparado";
  return "Pendiente de autorización";
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
  const runEpic = async (action: "authorize" | "connect" | "sync" | "disconnect") => {
    if (!props.backend || epicBusy()) return;
    setEpicBusy(true);
    setEpicMessage("");
    try {
      if (action === "authorize") {
        await props.backend.startEpicAuthorization();
        setEpicMessage("Inicia sesión en Epic y pega aquí el código de autorización.");
      } else if (action === "connect") {
        await props.backend.completeEpicAuthorization(epicCode());
        setEpicCode("");
        setEpicMessage("Cuenta de Epic conectada. Ya puedes sincronizar la biblioteca.");
      } else if (action === "sync") {
        const result = await props.backend.syncEpicLibrary();
        setEpicMessage(`${result.importedGames} juegos de Epic sincronizados.`);
      } else {
        await props.backend.disconnectEpic();
        setEpicMessage("Sesión de Epic cerrada.");
      }
      await refetch();
    } catch {
      setEpicMessage("La operación Epic no se completó. Revisa el código de autorización e inténtalo de nuevo.");
      await refetch();
    } finally {
      setEpicBusy(false);
    }
  };
  const runGog = async (action: "authorize" | "connect" | "sync" | "disconnect") => {
    if (!props.backend || gogBusy()) return;
    setGogBusy(true);
    setGogMessage("");
    try {
      if (action === "authorize") {
        await props.backend.startGogAuthorization();
        setGogMessage("Inicia sesión en GOG y pega aquí el código de autorización.");
      } else if (action === "connect") {
        await props.backend.completeGogAuthorization(gogCode());
        setGogCode("");
        setGogMessage("Cuenta de GOG conectada. Ya puedes sincronizar la biblioteca.");
      } else if (action === "sync") {
        const result = await props.backend.syncGogLibrary();
        setGogMessage(`${result.importedGames} juegos de GOG sincronizados.`);
      } else {
        await props.backend.disconnectGog();
        setGogMessage("Sesión de GOG cerrada.");
      }
      await refetch();
    } catch {
      setGogMessage("La operación GOG no se completó. Revisa el código de autorización e inténtalo de nuevo.");
      await refetch();
    } finally {
      setGogBusy(false);
    }
  };
  const runSteam = async (action: "authorize" | "connect" | "sync" | "disconnect") => {
    if (!props.backend || steamBusy()) return;
    setSteamBusy(true);
    setSteamMessage("");
    try {
      if (action === "authorize") {
        await props.backend.startSteamAuthorization();
        setSteamMessage("Genera una clave Web API de Steam y completa los datos aquí.");
      } else if (action === "connect") {
        await props.backend.completeSteamAuthorization(steamId(), steamKey());
        setSteamKey("");
        setSteamMessage("Cuenta de Steam conectada. Ya puedes sincronizar la biblioteca.");
      } else if (action === "sync") {
        const result = await props.backend.syncSteamLibrary();
        setSteamMessage(`${result.importedGames} juegos de Steam sincronizados.`);
      } else {
        await props.backend.disconnectSteam();
        setSteamMessage("Credenciales de Steam eliminadas.");
      }
      await refetch();
    } catch {
      setSteamMessage("La operación Steam no se completó. Revisa los datos y la visibilidad de tu biblioteca.");
      await refetch();
    } finally {
      setSteamBusy(false);
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
                  <Show when={accountsFor(provider.id).length > 0} fallback={<p class="store-provider-note">La conexión requiere un flujo autorizado específico de {provider.name}.</p>}>
                    <div class="store-account-list">
                      <For each={accountsFor(provider.id)}>
                        {(account) => <span class="store-account-chip">{account.displayName}</span>}
                      </For>
                    </div>
                  </Show>
                  <Show when={provider.sync.errorMessage}>
                    <p class="store-provider-error">{provider.sync.errorMessage}</p>
                  </Show>
                  <Show when={provider.id === "epic"}>
                    <div class="store-credential-form">
                      <input aria-label="Código de autorización de Epic" autocomplete="off" placeholder="Código de autorización" type="password" value={epicCode()} onInput={(event) => setEpicCode(event.currentTarget.value)} />
                    </div>
                    <div class="store-actions">
                      <ConsoleButton label={epicBusy() ? "ESPERA..." : "ABRIR EPIC"} variant="action" onClick={() => void runEpic("authorize")} />
                      <ConsoleButton label="CONECTAR" variant="action" onClick={() => void runEpic("connect")} />
                      <ConsoleButton label="SINCRONIZAR" variant="action" onClick={() => void runEpic("sync")} />
                      <ConsoleButton label="CERRAR SESIÓN" variant="action" onClick={() => void runEpic("disconnect")} />
                    </div>
                    <Show when={epicMessage()}>
                      <p class="store-provider-note">{epicMessage()}</p>
                    </Show>
                  </Show>
                  <Show when={provider.id === "gog"}>
                    <div class="store-credential-form">
                      <input aria-label="Código de autorización de GOG" autocomplete="off" placeholder="Código de autorización" type="password" value={gogCode()} onInput={(event) => setGogCode(event.currentTarget.value)} />
                    </div>
                    <div class="store-actions">
                      <ConsoleButton label={gogBusy() ? "ESPERA..." : "ABRIR GOG"} variant="action" onClick={() => void runGog("authorize")} />
                      <ConsoleButton label="CONECTAR" variant="action" onClick={() => void runGog("connect")} />
                      <ConsoleButton label="SINCRONIZAR" variant="action" onClick={() => void runGog("sync")} />
                      <ConsoleButton label="CERRAR SESIÓN" variant="action" onClick={() => void runGog("disconnect")} />
                    </div>
                    <Show when={gogMessage()}>
                      <p class="store-provider-note">{gogMessage()}</p>
                    </Show>
                  </Show>
                  <Show when={provider.id === "steam"}>
                    <div class="store-credential-form store-steam-credential-form">
                      <input aria-label="SteamID64" autocomplete="off" inputmode="numeric" placeholder="SteamID64" value={steamId()} onInput={(event) => setSteamId(event.currentTarget.value)} />
                      <input aria-label="Clave Web API de Steam" autocomplete="off" placeholder="Clave Web API" type="password" value={steamKey()} onInput={(event) => setSteamKey(event.currentTarget.value)} />
                    </div>
                    <div class="store-actions">
                      <ConsoleButton label={steamBusy() ? "ESPERA..." : "ABRIR STEAM"} variant="action" onClick={() => void runSteam("authorize")} />
                      <ConsoleButton label="CONECTAR" variant="action" onClick={() => void runSteam("connect")} />
                      <ConsoleButton label="SINCRONIZAR" variant="action" onClick={() => void runSteam("sync")} />
                      <ConsoleButton label="CERRAR SESIÓN" variant="action" onClick={() => void runSteam("disconnect")} />
                    </div>
                    <Show when={steamMessage()}>
                      <p class="store-provider-note">{steamMessage()}</p>
                    </Show>
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