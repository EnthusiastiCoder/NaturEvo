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

interface AtlasFrame { x: number; y: number; w: number; h: number; px: number; py: number }
interface AtlasJson { animations: Record<string, { frameCount: number }>; frames: Record<string, AtlasFrame> }
export interface Pick { texture: Texture; ax: number; ay: number }

export class Atlas {
  private cache = new Map<string, Pick>();
  constructor(private base: Texture, private json: AtlasJson) {}
  count(anim: string): number { return this.json.animations[anim]?.frameCount ?? 1; }
  /** 8-direction frame; `dir` follows the bake (0 toward viewer, clockwise on screen). */
  get(anim: string, dir: number, index: number): Pick | null {
    const n = this.count(anim);
    const key = `${anim}/dir${((dir % 8) + 8) % 8}/${String(((index % n) + n) % n).padStart(4, '0')}`;
    const hit = this.cache.get(key);
    if (hit) return hit;
    const f = this.json.frames[key];
    if (!f) return null;
    const p: Pick = { texture: new Texture({ source: this.base.source, frame: new Rectangle(f.x, f.y, f.w, f.h) }), ax: f.px / f.w, ay: f.py / f.h };
    this.cache.set(key, p);
    return p;
  }
}

export async function loadAtlas(model: string): Promise<Atlas> {
  const u = URLS[model];
  const [tex, json] = await Promise.all([Assets.load<Texture>(u.png), fetch(u.json).then((r) => r.json() as Promise<AtlasJson>)]);
  return new Atlas(tex, json);
}
