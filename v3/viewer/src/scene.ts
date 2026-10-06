import { Application, Container, Graphics } from 'pixi.js';
import type { Fight, FighterFrame, Frame, Seg } from './data';
import { cameraFor, lerpCamera, type Camera } from './camera';
import { debrisPose, drawRobot, PART_Z } from './robot-draw';
import { drawGround } from './ground';
import { hitstopTicks, shakeFor, sparkSpec, spawnParticles, type Particle } from './fx';
import { rendererOptions } from './renderer';

const hexNum = (css: string, fb: number) => { const m = /^#?([0-9a-f]{6})$/i.exec(css.trim()); return m ? parseInt(m[1], 16) : fb; };
const lerp = (a: number, b: number, k: number) => a + (b - a) * k;
const lerpSeg = (a: Seg, b: Seg, k: number): Seg => [lerp(a[0], b[0], k), lerp(a[1], b[1], k), lerp(a[2], b[2], k), lerp(a[3], b[3], k)];

export function sampleFrame(fight: Fight, tick: number): Frame {
  const fr = fight.frames, last = fr.length - 1;
  const step = fr.length > 1 ? fr[1].t - fr[0].t : 2;
  const s = Math.max(0, Math.min(last, tick / step)), i = Math.floor(s), k = s - i;
  if (i >= last || k === 0) return fr[Math.min(i, last)];
  const a = fr[i], b = fr[i + 1];
  const mix = (x: FighterFrame, y: FighterFrame): FighterFrame => ({
    ...x, fx: lerp(x.fx, y.fx, k), parts: x.parts.map((p, j) => lerpSeg(p, y.parts[j], k)),
    sword: lerpSeg(x.sword, y.sword, k), shield: lerpSeg(x.shield, y.shield, k),
    energy: lerp(x.energy, y.energy, k), posture: lerp(x.posture, y.posture, k),
  });
  return { t: tick, f: [mix(a.f[0], b.f[0]), mix(a.f[1], b.f[1])] };
}

export class FightScene {
  app = new Application();
  tick = 0; playing = true; speed = 1; showNumbers = false;
  private fight!: Fight;
  private world = new Container();
  private ground = new Graphics();
  private robots = [new Graphics(), new Graphics()];
  private debris = new Graphics();
  private fxg = new Graphics();
  private particles: Particle[] = [];
  private rings: { x: number; y: number; age: number; color: number; r: number }[] = [];
  private flash = [0, 0];
  private freeze = 0;
  private shake = 0;
  private fired = -1;
  private cam!: Camera;
  private detachedAt: (number | null)[][] = [new Array(13).fill(null), new Array(13).fill(null)];
  private host!: HTMLElement;
  private time = 0;
  onFrame: (fr: Frame, tick: number) => void = () => {};
  onEnd: () => void = () => {};

  async mount(host: HTMLElement, fight: Fight): Promise<void> {
    this.host = host; this.fight = fight;
    await this.app.init({ ...rendererOptions(window.devicePixelRatio || 1), resizeTo: host, background: 0x000000 });
    host.appendChild(this.app.canvas);
    this.app.stage.addChild(this.ground, this.world);
    this.world.addChild(this.debris, this.robots[0], this.robots[1], this.fxg);
    this.cam = cameraFor(fight.frames[0], this.app.screen.width, this.app.screen.height);
    this.app.ticker.add((tk) => this.update(tk.deltaMS / 1000));
  }

  seek(tick: number): void {
    this.tick = Math.max(0, tick); this.fired = Math.ceil(this.tick) - 1; this.particles = []; this.rings = [];
    this.detachedAt = [new Array(13).fill(null), new Array(13).fill(null)];
    for (const e of this.fight.events) if (e.kind === 'destroyed' && e.t <= this.tick) this.detachedAt[e.who][this.fight.parts.indexOf(e.part)] = e.t;
  }
  get lastTick(): number { return this.fight.frames[this.fight.frames.length - 1].t; }
  destroy(): void { this.app.destroy(true, { children: true }); }

  private css(v: string): string { return getComputedStyle(document.documentElement).getPropertyValue(v); }
