export type Theme = 'light' | 'dark';
export const DEFAULT_ACCENT = '#2f9e44';
export const nextTheme = (t: Theme): Theme => (t === 'light' ? 'dark' : 'light');
export function applyTheme(root: { dataset: Record<string, string | undefined> }, t: Theme): void {
  root.dataset.theme = t;
}
export function setAccent(root: { style: { setProperty(k: string, v: string): void } }, hex: string): void {
  root.style.setProperty('--accent', hex);
}

/** SVG inner markup for the toggle button: sun while dark (click for light), moon while light. */
export function themeIconPath(t: Theme): string {
  return t === 'dark'
    ? '<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/>'
    : '<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9z"/>';
}
