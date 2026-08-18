import { animate } from 'animejs';

/**
 * Focus Ring and Selection Glide Animations.
 * Coordinates smooth interpolation of the console focus frame tracking across cards.
 */
export function animateFocusGlide(
  frameEl: HTMLElement,
  targetEl: HTMLElement,
  containerEl?: HTMLElement,
  onComplete?: () => void
): void {
  if (!frameEl || !targetEl) return;

  const targetRect = targetEl.getBoundingClientRect();
  const parentRect = containerEl
    ? containerEl.getBoundingClientRect()
    : targetEl.parentElement?.getBoundingClientRect() || { left: 0, top: 0 };

  const x = targetRect.left - parentRect.left;
  const y = targetRect.top - parentRect.top;
  const w = targetRect.width;
  const h = targetRect.height;

  animate(frameEl, {
    translateX: x,
    translateY: y,
    width: w,
    height: h,
    opacity: 1,
    scale: [0.96, 1],
    duration: 180,
    ease: 'outCubic',
    onComplete: () => {
      onComplete?.();
    }
  });
}

/**
 * Card focus scale & glow emphasis.
 */
export function animateCardFocusEmphasis(cardEl: HTMLElement): void {
  if (!cardEl) return;
  animate(cardEl, {
    scale: [1, 1.05],
    duration: 200,
    ease: 'outBack(1.4)'
  });
}
