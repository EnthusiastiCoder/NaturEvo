import { Assets, Rectangle, Texture } from 'pixi.js';

// KayKit atlases copied into v2/assets (CC0, see CREDITS.md). Static URLs so Vite bundles them.
const URLS: Record<string, { png: string; json: string }> = {
  Skeleton_Warrior: { png: new URL('../../assets/Skeleton_Warrior.png', import.meta.url).href, json: new URL('../../assets/Skeleton_Warrior.json', import.meta.url).href },
  Skeleton_Minion: { png: new URL('../../assets/Skeleton_Minion.png', import.meta.url).href, json: new URL('../../assets/Skeleton_Minion.json', import.meta.url).href },
  Skeleton_Rogue: { png: new URL('../../assets/Skeleton_Rogue.png', import.meta.url).href, json: new URL('../../assets/Skeleton_Rogue.json', import.meta.url).href },
  Skeleton_Mage: { png: new URL('../../assets/Skeleton_Mage.png', import.meta.url).href, json: new URL('../../assets/Skeleton_Mage.json', import.meta.url).href },
};
export const MONSTER_MODELS = ['Skeleton_Minion', 'Skeleton_Rogue', 'Skeleton_Mage'];
export const HERO_MODEL = 'Skeleton_Warrior';
