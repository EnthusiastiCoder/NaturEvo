import { Application, Container, Graphics, Sprite, Texture } from 'pixi.js';
import { type Fight, type Run } from './data';
import { fitStage } from './layout';
import { hash2, paintGround, paintRock, paintTree } from './ground';
import { sampleFight, type FighterFrame } from './interp';
import { ANIM, Atlas, HERO_MODEL, MONSTER_MODELS, directionFromAngle, loadAtlas } from './sprites';

export const STAGE_W = 1536, STAGE_H = 1024;
const PITCH = 0.8;
const ATTACK_TICKS = { Windup: 0, Active: 6, Recovery: 10 } as Record<string, number>;
const ATTACK_TOTAL = 16;

export interface FightOpts { rays: boolean }
interface Rig { sprite: Sprite }

export class Scene {
  app!: Application;
  private atlases = new Map<string, Atlas>();
  private world = new Container();
  private layer = new Container();
  private overlay = new Graphics();
  private fx = new Graphics();
  private ground: Sprite | null = null;
  private rigs: Rig[] = [];
  private mode: 'fight' | 'map' | null = null;
  private ts = 32;
  private blocked: ((x: number, y: number) => boolean) | null = null;
  private trees = [0, 1, 2].map((v) => Texture.from(paintTree(v)));
  private rocks = [0, 1, 2].map((v) => Texture.from(paintRock(v)));
  private heroKey = HERO_MODEL;
  private monsterKey = MONSTER_MODELS[0];

  static async create(host: HTMLElement): Promise<Scene> {
    const s = new Scene();
    s.app = new Application();
    await s.app.init({ width: STAGE_W, height: STAGE_H, antialias: true, background: 0x0c120b, resolution: Math.min(2, window.devicePixelRatio || 1), autoDensity: true });
    host.appendChild(s.app.canvas);
    s.layer.sortableChildren = true;
    s.world.addChild(s.layer);
    s.app.stage.addChild(s.world, s.fx);
    await Promise.all([HERO_MODEL, ...MONSTER_MODELS].map(async (m) => s.atlases.set(m, await loadAtlas(m))));
    s.app.ticker.stop();
    return s;
  }

  /** Fit the fixed-size logical stage to the host element. */
  fit(hostW: number, hostH: number): void {
    const f = fitStage(hostW - 2, hostH - 2, STAGE_W, STAGE_H);
    this.app.canvas.style.width = `${f.w}px`;
    this.app.canvas.style.height = `${f.h}px`;
  }

  private reset(mode: 'fight' | 'map', ts: number): void {
    this.layer.removeChildren().forEach((c) => c.destroy({ children: true }));
    this.rigs = [];
    this.mode = mode; this.ts = ts;
    this.ground = null;
  }

  private addGround(canvas: HTMLCanvasElement): void {
    const g = new Sprite(Texture.from(canvas));
    g.zIndex = -1e9;
    this.layer.addChild(g);
    this.ground = g;
    this.overlay = new Graphics();
    this.overlay.zIndex = 1e8;
    this.layer.addChild(this.overlay);
  }

  private addProp(kind: 'T' | 'R', x: number, y: number, scale: number): void {
    const v = Math.floor(hash2(x, y, 5) * 3);
    const sp = new Sprite(kind === 'T' ? this.trees[v] : this.rocks[v]);
    if (kind === 'T') sp.anchor.set(0.5, 108 / 120); else sp.anchor.set(0.5, 40 / 48);
    const s = scale * (0.85 + hash2(x, y, 9) * 0.35);
    sp.scale.set(hash2(x, y, 2) > 0.5 ? -s : s, s);
    sp.position.set((x + 0.5) * this.ts, (y + 0.9) * this.ts * PITCH);
    sp.zIndex = sp.y;
    this.layer.addChild(sp);
  }

  private addFighters(): void {
    for (let i = 0; i < 2; i++) {
      const sprite = new Sprite();
      this.layer.addChild(sprite);
      this.rigs.push({ sprite });
    }
  }

  // ---------- fight ----------
  setFight(f: Fight): void {
    const ts = 32;
    this.reset('fight', ts);
    const { w, h, blocked } = f.arena;
    const isB = (x: number, y: number) => x < 0 || y < 0 || x >= w || y >= h || (blocked ? blocked[y]?.[x] === '#' : false);
    this.blocked = (x, y) => isB(Math.floor(x), Math.floor(y));
    this.addGround(paintGround({
      w, h, seed: f.seed,
      tile: (x, y) => (blocked ? (blocked[y]?.[x] === '#' ? 'T' : ',') : ((x + y) % 2 === 0 ? ',' : '.')),
      zone: (x, y) => (Math.hypot(x - w / 2, y - h / 2) > 15 ? 1 : 0),
    }, ts, PITCH));
    if (blocked) {
      for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) if (blocked[y][x] === '#') this.addProp(hash2(x, y) > 0.3 ? 'T' : 'R', x, y, 0.7);
    } else {
      // decorative treeline so the open arena still reads as a jungle clearing
      for (let x = 0; x < w; x += 2) { this.addProp('T', x, -1, 0.9); this.addProp('T', x + (x % 4), h, 0.9); }
      for (let y = 0; y < h; y += 3) { this.addProp('T', -1, y, 0.9); this.addProp('T', w, y, 0.9); }
    }
    this.world.scale.set(1); this.world.position.set((STAGE_W - w * ts) / 2, (STAGE_H - h * ts * PITCH) / 2);
    this.addFighters();
    this.monsterKey = MONSTER_MODELS[f.monsterIndex % MONSTER_MODELS.length];
  }
