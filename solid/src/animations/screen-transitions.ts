import { animate, stagger } from 'animejs';

/**
 * Screen and section transition animations.
 * Coordinates smooth fades, dome arch rotations, and 2-second water emergence camera zooms.
 */
export function animateScreenEnter(target: HTMLElement | string, onComplete?: () => void): void {
  animate(target, {
    opacity: [0, 1],
    translateY: ['1.25rem', '0rem'],
    duration: 320,
    ease: 'outCubic',
    onComplete: () => {
      onComplete?.();
    }
  });
}

export function animateScreenExit(target: HTMLElement | string, onComplete?: () => void): void {
  animate(target, {
    opacity: [1, 0],
    translateY: ['0rem', '-0.75rem'],
    duration: 180,
    ease: 'inCubic',
    onComplete: () => {
      onComplete?.();
    }
  });
}

/**
 * 2-Second Cinematic Camera Journey:
 * Lifts the top dome arch upwards while driving the water-emerged library forward into interactive foreground.
 */
export function animateCameraZoomIntoLibrary(
  waterHorizonTarget: HTMLElement | string,
  domeArchTarget: HTMLElement | string,
  onComplete?: () => void
): void {
  // 1. Lift and dissolve top dome arch
  animate(domeArchTarget, {
    opacity: [1, 0],
    translateY: ['0rem', '-12rem'],
    scale: [1, 1.15],
    duration: 650,
    ease: 'inOutCubic'
  });

  // 2. 2-Second cinematic camera journey: library rises and pushes forward
  animate(waterHorizonTarget, {
    scale: [0.82, 1.0],
    translateZ: ['-20rem', '0rem'],
    rotateX: ['18deg', '0deg'],
    opacity: [0.8, 1.0],
    duration: 2000,
    ease: 'outExpo',
    onComplete: () => {
      onComplete?.();
    }
  });
}

export function animateCameraZoomOutToWheel(
  gamesTarget: HTMLElement | string,
  onComplete?: () => void
): void {
  animate(gamesTarget, {
    opacity: [1, 0],
    scale: [1, 0.94],
    duration: 350,
    ease: 'inCubic',
    onComplete: () => {
      onComplete?.();
    }
  });
}

export function animateGamesToWheel(
  gamesTarget: HTMLElement | string,
  onComplete?: () => void
): void {
  animateCameraZoomOutToWheel(gamesTarget, onComplete);
}

/**
 * Water emergence wave animation for the cards when platform changes
 */
export function animateWaterEmergence(cardsTarget: HTMLElement | string | NodeListOf<HTMLElement>): void {
  animate(cardsTarget, {
    translateY: ['2.5rem', '0rem'],
    opacity: [0.2, 1],
    delay: stagger(35, { start: 20 }),
    duration: 450,
    ease: 'outCubic'
  });
}

export function animateBackdropMosaic(target: HTMLElement | string): void {
  animate(target, {
    opacity: [0.4, 0.85],
    duration: 350,
    ease: 'outCubic'
  });
}
