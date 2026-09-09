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
