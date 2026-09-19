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
  const runEpic = async (action: "authorize" | "sync" | "disconnect") => {
    if (!props.backend || epicBusy()) return;
    setEpicBusy(true);
    setEpicMessage("");
    try {
      if (action === "authorize") {
        await props.backend.startEpicAuthorization();
        setEpicMessage("Completa el inicio de sesión en la terminal y después sincroniza.");
      } else if (action === "sync") {
        const result = await props.backend.syncEpicLibrary();
        setEpicMessage(`${result.importedGames} juegos de Epic sincronizados.`);
      } else {
        await props.backend.disconnectEpic();
        setEpicMessage("Sesión de Epic cerrada.");
      }
      await refetch();
    } catch {
      setEpicMessage("La operación Epic no se completó. Revisa que el adaptador esté instalado y la sesión siga activa.");
      await refetch();
    } finally {
      setEpicBusy(false);
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
                    <div class="store-actions">
                      <ConsoleButton label={epicBusy() ? "ESPERA..." : "CONECTAR EPIC"} variant="action" onClick={() => void runEpic("authorize")} />
                      <ConsoleButton label="SINCRONIZAR" variant="action" onClick={() => void runEpic("sync")} />
                      <ConsoleButton label="CERRAR SESIÓN" variant="action" onClick={() => void runEpic("disconnect")} />
                    </div>
                    <Show when={epicMessage()}>
                      <p class="store-provider-note">{epicMessage()}</p>
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