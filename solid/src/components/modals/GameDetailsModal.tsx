import { Component, Show } from 'solid-js';
import { Dialog } from '@kobalte/core/dialog';
import type { Game } from '@contracts/game.types';

interface GameDetailsModalProps {
  game: Game | null;
  isOpen: boolean;
  onClose: () => void;
  onLaunch: (game: Game) => void;
  onToggleFavorite: (id: string) => void;
}

export const GameDetailsModal: Component<GameDetailsModalProps> = (props) => {
  return (
    <Dialog open={props.isOpen} onOpenChange={(open) => { if (!open) props.onClose(); }}>
      <Dialog.Portal>
        <Dialog.Overlay class="console-modal-backdrop" />
        <div class="console-modal-center-container">
          <Dialog.Content class="console-game-blade">
            <Show when={props.game}>
              {(g) => (
                <div class="blade-inner-layout">
                  <div class="blade-poster-wrapper">
                    <img src={g().coverImage} alt={g().title} class="blade-poster-img" />
                    <div class="blade-poster-glow"></div>
                  </div>

                  <div class="blade-info-wrapper">
                    <div>
                      <div class="blade-badge-row">
                        <span class="blade-platform-tag">{g().platformName.toUpperCase()}</span>
                        <span class="blade-year-tag">{g().releaseYear}</span>
                        <span class="blade-rating-tag">★ {g().rating.toFixed(1)} / 5.0</span>
                      </div>

                      <Dialog.Title class="blade-game-title">{g().title}</Dialog.Title>

                      <div class="blade-specs-grid">
                        <div class="spec-cell">
                          <span class="spec-header">Desarrollador</span>
                          <span class="spec-data">{g().developer}</span>
                        </div>
                        <div class="spec-cell">
                          <span class="spec-header">Distribuidor</span>
                          <span class="spec-data">{g().publisher}</span>
                        </div>
                        <div class="spec-cell">
                          <span class="spec-header">Género</span>
                          <span class="spec-data">{g().genre}</span>
                        </div>
                        <div class="spec-cell">
                          <span class="spec-header">Tiempo de Juego</span>
                          <span class="spec-data">{g().playTimeMinutes} min</span>
                        </div>
                      </div>

                      <Dialog.Description class="blade-synopsis-text">
                        {g().description}
                      </Dialog.Description>
                    </div>

                    <div class="blade-action-buttons">
                      <button
                        class="console-btn primary-glow-btn"
                        id="btn-play-game"
                        onClick={() => props.onLaunch(g())}
                      >
                        <span class="btn-icon">▶</span>
                        <span>JUGAR AHORA (A)</span>
                      </button>

                      <button
                        class="console-btn secondary-frosted-btn"
                        id="btn-fav-game"
                        onClick={() => props.onToggleFavorite(g().id)}
                      >
                        <span>{g().favorite ? '★ EN FAVORITOS' : '☆ MARCAR FAVORITO'}</span>
                      </button>

                      <button
                        class="console-btn ghost-btn"
                        id="btn-close-modal"
                        onClick={props.onClose}
                      >
                        <span>CERRAR (B)</span>
                      </button>
                    </div>
                  </div>
                </div>
              )}
            </Show>
          </Dialog.Content>
        </div>
      </Dialog.Portal>
    </Dialog>
  );
};
