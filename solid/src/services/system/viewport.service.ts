import { createSignal, createMemo } from 'solid-js';

export interface ViewportState {
  width: number;
  height: number;
  aspectRatio: number;
  scaleFactor: number;
}

export class ViewportService {
  private static instance: ViewportService | null = null;
  private widthSignal = createSignal<number>(typeof window !== 'undefined' ? window.innerWidth : 1920);
  private heightSignal = createSignal<number>(typeof window !== 'undefined' ? window.innerHeight : 1080);
  private resizeObserver: ResizeObserver | null = null;
  private initialized = false;

  constructor() {
    if (ViewportService.instance) {
      return ViewportService.instance;
    }
    ViewportService.instance = this;
    this.init();
  }

  public static getInstance(): ViewportService {
    if (!ViewportService.instance) {
      ViewportService.instance = new ViewportService();
    }
    return ViewportService.instance;
  }

  public get width(): () => number {
    return this.widthSignal[0];
  }

  public get height(): () => number {
    return this.heightSignal[0];
  }

  public get aspectRatio(): () => number {
    return createMemo(() => {
      const h = this.heightSignal[0]();
      return h > 0 ? this.widthSignal[0]() / h : 16 / 9;
    });
  }

  public get scaleFactor(): () => number {
    return createMemo(() => {
      const w = this.widthSignal[0]();
      return Math.min(Math.max(w / 1920, 0.65), 2.5);
    });
  }

  private handleResize = () => {
    if (typeof window === 'undefined') return;

    const newWidth = window.innerWidth || document.documentElement.clientWidth || 1920;
    const newHeight = window.innerHeight || document.documentElement.clientHeight || 1080;

    const [currentWidth, setWidth] = this.widthSignal;
    const [currentHeight, setHeight] = this.heightSignal;

    if (currentWidth() !== newWidth) {
      setWidth(newWidth);
    }
    if (currentHeight() !== newHeight) {
      setHeight(newHeight);
    }

    // Sync CSS custom properties for hardware-accelerated layouts
    document.documentElement.style.setProperty('--viewport-width', `${newWidth}px`);
    document.documentElement.style.setProperty('--viewport-height', `${newHeight}px`);
    document.documentElement.style.setProperty('--viewport-scale', `${newWidth / 1920}`);
  };

  public init() {
    if (this.initialized || typeof window === 'undefined') return;
    this.initialized = true;

    // 1. Window resize listener
    window.addEventListener('resize', this.handleResize, { passive: true });

    // 2. Visual Viewport API listener (WebKitGTK & mobile/tiling window managers)
    if (window.visualViewport) {
      window.visualViewport.addEventListener('resize', this.handleResize, { passive: true });
    }

    // 3. ResizeObserver on documentElement (guarantees sub-pixel layout trigger)
    if (typeof ResizeObserver !== 'undefined') {
      this.resizeObserver = new ResizeObserver(() => {
        this.handleResize();
      });
      this.resizeObserver.observe(document.documentElement);
    }

    // Initial pass
    this.handleResize();
  }

  public destroy() {
    if (typeof window === 'undefined') return;
    window.removeEventListener('resize', this.handleResize);
    if (window.visualViewport) {
      window.visualViewport.removeEventListener('resize', this.handleResize);
    }
    if (this.resizeObserver) {
      this.resizeObserver.disconnect();
      this.resizeObserver = null;
    }
    this.initialized = false;
  }
}
