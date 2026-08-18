import { animate, stagger } from 'animejs';

/**
 * Coordinates staggered wave entrances and grid layout placement when opening shelves.
 */
export function animateStaggerShelf(targets: string | HTMLElement[] | NodeListOf<HTMLElement>): void {
  animate(targets, {
    opacity: [0, 1],
    translateY: ['1.75rem', '0rem'],
    scale: [0.92, 1],
    delay: stagger(24, { start: 40 }),
    duration: 380,
    ease: 'outBack(1.3)'
  });
}

/**
 * Orchestrates layout placement and positioning transitions of cards in the grid.
 */
export function animateShelfLayoutPlacement(targets: string | HTMLElement[] | NodeListOf<HTMLElement>): void {
  animate(targets, {
    opacity: [0.2, 1],
    scale: [0.94, 1],
    translateY: ['1rem', '0rem'],
    delay: stagger(18, { start: 20 }),
    duration: 320,
    ease: 'outCubic'
  });
}
