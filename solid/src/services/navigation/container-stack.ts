import type { SpatialContainer } from '@contracts/navigation.types';

export class ContainerStack {
  private stack: SpatialContainer[] = [];
  private containers: Map<string, SpatialContainer> = new Map();

  constructor(defaultContainerId: string = 'root') {
    this.registerContainer({ id: defaultContainerId });
    this.push(defaultContainerId);
  }

  public registerContainer(container: SpatialContainer): void {
    this.containers.set(container.id, container);
  }

  public unregisterContainer(id: string): void {
    this.containers.delete(id);
    this.stack = this.stack.filter(c => c.id !== id);
  }

  public push(id: string, isTrap: boolean = false): void {
    let container = this.containers.get(id);
    if (!container) {
      container = { id, isTrap };
      this.registerContainer(container);
    } else {
      container.isTrap = isTrap;
    }
    this.stack.push(container);
  }

  public pop(): SpatialContainer | null {
    if (this.stack.length <= 1) return null;
    return this.stack.pop() || null;
  }

  public getActiveContainer(): SpatialContainer | null {
    if (this.stack.length === 0) return null;
    return this.stack[this.stack.length - 1];
  }

  public getActiveContainerId(): string {
    const active = this.getActiveContainer();
    return active ? active.id : 'root';
  }

  public setLastFocus(containerId: string, focusId: string): void {
    const container = this.containers.get(containerId);
    if (container) {
      container.lastFocusId = focusId;
    }
  }

  public getLastFocus(containerId: string): string | undefined {
    return this.containers.get(containerId)?.lastFocusId;
  }
}
