import { Show } from "solid-js";
import { FlaskConical } from "lucide-solid";
import { XmbLibrary } from "@components/library/XmbLibrary";
import { SettingsView } from "@components/settings/SettingsView";
import { useDevelopmentPreview } from "@hooks/useDevelopmentPreview";
import "@styles/development-preview.css";

export default function DevelopmentPreview() {
  const preview = useDevelopmentPreview();
  return (
    <div class="development-preview">
      <aside class="development-preview-status" role="status">
        <FlaskConical size={16} aria-hidden="true" />
        <strong>DESARROLLO</strong>
        <Show
          when={preview.backend.isVmConnected}
          fallback={
            <span>Datos de ejemplo. Solo en memoria. Sin servicios nativos.</span>
          }
        >
          <span style={{ color: "#38bdf8", "font-weight": "600" }}>
            Conectado a MV ({preview.backend.vmAddress}) •{" "}
            {preview.library.catalogGames().length} juegos cargados
          </span>
        </Show>
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
            downloadJobs={preview.library.downloadJobs()}
            loading={preview.library.isLoading()}
            inputStatus={preview.inputStatus()}
            onOpenGame={(game) => {
              const isDownloading =
                preview.library.catalogDownloadingIds().has(game.id) ||
                preview.library
                  .downloadJobs()
                  .some(
                    (j) =>
                      j.gameId === game.id &&
                      (j.status === "downloading" || j.status === "queued"),
                  );
              if (isDownloading) {
                const job = preview.library
                  .downloadJobs()
                  .find((j) => j.gameId === game.id);
                if (job) void preview.library.controlDownload(job, "cancel");
                return;
              }
              if (preview.backend.isVmConnected) {
                preview.setNotice(
                  `Enviando orden de ejecución para "${game.title}" a la pantalla de la VM...`,
                );
                void preview.backend
                  .launchGame(game.id)
                  .then((res: any) => {
                    preview.setNotice(
                      res.message ||
                        `Juego "${game.title}" lanzado en la pantalla de la VM.`,
                    );
                  })
                  .catch((err: any) => {
                    preview.setNotice(`Error al lanzar en la VM: ${err.message}`);
                  });
              } else {
                void preview.library.downloadGame(game.id);
              }
            }}
            onCancelDownload={(id) => {
              const job = preview.library
                .downloadJobs()
                .find(
                  (j) =>
                    (j.gameId === id || j.id === id) &&
                    (j.status === "downloading" || j.status === "queued"),
                );
              if (job) {
                void preview.library.controlDownload(job, "cancel");
              }
            }}
            onUninstall={(id) => {
              void preview.library.uninstallGame(id);
            }}
            onOpenSettings={preview.openSettings}
            onFavorite={preview.toggleFavorite}
            onMove={() => {}}
            onControllerReady={preview.registerLibrary}
            onCloseSettings={preview.back}
            settingsPanel={
              <SettingsView
                onNavigate={preview.navigateSettings}
                onControllerReady={preview.registerSettings}
                settings={preview.system.settings()}
                emulators={preview.system.emulators()}
                storeBackend={preview.backend}
                libraryStore={preview.library}
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
      </div>
    </div>
  );
}

