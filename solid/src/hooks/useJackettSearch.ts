import { createEffect, createSignal, on, onCleanup } from 'solid-js';
import type { JackettResult } from '@contracts/download.types';
import type { InputAction } from '@contracts/input.types';
import type { LibraryStore } from '@stores/library.store';

export function useJackettSearch(store: LibraryStore) {
  const gameId = () => store.sourceGame() ? store.releaseOptions()[store.releaseIndex()]?.catalogGameId || store.sourceGame()?.id : undefined;
  const [results, setResults] = createSignal<JackettResult[]>([]);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  const [open, setOpen] = createSignal(false);
  const [index, setIndex] = createSignal(0);
  let generation = 0;
  let resultList: HTMLDivElement | undefined;
  const close = () => { generation++; setOpen(false); setBusy(false); setResults([]); setError(''); };
  createEffect(on(gameId, close));
  createEffect(() => {
    const selected = index();
    if (open() && results().length) resultList?.querySelector(`[data-jackett-index="${selected}"]`)?.scrollIntoView({ block: 'nearest' });
  });
  onCleanup(() => { generation++; });
  const search = async () => {
    const id = gameId();
    if (!id || busy() || store.sourcesLoading()) return;
    const request = ++generation;
    setOpen(true); setBusy(true); setError(''); setResults([]); setIndex(0);
    try {
      const entries = await store.searchJackett(id);
      if (request === generation) setResults(entries);
    } catch (cause) {
      if (request === generation) setError(cause instanceof Error ? cause.message : typeof cause === 'string' ? cause : 'No se pudo consultar Jackett');
    } finally { if (request === generation) setBusy(false); }
  };
  const select = async () => {
    const id = gameId();
    const result = results()[index()];
    const game = store.sourceGame();
    if (!id || !result || !game || busy()) return;
    const request = ++generation;
    setBusy(true); setError('');
    try {
      const source = await store.selectJackettResult(id, result.id);
      if (request !== generation) return;
      close();
      await store.openSources({ ...game, id });
      const selected = store.sourceOptions().findIndex(entry => entry.id === source.id);
      if (selected >= 0) store.setSourceIndex(selected);
    } catch (cause) {
      if (request === generation) setError(cause instanceof Error ? cause.message : 'No se pudo registrar la fuente');
    } finally { if (request === generation) setBusy(false); }
  };
  const handleInput = (action: InputAction) => {
    if (action === 'BUTTON_START' && !open()) { void search(); return true; }
    if (!open()) return false;
    if (action === 'BUTTON_B') close();
    else if (!busy() && (action === 'NAV_DOWN' || action === 'NAV_UP')) setIndex(current => Math.max(0, Math.min(results().length - 1, current + (action === 'NAV_DOWN' ? 1 : -1))));
    else if (action === 'BUTTON_A') void select();
    return true;
  };
  return { results, busy, error, open, index, setIndex, close, search, select, handleInput,
    bindResults: (element: HTMLDivElement) => { resultList = element; } };
}