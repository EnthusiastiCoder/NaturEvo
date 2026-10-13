import { Application, Container, Graphics, Text } from 'pixi.js';
import type { Fight, FighterFrame, Frame, Seg } from './data';
import { cameraFor, lerpCamera, type Camera } from './camera';
import { debrisPose, drawRobot, PART_Z, shieldPlate } from './robot-draw';
import { drawGround } from './ground';
import { ageParticles, hitstopTicks, recentEvents, shakeFor, sparkLayers, sparkSpec, spawnParticles, type Particle } from './fx';
import { blockPoint, floatNumber, hitFx, hitstopPlan, partFlashAt, recoilCm, shakeOffset } from './impact';
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

const PART_SHORT: Record<string, string> = { UpperArmL: 'U.arm', UpperArmR: 'U.arm', ForearmL: 'Shield arm', ForearmR: 'Sword arm' };
const KO_TINT = 0xff3b30;
type Ev = Fight['events'][number];

export class FightScene {
  app = new Application();
  tick = 0; playing = true; speed = 1; showNumbers = false;
  /** Reduced motion: no camera shake, hit-stop, recoil or screen flash (sparks and numbers stay). */
  reduceMotion = typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;
  private fight!: Fight;
  private world = new Container();
  private ground = new Graphics();
  private robots = [new Graphics(), new Graphics()];
  private debris = new Graphics();
  private fxg = new Graphics();
  private screenFx = new Graphics();
  private labels: Text[] = [];
  private banner!: Text;
  private bannerSub!: Text;
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
    // scrubbing onto a hit/clash must still show its sparks: replay recent events, pre-aged
    for (const { e, age } of recentEvents(this.fight.events, this.tick)) {
      const spec = sparkSpec(e);
      if (spec) this.particles.push(...ageParticles(spawnParticles(spec, e.x, e.y, e.t * 31 + 7), age));
      if (e.kind === 'hit') {
        this.flash[e.victim] = Math.max(this.flash[e.victim], Math.max(0, 1 - age * 5));
        if (age * 3 < 1) this.rings.push({ x: e.x, y: e.y, age: age * 3, color: spec?.color ?? 0xffa83a, r: 28 });
      }
    }
  }
  get lastTick(): number { return this.fight.frames[this.fight.frames.length - 1].t; }
  destroy(): void { this.app.destroy(true, { children: true }); }

  private css(v: string): string { return getComputedStyle(document.documentElement).getPropertyValue(v); }

  private update(dt: number): void {
    this.time += dt;
    const ticksPerSec = 60;
    if (this.playing) {
      if (this.freeze > 0) this.freeze = Math.max(0, this.freeze - dt * ticksPerSec);
      else {
        const prev = this.tick;
        this.tick = Math.min(this.lastTick, this.tick + dt * ticksPerSec * this.speed);
        for (const e of this.fight.events) {
          if (e.t > this.fired && e.t <= this.tick) this.fire(e);
        }
        this.fired = Math.floor(this.tick);
        if (prev < this.lastTick && this.tick >= this.lastTick) this.onEnd();
      }
    }
    this.render(dt);
  }

  private fire(e: Fight['events'][number]): void {
    const spec = sparkSpec(e);
    if (spec) {
      this.particles.push(...spawnParticles(spec, e.x, e.y, e.t * 31 + 7));
      if (spec.kind === 'ring') this.rings.push({ x: e.x, y: e.y, age: 0, color: spec.color, r: 30 });
      if (e.kind === 'clash' && e.type !== 'clash') this.rings.push({ x: e.x, y: e.y, age: 0, color: 0xfff2b0, r: 60 });
    }
    this.shake = Math.max(this.shake, shakeFor(e));
    if (e.kind === 'hit') { this.flash[e.victim] = 1; this.rings.push({ x: e.x, y: e.y, age: 0, color: 0xffa83a, r: 28 }); }
    if (e.kind === 'stagger') this.rings.push({ ...this.torso(this.sampleNow(), e.who), age: 0, color: 0xffe14a, r: 70 });
    if (e.kind === 'destroyed') this.detachedAt[e.who][this.fight.parts.indexOf(e.part)] = e.t;
    const hs = hitstopTicks(this.fight.events, e.t);
    if (e.kind === 'hit' && hs > 0) this.freeze = Math.max(this.freeze, hs);
  }

  private torso(fr: Frame, who: number): { x: number; y: number } {
    const p = fr.f[who].parts[1];
    return { x: (p[0] + p[2]) / 2, y: (p[1] + p[3]) / 2 };
  }

  private sampleNow(): Frame { return sampleFrame(this.fight, this.tick); }

  private render(dt: number): void {
    const w = this.app.screen.width, h = this.app.screen.height;
    const fr = this.sampleNow();
    const target = cameraFor(fr, w, h);
    this.cam = lerpCamera(this.cam, target, Math.min(1, dt * 6));
    const c = this.cam;
    const sx = this.shake > 0.1 ? (Math.sin(this.time * 90) * this.shake) : 0, sy = this.shake > 0.1 ? (Math.cos(this.time * 77) * this.shake) : 0;
    this.shake *= Math.pow(0.001, dt);
    drawGround(this.ground, this.fight.arena, c.scale, w, h, c.tx, c.ty);
    this.world.position.set(Math.round(c.tx + sx), Math.round(c.ty + sy));
    const col = [hexNum(this.css('--hero'), 0x56b4e9), hexNum(this.css('--monster'), 0xe69f00)];
    const accent = hexNum(this.css('--accent'), 0x2f9e44);
    this.debris.clear();
    for (let i = 0; i < 2; i++) {
      this.flash[i] = Math.max(0, this.flash[i] - dt * 5);
      const det = this.detachedAt[i].map((t) => t !== null && this.tick >= t);
      drawRobot(this.robots[i], fr.f[i], { color: col[i], accent, scale: c.scale, time: this.time, detached: det, flash: this.flash[i] });
      this.detachedAt[i].forEach((t, p) => {
        if (t === null || this.tick < t) return;
        const last = sampleFrame(this.fight, Math.max(0, t - 2)).f[i].parts[p];
        const d = debrisPose(last, (this.tick - t) / 60, p);
        const sc = c.scale;
        this.debris.moveTo(d[0] * sc, -d[1] * sc).lineTo(d[2] * sc, -d[3] * sc).stroke({ width: 14 * sc, color: col[i], cap: 'round', alpha: 0.9 });
      });
      this.robots[i].scale.set(1);
      if (fr.f[i].state === 'staggered' && (this.playing || this.rings.length < 2)) this.rings.push({ ...this.torso(fr, i), age: 0.85, color: 0xffe14a, r: 70 });
    }
    void PART_Z;
    this.drawFx(this.playing && this.freeze <= 0 ? dt : 0); // paused: sparks hold still so a scrubbed hit frame stays visible
    this.onFrame(fr, this.tick);
  }

  private drawFx(dt: number): void {
    const g = this.fxg, sc = this.cam.scale;
    g.clear();
    this.particles = this.particles.filter((p) => (p.life += dt) < p.max);
    for (const p of this.particles) {
      p.x += p.vx * dt; p.y += p.vy * dt; p.vy -= 600 * dt;
      const a = 1 - p.life / p.max;
      g.moveTo(p.x * sc, -p.y * sc).lineTo((p.x - p.vx * 0.03) * sc, -(p.y - p.vy * 0.03) * sc).stroke({ width: p.size, color: p.color, alpha: a, cap: 'round' });
    }
    this.rings = this.rings.filter((r) => (r.age += dt * 3) < 1);
    for (const r of this.rings) g.circle(r.x * sc, -r.y * sc, r.r * sc * (0.4 + r.age)).stroke({ width: 2, color: r.color, alpha: 1 - r.age });
  }
}
