import { animate } from 'animejs';

export { animateEmulatorModalEntrance } from './emulator-animations';

/**
 * Screen entrance animation for the Settings Container.
 */
export function animateSettingsEntrance(target: HTMLElement): void {
  if (!target) return;
  if (document.hidden || window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
    target.style.opacity = '1';
    target.style.transform = 'none';
    return;
  }
  animate(target, {
    opacity: [0, 1],
    translateY: [15, 0],
    duration: 320,
    ease: 'outCubic'
  });
}

/**
 * Tab Transition Fade and Sweep for Settings Content Pane.
 */
export function animateTabTransition(target: HTMLElement): void {
  if (!target) return;
  if (document.hidden || window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
    target.style.opacity = '1';
    target.style.transform = 'none';
    return;
  }
  animate(target, {
    opacity: [0, 1],
    translateX: [12, 0],
    duration: 240,
    ease: 'outCubic'
  });
}
