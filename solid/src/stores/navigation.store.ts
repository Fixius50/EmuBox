import { createSignal } from "solid-js";
import type { AppSection } from "@contracts/navigation.types";

export function createNavigationStore() {
  const [currentSection, setCurrentSection] =
    createSignal<AppSection>("library");
  return {
    currentSection,
    setCurrentSection,
  };
}

export type NavigationStore = ReturnType<typeof createNavigationStore>;
