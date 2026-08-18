import { animate, stagger } from 'animejs';

/**
 * Pulse animation for the Update Status Badge during active checking/downloading.
 */
export function animateUpdatePulse(target: HTMLElement): void {
  if (!target) return;
  animate(target, {
    scale: [1, 1.04, 1],
    opacity: [0.85, 1, 0.85],
    duration: 1200,
    ease: 'inOutQuad',
    loop: true
  });
}

/**
 * Smooth transition for download / extraction progress bar.
 */
export function animateUpdateProgressBar(target: HTMLElement, targetPercent: number): void {
  if (!target) return;
  animate(target, {
    width: `${targetPercent}%`,
    duration: 250,
    ease: 'outCubic'
  });
}

/**
 * Staggered entrance for update history / rollback cards.
 */
export function animateReleaseHistoryEntrance(targets: string | HTMLElement[]): void {
  animate(targets, {
    translateY: [20, 0],
    opacity: [0, 1],
    scale: [0.97, 1],
    delay: stagger(60, { start: 50 }),
    duration: 320,
    ease: 'outCubic'
  });
}
