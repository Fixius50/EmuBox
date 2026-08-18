import { animate, stagger } from 'animejs';

/**
 * Staggered Entrance Animation for Emulator Core Deck Blades.
 */
export function animateEmulatorDeckEntrance(targets: string | HTMLElement[]): void {
  animate(targets, {
    translateY: [30, 0],
    opacity: [0, 1],
    scale: [0.96, 1],
    delay: stagger(55, { start: 40 }),
    duration: 350,
    ease: 'outCubic'
  });
}

/**
 * Interactive Focus Spring for the Active Emulator Core Card.
 */
export function animateEmulatorCardFocus(target: HTMLElement): void {
  if (!target) return;
  animate(target, {
    scale: [1, 1.025],
    duration: 220,
    ease: 'outCubic'
  });
}

/**
 * Cyber Emergence for the Emulator CRUD Management Modal.
 */
export function animateEmulatorModalEntrance(target: HTMLElement): void {
  if (!target) return;
  animate(target, {
    scale: [0.92, 1],
    opacity: [0, 1],
    translateY: [20, 0],
    duration: 260,
    ease: 'outBack(1.3)'
  });
}
