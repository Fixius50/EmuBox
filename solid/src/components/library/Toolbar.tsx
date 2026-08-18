import { Component } from 'solid-js';

interface ToolbarProps {
  searchQuery: string;
  onSearchChange: (q: string) => void;
  favoritesOnly: boolean;
  onToggleFavorites: () => void;
  datasetLimit: number;
  onDatasetLimitChange: (limit: number) => void;
  totalGames: number;
}

export const Toolbar: Component<ToolbarProps> = (props) => {
  return (
    <div class="console-filter-hud">
      <div class="hud-left-controls">
        <div class="console-search-capsule" id="search-box">
          <kbd class="gamepad-key-badge btn-y">Y</kbd>
          <input
            type="text"
            class="console-search-field"
            placeholder="Filtrar por título, género o saga..."
            value={props.searchQuery}
            onInput={(e) => props.onSearchChange(e.currentTarget.value)}
            id="main-search-input"
          />
          {props.searchQuery && (
            <button class="clear-search-btn" onClick={() => props.onSearchChange('')}>✕</button>
          )}
        </div>

        <button
          class={`hud-toggle-pill ${props.favoritesOnly ? 'active' : ''}`}
          onClick={props.onToggleFavorites}
          id="btn-filter-favs"
        >
          <span class="hud-star-icon">★</span>
          <span>Sólo Favoritos</span>
        </button>
      </div>

      <div class="hud-right-telemetry">
        <div class="hud-dataset-selector">
          <span class="hud-label">Partición:</span>
          <select
            value={props.datasetLimit}
            onChange={(e) => props.onDatasetLimitChange(parseInt(e.currentTarget.value, 10))}
            class="hud-select-input"
            id="select-dataset-scale"
          >
            <option value="20">20 Títulos</option>
            <option value="100">100 Títulos</option>
            <option value="500">500 Títulos</option>
            <option value="1000">1.000 Títulos</option>
            <option value="5000">5.000 Títulos</option>
            <option value="10000">10.000 Títulos</option>
          </select>
        </div>

        <div class="hud-counter-pill">
          <span class="hud-counter-num">{props.totalGames.toLocaleString()}</span>
          <span class="hud-counter-label">ROMs DISPONIBLES</span>
        </div>
      </div>
    </div>
  );
};
