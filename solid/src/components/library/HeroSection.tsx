import { Component, Show } from 'solid-js';
import type { Game } from '@contracts/game.types';

interface HeroSectionProps {
  focusedGame: Game | null;
  onPlayGame: (game: Game) => void;
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
            <div class="hero-subtitle-placeholder">Explora tus títulos con el D-pad o stick analógico</div>
          </div>
        }
      >
        {(game) => (
          <div class="hero-active-content">
            <div class="hero-metadata-column">
              <div class="hero-tags-row">
                <span class="hero-platform-badge">{game().platformName.toUpperCase()}</span>
                <span class="hero-genre-pill">{game().genre}</span>
                <span class="hero-year-pill">{game().releaseYear}</span>
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
                  <span class="stat-val">{game().developer}</span>
                </div>
                <div class="hero-stat-item">
                  <span class="stat-label">Valoración</span>
                  <span class="stat-val rating-accent">★ {game().rating.toFixed(1)} / 5.0</span>
                </div>
                <div class="hero-stat-item">
                  <span class="stat-label">Tiempo de Juego</span>
                  <span class="stat-val">{game().playTimeMinutes} min</span>
                </div>
              </div>

              <div class="hero-actions-row">
                <button
                  class="hero-cta-btn primary-play-btn"
                  id="btn-hero-play"
                  onClick={() => props.onPlayGame(game())}
                >
                  <span class="btn-play-triangle">▶</span>
                  <span>JUGAR AHORA (A)</span>
                </button>

                <button
                  class="hero-cta-btn secondary-btn"
                  id="btn-hero-details"
                  onClick={() => props.onOpenDetails(game())}
                >
                  <span>MÁS DETALLES</span>
                </button>

                <button
                  class="hero-cta-btn icon-only-btn"
                  id="btn-hero-fav"
                  title="Alternar Favorito"
                  onClick={() => props.onToggleFavorite(game().id)}
                >
                  <span>{game().favorite ? '★' : '☆'}</span>
                </button>
              </div>
            </div>

            <div class="hero-art-showcase">
              <div class="hero-art-frame">
                <img src={game().coverImage} alt={game().title} class="hero-poster-img" />
                <div class="hero-art-sheen"></div>
              </div>
            </div>
          </div>
        )}
      </Show>
    </div>
  );
};
