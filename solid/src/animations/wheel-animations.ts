import { animate } from 'animejs';

/**
 * Coordinates console card-passing and dial-turn animations on the wide 3D arch using Anime.js.
 */
export function animateConsoleCardSwitch(
  cardsTarget: HTMLElement[] | NodeListOf<HTMLElement> | string,
  _direction: number = 1,
  onComplete?: () => void
): void {
  // Pulse and glide the cards along the arch
  animate(cardsTarget, {
    scale: (el: HTMLElement) => {
      return el.classList.contains('active-center') ? [1.1, 1.25] : [0.85, 0.95];
    },
    duration: 360,
    ease: 'outBack(1.2)',
    onComplete: () => {
      onComplete?.();
    }
  });
}

/**
 * Spring entrance for the active console card badge contents (title, chips, action button).
 */
export function animateActiveCardBadgeEntrance(target: HTMLElement | string): void {
  animate(target, {
    opacity: [0.4, 1],
    scale: [0.94, 1],
    duration: 280,
    ease: 'outCubic'
  });
}
