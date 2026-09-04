import { createSignal } from 'solid-js';
import type { AppSection, LibraryViewMode } from '@contracts/navigation.types';

export function createNavigationStore() {
  const [currentSection, setCurrentSection] = createSignal<AppSection>('library');
  const [libraryViewMode, setLibraryViewMode] = createSignal<LibraryViewMode>('games');
  const [wheelPlatformIndex, setWheelPlatformIndex] = createSignal<number>(0);
  const [focusedGameIndex, setFocusedGameIndex] = createSignal<number>(0);

  const navigateToSection = (section: AppSection) => {
    setCurrentSection(section);
  };

  const navigateToLibraryMode = (mode: LibraryViewMode) => {
    setLibraryViewMode(mode);
  };

  return {
    currentSection,
    setCurrentSection,
    libraryViewMode,
    setLibraryViewMode,
    wheelPlatformIndex,
    setWheelPlatformIndex,
    focusedGameIndex,
    setFocusedGameIndex,
    navigateToSection,
    navigateToLibraryMode,
    setSection: setCurrentSection
  };
}

export type NavigationStore = ReturnType<typeof createNavigationStore>;
