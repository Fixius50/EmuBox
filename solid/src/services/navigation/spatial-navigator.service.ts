import type { NavDirection, SpatialNode, SpatialRect, FocusChangeListener } from '@contracts/navigation.types';
import { ContainerStack } from './container-stack';

export class SpatialNavigatorService {
  private elements: Map<string, SpatialNode> = new Map();
  private containerStack: ContainerStack;
  private currentFocusId: string | null = null;
  private focusListeners: Set<FocusChangeListener> = new Set();

  constructor() {
    this.containerStack = new ContainerStack('library');
  }

  public register(node: SpatialNode): void {
    this.elements.set(node.id, node);
  }

  public unregister(id: string): void {
    this.elements.delete(id);
    if (this.currentFocusId === id) {
      this.currentFocusId = null;
    }
  }

  public clear(): void {
    this.elements.clear();
    this.currentFocusId = null;
  }

  public getElements(): Map<string, SpatialNode> {
    return this.elements;
  }

  public getContainerStack(): ContainerStack {
    return this.containerStack;
  }

  public getActiveContainerId(): string {
    return this.containerStack.getActiveContainerId();
  }

  public getCurrentFocusId(): string | null {
    return this.currentFocusId;
  }

  public onFocusChange(listener: FocusChangeListener): () => void {
    this.focusListeners.add(listener);
    return () => this.focusListeners.delete(listener);
  }

  public pushContainer(containerId: string, isTrap: boolean = false): void {
    if (this.currentFocusId) {
      const activeId = this.containerStack.getActiveContainerId();
      this.containerStack.setLastFocus(activeId, this.currentFocusId);
    }
    this.containerStack.push(containerId, isTrap);
    this.focusFirstInContainer(containerId);
  }

  public popContainer(): void {
    const popped = this.containerStack.pop();
    if (popped) {
      const activeContainerId = this.containerStack.getActiveContainerId();
      const lastFocusId = this.containerStack.getLastFocus(activeContainerId);
      if (lastFocusId && this.elements.has(lastFocusId)) {
        this.setFocus(lastFocusId);
      } else {
        this.focusFirstInContainer(activeContainerId);
      }
    }
  }

  public setFocus(id: string): boolean {
    const target = this.elements.get(id);
    if (!target) return false;

    const prevId = this.currentFocusId;
    if (prevId && this.elements.has(prevId)) {
      const prevElement = this.elements.get(prevId)!.element;
      if (prevElement) {
        prevElement.classList.remove('focused', 'spatial-focus');
      }
    }

    this.currentFocusId = id;
    if (target.element) {
      target.element.classList.add('focused', 'spatial-focus');
      target.element.focus?.();
    }

    for (const listener of this.focusListeners) {
      listener(id, prevId);
    }
    return true;
  }

  public focusFirstInContainer(containerId: string): boolean {
    const candidates = Array.from(this.elements.values())
      .filter(e => e.containerId === containerId && !e.disabled);

    if (candidates.length === 0) return false;
    candidates.sort((a, b) => (b.priority || 0) - (a.priority || 0));
    return this.setFocus(candidates[0].id);
  }

  public navigate(direction: NavDirection): boolean {
    return this.move(direction);
  }

  public move(direction: NavDirection): boolean {
    const activeContainer = this.containerStack.getActiveContainerId();
    const candidates = Array.from(this.elements.values())
      .filter(e => e.containerId === activeContainer && !e.disabled);

    if (candidates.length === 0) return false;

    if (!this.currentFocusId || !this.elements.has(this.currentFocusId)) {
      return this.focusFirstInContainer(activeContainer);
    }

    const currentNode = this.elements.get(this.currentFocusId)!;
    const currentRect = this.getNodeRect(currentNode);

    let bestCandidate: SpatialNode | null = null;
    let minScore = Infinity;

    for (const candidate of candidates) {
      if (candidate.id === currentNode.id) continue;
      const candidateRect = this.getNodeRect(candidate);

      if (this.isInDirection(currentRect, candidateRect, direction)) {
        const score = this.calculateEuclideanScore(currentRect, candidateRect, direction);
        if (score < minScore) {
          minScore = score;
          bestCandidate = candidate;
        }
      }
    }

    if (bestCandidate) {
      return this.setFocus(bestCandidate.id);
    }

    return false;
  }

  private getNodeRect(node: SpatialNode): SpatialRect {
    if (node.rect) {
      const left = node.rect.left ?? (node.rect as any).x ?? 0;
      const top = node.rect.top ?? (node.rect as any).y ?? 0;
      const width = node.rect.width ?? 0;
      const height = node.rect.height ?? 0;
      return {
        left,
        top,
        right: left + width,
        bottom: top + height,
        width,
        height
      };
    }
    if (node.element) {
      const r = node.element.getBoundingClientRect();
      return {
        left: r.left,
        top: r.top,
        right: r.right,
        bottom: r.bottom,
        width: r.width,
        height: r.height
      };
    }
    return { left: 0, top: 0, right: 0, bottom: 0, width: 0, height: 0 };
  }

  private normalizeDir(dir: NavDirection): 'left' | 'right' | 'up' | 'down' {
    const d = dir.replace('NAV_', '').toLowerCase();
    if (d === 'left' || d === 'right' || d === 'up' || d === 'down') return d;
    return 'right';
  }

  private isInDirection(from: SpatialRect, to: SpatialRect, dir: NavDirection): boolean {
    const fromCenterX = from.left + from.width / 2;
    const fromCenterY = from.top + from.height / 2;
    const toCenterX = to.left + to.width / 2;
    const toCenterY = to.top + to.height / 2;

    const norm = this.normalizeDir(dir);

    switch (norm) {
      case 'left': return toCenterX < fromCenterX;
      case 'right': return toCenterX > fromCenterX;
      case 'up': return toCenterY < fromCenterY;
      case 'down': return toCenterY > fromCenterY;
    }
  }

  private calculateEuclideanScore(from: SpatialRect, to: SpatialRect, dir: NavDirection): number {
    const fromCenterX = from.left + from.width / 2;
    const fromCenterY = from.top + from.height / 2;
    const toCenterX = to.left + to.width / 2;
    const toCenterY = to.top + to.height / 2;

    const dx = toCenterX - fromCenterX;
    const dy = toCenterY - fromCenterY;
    const euclideanDist = Math.sqrt(dx * dx + dy * dy);

    const norm = this.normalizeDir(dir);
    let primaryDist = 0;
    let crossDist = 0;

    if (norm === 'left' || norm === 'right') {
      primaryDist = Math.abs(dx);
      crossDist = Math.abs(dy);
    } else {
      primaryDist = Math.abs(dy);
      crossDist = Math.abs(dx);
    }

    return euclideanDist + primaryDist * 0.75 + crossDist * 1.5;
  }
}

export { SpatialNavigatorService as EmuBoxSpatialNavigator };
