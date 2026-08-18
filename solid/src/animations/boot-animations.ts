import { createTimeline } from 'animejs';

/**
 * Console splash boot sequence.
 * Orchestrates a sleek neon pulse and emblem fade-in on application start.
 */
export function animateBootSequence(logoTarget: HTMLElement | string, telemetryTarget: HTMLElement | string, onComplete?: () => void): void {
  const tl = createTimeline({
    defaults: {
      ease: 'outExpo'
    },
    onComplete: () => {
      onComplete?.();
    }
  });

  tl.add(logoTarget, {
    opacity: [0, 1],
    scale: [0.85, 1],
    duration: 600
  }).add(telemetryTarget, {
    opacity: [0, 1],
    translateY: ['1.5rem', '0rem'],
    duration: 450
  }, '-=300');
}
