import { Component, Show } from 'solid-js';
import type { Game } from '@contracts/game.types';
import { ConsoleHardwareVisual } from '@components/common/ConsoleHardwareVisual';

interface HeroSectionProps {
  focusedGame: Game | null;
  variantCount?: number;
  playBlockReason?: string | null;
  downloadingIds: Set<string>;
  onPlayGame: (game: Game) => void;
  onDownloadGame: (game: Game) => void;
  onOpenDetails: (game: Game) => void;
  onToggleFavorite: (id: string) => void;
}

export const HeroSection: Component<HeroSectionProps> = (props) => {
  return (
    <div class="console-hero-banner">
      <Show
        when={props.focusedGame}
        fallback={
          <div class="hero-empty-state">
            <div class="hero-title-placeholder">BIBLIOTECA EMUBOX</div>
            <div class="hero-subtitle-placeholder">Sin juegos registrados</div>
          </div>
        }
      >
        {(game) => (
          <div class="hero-active-content">
            <div class="hero-metadata-column">
              <div class="hero-tags-row">
                <span class="hero-platform-badge">{game().platformName.toUpperCase()}</span>
                <Show when={(props.variantCount ?? 1) > 1}><span class="hero-genre-pill">{props.variantCount} paquetes</span></Show>
                <Show when={game().genre}><span class="hero-genre-pill">{game().genre}</span></Show>
                <Show when={game().releaseYear > 0}><span class="hero-year-pill">{game().releaseYear}</span></Show>
                {game().favorite && (
                  <span class="hero-favorite-badge">★ FAVORITO</span>
                )}
              </div>

              <h1 class="hero-game-title">{game().title}</h1>

              <p class="hero-synopsis-text">
                {game().description}
              </p>

              <div class="hero-stats-row">
                <div class="hero-stat-item">
                  <span class="stat-label">Desarrollador</span>
                  <span class="stat-val">{game().developer || 'No disponible'}</span>
                </div>
                <div class="hero-stat-item">
                  <span class="stat-label">Valoración</span>
                  <span class="stat-val rating-accent">{game().rating > 0 ? `★ ${game().rating.toFixed(1)} / 5.0` : 'No disponible'}</span>
                </div>
                <div class="hero-stat-item">
                  <span class="stat-label">Tiempo de Juego</span>
                  <span class="stat-val">{game().playTimeMinutes} min</span>
                </div>
              </div>

              <div class="hero-actions-row">
                <Show
                  when={game().installed}
                  fallback={
                    <button
                      class="hero-cta-btn primary-play-btn"
                      id="btn-hero-download"
                      disabled={props.downloadingIds.has(game().id)}
                      onClick={() => props.onDownloadGame(game())}
                    >
                      <span>{props.downloadingIds.has(game().id) ? 'DESCARGANDO...' : 'DESCARGAR'}</span>
                    </button>
                  }
                >
                  <button
                    class="hero-cta-btn primary-play-btn"
                    id="btn-hero-play"
                    disabled={Boolean(props.playBlockReason)}
                    title={props.playBlockReason ?? undefined}
                    onClick={() => props.onPlayGame(game())}
                  >
                    <span>JUGAR</span>
                  </button>
                </Show>

                <button
                  class="hero-cta-btn secondary-btn"
                  id="btn-hero-details"
                  onClick={() => props.onOpenDetails(game())}
                >
                  <span>FUENTES Y PAQUETES</span>
                </button>

                <button
                  class="hero-cta-btn secondary-btn"
                  id="btn-hero-fav"
                  title="Alternar Favorito"
                  onClick={() => props.onToggleFavorite(game().id)}
                >
                  <span>{game().favorite ? '★ EN FAVORITOS' : '☆ AÑADIR A FAVORITOS'}</span>
                </button>
              </div>
              <Show when={game().installed && props.playBlockReason}>
                <p role="status">{props.playBlockReason}</p>
              </Show>
            </div>

            <div class="hero-art-showcase">
              <div class="hero-art-frame">
                <Show when={game().coverImage} fallback={<ConsoleHardwareVisual platformId={game().platform} size="lg" />}>
                  <img src={game().coverImage} alt={game().title} class="hero-poster-img" />
                </Show>
                <div class="hero-art-sheen"></div>
              </div>
            </div>
          </div>
        )}
      </Show>
    </div>
  );
};
