import { Show } from "solid-js";
import { FlaskConical } from "lucide-solid";
import { XmbLibrary } from "@components/library/XmbLibrary";
import { SettingsView } from "@components/settings/SettingsView";
import { DownloadSourceModal } from "@components/modals/DownloadSourceModal";
import { useDevelopmentPreview } from "@hooks/useDevelopmentPreview";
import "@styles/development-preview.css";

export default function DevelopmentPreview() {
  const preview = useDevelopmentPreview();
  return (
    <div class="development-preview">
      <aside class="development-preview-status" role="status">
        <FlaskConical size={16} aria-hidden="true" />
        <strong>DESARROLLO</strong>
        <span>Datos de ejemplo. Solo en memoria. Sin servicios nativos.</span>
        <Show when={preview.notice()}>
          <span>{preview.notice()}</span>
        </Show>
      </aside>
      <div class="emubox-xmb-root">
        <div class="xmb-library-layer">
          <XmbLibrary
            games={preview.library.catalogGames()}
            platforms={preview.system.platforms()}
            downloadingIds={preview.library.catalogDownloadingIds()}
            loading={preview.library.isLoading()}
            inputStatus={preview.inputStatus()}
            onOpenGame={(game) => {
              void preview.library.openSources(game);
            }}
            onOpenSettings={preview.openSettings}
            onFavorite={preview.toggleFavorite}
            onMove={() => {}}
            onControllerReady={preview.registerLibrary}
            onCloseSettings={preview.back}
            settingsActive={preview.section() === 'settings'}
            settingsPanel={
          <SettingsView
            onNavigate={preview.navigateSettings}
            onControllerReady={preview.registerSettings}
            settings={preview.system.settings()}
            emulators={preview.system.emulators()}
            storeBackend={preview.backend}
            activeTab={preview.activeTab()}
            focusArea={preview.focusArea()}
            focusedRowIndex={preview.row()}
            onSelectContentArea={preview.selectContent}
            onTabChange={preview.changeTab}
            onUpdateSettings={preview.updateSettings}
            onBack={preview.back}
          />
            }
          />
        </div>
        <DownloadSourceModal
          store={preview.library}
          onConfirm={preview.blockedAction}
          onPlay={preview.blockedAction}
          playBlockReason="Ejecucion deshabilitada en la previsualizacion"
          searchDisabled
          onControllerReady={preview.registerSources}
        />
      </div>
    </div>
  );
}
