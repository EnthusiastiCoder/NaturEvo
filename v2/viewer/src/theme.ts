export type Theme = 'light' | 'dark';
export const DEFAULT_ACCENT = '#2f9e44';
export const nextTheme = (t: Theme): Theme => (t === 'light' ? 'dark' : 'light');
export function applyTheme(root: { dataset: Record<string, string | undefined> }, t: Theme): void {
  root.dataset.theme = t;
}
export function setAccent(root: { style: { setProperty(k: string, v: string): void } }, hex: string): void {
  root.style.setProperty('--accent', hex);
}
