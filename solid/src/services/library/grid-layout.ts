export const DEFAULT_SHELF_COLUMNS = 5;

export function shelfColumns(viewportWidth: number): number {
  if (viewportWidth < 600) return 1;
  if (viewportWidth < 1100) return 3;
  return DEFAULT_SHELF_COLUMNS;
}