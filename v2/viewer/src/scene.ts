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
