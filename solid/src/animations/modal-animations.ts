import { animate } from 'animejs';

/**
 * Coordinates console dialog and modal animations.
 * Provides an elastic blade entrance and a crisp exit.
 */
export function animateModalOpen(target: HTMLElement | string, onComplete?: () => void): void {
  animate(target, {
    opacity: [0, 1],
    scale: [0.92, 1],
    translateY: ['1rem', '0rem'],
    duration: 260,
    ease: 'outBack(1.5)',
    onComplete: () => {
      onComplete?.();
    }
  });
}

export function animateModalClose(target: HTMLElement | string, onComplete: () => void): void {
  animate(target, {
    opacity: [1, 0],
    scale: [1, 0.94],
    translateY: ['0rem', '0.75rem'],
    duration: 160,
    ease: 'inCubic',
    onComplete: () => {
      onComplete();
    }
  });
}
