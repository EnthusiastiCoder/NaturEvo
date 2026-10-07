import { parseFight, type Fight } from './data';
import { FightScene } from './scene';
import { defenceTallyModel, hudModel } from './hud';
import { DEFAULT_ACCENT, applyTheme, nextTheme, setAccent, themeIconPath, type Theme } from './theme';

// Lucide icon paths (inline, 24x24 viewBox)
const ICON = {
  play: '<polygon points="6 3 20 12 6 21 6 3"/>',
  pause: '<rect x="14" y="4" width="4" height="16" rx="1"/><rect x="6" y="4" width="4" height="16" rx="1"/>',
  rotate: '<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/>',
  info: '<circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/>',
  home: '<path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
};
const svg = (p: string) => `<svg viewBox="0 0 24 24" aria-hidden="true">${p}</svg>`;
// silhouette part centre and half-size (13 parts) in a 40x72 box
const SIL: [number, number, number, number][] = [[20, 6, 5, 5], [20, 22, 9, 12], [20, 36, 8, 6], [10, 20, 3, 8], [30, 20, 3, 8], [8, 33, 3, 8], [32, 33, 3, 8], [15, 48, 4, 10], [25, 48, 4, 10], [15, 60, 3, 9], [25, 60, 3, 9], [13, 68, 4, 2], [27, 68, 4, 2]];
const PART_NAMES = ['Head', 'Torso', 'Pelvis', 'Upper arm L', 'Upper arm R', 'Forearm L (shield)', 'Forearm R (sword)', 'Thigh L', 'Thigh R', 'Shin L', 'Shin R', 'Foot L', 'Foot R'];
const SPEEDS = [0.25, 0.5, 1, 2];
