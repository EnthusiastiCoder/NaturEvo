import { Application, Container, Graphics, Text } from 'pixi.js';
import type { Fight, FighterFrame, Frame, Seg } from './data';
import { cameraFor, lerpCamera, type Camera } from './camera';
import { blockPlates, debrisPose, drawRobot, PART_Z } from './robot-draw';
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
  private freeze = 0;
  private slowLeft = 0;
  private slowFactor = 1;
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
    const mk = (size: number) => new Text({ text: '', style: { fontFamily: 'system-ui, Segoe UI, sans-serif', fontWeight: '900', fontSize: size, fill: 0xffffff, stroke: { color: 0x000000, width: Math.max(3, size / 7), join: 'round' } } });
    this.banner = mk(120); this.bannerSub = mk(30);
    for (const t of [this.banner, this.bannerSub]) { t.anchor.set(0.5); t.visible = false; }
    this.app.stage.addChild(this.ground, this.world, this.screenFx, this.banner, this.bannerSub);
    this.world.addChild(this.debris, this.robots[0], this.robots[1], this.fxg);
    this.cam = cameraFor(fight.frames[0], this.app.screen.width, this.app.screen.height);
    this.app.ticker.add((tk) => this.update(tk.deltaMS / 1000));
  }

  seek(tick: number): void {
    this.tick = Math.max(0, tick); this.fired = Math.ceil(this.tick) - 1; this.particles = []; this.rings = [];
    this.freeze = 0; this.slowLeft = 0;
    this.detachedAt = [new Array(13).fill(null), new Array(13).fill(null)];
    for (const e of this.fight.events) if (e.kind === 'destroyed' && e.t <= this.tick) this.detachedAt[e.who][this.fight.parts.indexOf(e.part)] = e.t;
    // scrubbing onto an impact must still show its sparks: replay recent events, pre-aged
    for (const { e, age } of recentEvents(this.fight.events, this.tick)) this.burst(e, age);
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
        const k = this.slowLeft > 0 ? this.slowFactor : 1;
        const adv = dt * ticksPerSec * this.speed * k;
        if (this.slowLeft > 0) this.slowLeft = Math.max(0, this.slowLeft - adv / k);
        this.tick = Math.min(this.lastTick, this.tick + adv);
        for (const e of this.fight.events) {
          if (e.t > this.fired && e.t <= this.tick) this.fire(e);
        }
        this.fired = Math.floor(this.tick);
        if (prev < this.lastTick && this.tick >= this.lastTick) this.onEnd();
      }
    }
    this.render(dt);
  }

  /** World point (cm) where an event's effect appears. Block sparks snap onto the nearest drawn shield plate; ko/destroyed use the part. */
  private evPoint(e: Ev): { x: number; y: number } {
    if (e.kind === 'block') {
      const fr = sampleFrame(this.fight, e.t);
      const [x, y] = blockPoint(e.x, e.y, blockPlates(fr.f, sampleFrame(this.fight, e.t - 2).f, this.fight.geometry.shieldAlong, this.fight.geometry.shieldLen));
      return { x, y };
    }
    if (e.kind === 'destroyed') {
      const p = sampleFrame(this.fight, e.t).f[e.who].parts[Math.max(0, this.fight.parts.indexOf(e.part))];
      return { x: (p[0] + p[2]) / 2, y: (p[1] + p[3]) / 2 };
    }
    if (e.kind === 'ko') return this.torso(sampleFrame(this.fight, e.t), e.who);
    return { x: e.x, y: e.y };
  }

  /** Spawn the stateful sparks for an event, pre-aged by `age` seconds. */
  private burst(e: Ev, age: number): void {
    const layers = sparkLayers(e);
    if (!layers.length) return;
    const pt = this.evPoint(e);
    layers.forEach((spec, k) => this.particles.push(...ageParticles(spawnParticles(spec, pt.x, pt.y, e.t * 31 + 7 + k * 101), age)));
  }

  private fire(e: Ev): void {
    this.burst(e, 0);
    if (e.kind === 'stagger') this.rings.push({ ...this.torso(this.sampleNow(), e.who), age: 0, color: 0xffe14a, r: 70 });
    if (e.kind === 'destroyed') this.detachedAt[e.who][this.fight.parts.indexOf(e.part)] = e.t;
    if (e.kind === 'hit') {
      const p = hitstopPlan(hitstopTicks(this.fight.events, e.t), this.reduceMotion);
      this.freeze = Math.max(this.freeze, p.freeze);
      if (p.slow > 0) { this.slowLeft = p.slow; this.slowFactor = p.slowFactor; }
    }
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
    // every effect below is a pure function of (tick - event.t): scrub and pause stay deterministic
    const rec = recentEvents(this.fight.events, this.tick, 60).map(({ e, age }) => ({ e, a: age * 60 }));
    let sx = 0, sy = 0, best = -1;
    const pf = [new Array(13).fill(0), new Array(13).fill(0)], body = [0, 0], rc = [[0, 0], [0, 0]];
    for (const { e, a } of rec) {
      if (!this.reduceMotion) {
        const [ox, oy] = shakeOffset(a, shakeFor(e), e.t);
        if (Math.hypot(ox, oy) > best) { best = Math.hypot(ox, oy); sx = ox; sy = oy; }
      }
      if (e.kind === 'hit') {
        const v = e.victim as number, pi = this.fight.parts.indexOf(e.part);
        if (pi >= 0) pf[v][pi] = Math.max(pf[v][pi], partFlashAt(a));
        body[v] = Math.max(body[v], partFlashAt(a) * 0.35);
        if (!this.reduceMotion) {
          const away = Math.sign(this.torso(fr, v).x - this.torso(fr, 1 - v).x) || 1, r = recoilCm(a, e.impact);
          rc[v][0] += away * r; rc[v][1] += e.dy * r * 0.3;
        }
      }
    }
    if (this.freeze > 0 && !this.reduceMotion) { sx += Math.sin(this.time * 120) * 2.5; sy += Math.cos(this.time * 97) * 2.5; }
    drawGround(this.ground, this.fight.arena, c.scale, w, h, c.tx, c.ty);
    this.world.position.set(Math.round(c.tx + sx), Math.round(c.ty + sy));
    const col = [hexNum(this.css('--hero'), 0x56b4e9), hexNum(this.css('--monster'), 0xe69f00)];
    const accent = hexNum(this.css('--accent'), 0x2f9e44);
    this.debris.clear();
    for (let i = 0; i < 2; i++) {
      const det = this.detachedAt[i].map((t) => t !== null && this.tick >= t);
      drawRobot(this.robots[i], fr.f[i], { color: col[i], accent, scale: c.scale, time: this.tick / 60, detached: det, flash: body[i], partFlash: pf[i], geom: this.fight.geometry });
      this.detachedAt[i].forEach((t, p) => {
        if (t === null || this.tick < t) return;
        const last = sampleFrame(this.fight, Math.max(0, t - 2)).f[i].parts[p];
        const d = debrisPose(last, (this.tick - t) / 60, p);
        const sc = c.scale;
        this.debris.moveTo(d[0] * sc, -d[1] * sc).lineTo(d[2] * sc, -d[3] * sc).stroke({ width: 14 * sc, color: col[i], cap: 'round', alpha: 0.9 });
      });
      this.robots[i].position.set(rc[i][0] * c.scale, -rc[i][1] * c.scale);
      if (fr.f[i].state === 'staggered' && (this.playing || this.rings.length < 2)) this.rings.push({ ...this.torso(fr, i), age: 0.85, color: 0xffe14a, r: 70 });
    }
    void PART_Z;
    const slow = this.slowLeft > 0 ? this.slowFactor : 1;
    this.drawFx(this.playing && this.freeze <= 0 ? dt * slow : 0, rec, w, h); // paused: sparks hold still so a scrubbed frame stays visible
    this.onFrame(fr, this.tick);
  }

  private label(i: number, text: string, x: number, y: number, tint: number, size: number, alpha: number): void {
    let t = this.labels[i];
    if (!t) { t = this.labels[i] = new Text({ text: '', style: { fontFamily: 'system-ui, Segoe UI, sans-serif', fontWeight: '900', fontSize: 34, fill: 0xffffff, stroke: { color: 0x000000, width: 5, join: 'round' } } }); t.anchor.set(0.5); this.world.addChild(t); }
    if (t.text !== text) t.text = text;
    t.tint = tint; t.alpha = alpha; t.scale.set(size); t.position.set(x, Math.max(y, 28 - this.world.y)); t.visible = true;
  }

  private drawFx(dt: number, rec: { e: Ev; a: number }[], w: number, h: number): void {
    const g = this.fxg, sc = this.cam.scale;
    g.clear();
    this.particles = this.particles.filter((p) => (p.life += dt) < p.max);
    for (const p of this.particles) {
      p.x += p.vx * dt; p.y += p.vy * dt; p.vy -= 600 * dt;
      const a = 1 - p.life / p.max;
      g.moveTo(p.x * sc, -p.y * sc).lineTo((p.x - p.vx * 0.035) * sc, -(p.y - p.vy * 0.035) * sc).stroke({ width: p.size * 1.6, color: p.color, alpha: a, cap: 'round' });
    }
    this.rings = this.rings.filter((r) => (r.age += dt * 3) < 1);
    for (const r of this.rings) g.circle(r.x * sc, -r.y * sc, r.r * sc * (0.4 + r.age)).stroke({ width: 2, color: r.color, alpha: 1 - r.age });
    // stateless flashes, shockwaves and flares
    let n = 0;
    const seen = new Set<string>();
    for (const { e, a } of rec) {
      if (!['hit', 'clash', 'block', 'destroyed', 'ko'].includes(e.kind)) continue;
      const pt = this.evPoint(e), X = pt.x * sc, Y = -pt.y * sc;
      if (e.kind === 'hit') {
        const fx = hitFx(e.impact), k = a / 14;
        if (a < 5) g.circle(X, Y, fx.flashR * sc * (1 + a * 0.15)).fill({ color: 0xffffff, alpha: 0.95 * (1 - a / 5) });
        if (a < 9) g.circle(X, Y, fx.flashR * 1.8 * sc).fill({ color: 0xffa83a, alpha: 0.45 * (1 - a / 9) });
        if (k < 1) {
          g.circle(X, Y, fx.ringR * sc * (0.25 + 0.9 * Math.sqrt(k))).stroke({ width: Math.max(2, 5 * (1 - k)), color: 0xffffff, alpha: 0.9 * (1 - k) });
          g.circle(X, Y, fx.ringR * 0.7 * sc * (0.25 + 0.9 * Math.sqrt(k))).stroke({ width: 3, color: 0xffa83a, alpha: 0.8 * (1 - k) });
        }
        const f = floatNumber(a);
        if (f) {
          const dmg = Math.round(e.dmg ?? 0), nm = PART_SHORT[e.part] ?? e.part;
          this.label(n++, `${nm} -${dmg}`, X, Y - (30 + f.rise) * sc, dmg >= 40 ? 0xff5a36 : 0xffd166, 0.7 + 0.35 * fx.scale, f.alpha);
        }
      } else if (e.kind === 'clash') {
        const key = `${e.t}|${e.type}`, k = a / 16;
        const perfect = e.type === 'perfect_parry', deflect = e.type === 'parry';
        if (k < 1) {
          const R = (perfect ? 90 : deflect ? 70 : 50) * sc, col = perfect ? 0xfff2b0 : deflect ? 0xbfeaff : 0xffe9a8;
          const spokes = perfect ? 8 : 6;
          for (let s = 0; s < spokes; s++) { // star flare: crossed streaks
            const ang = (s / spokes) * Math.PI + (e.t % 7) * 0.2, L = R * (1 - k) * (s % 2 ? 0.7 : 1);
            g.moveTo(X - Math.cos(ang) * L, Y - Math.sin(ang) * L).lineTo(X + Math.cos(ang) * L, Y + Math.sin(ang) * L).stroke({ width: 3, color: col, alpha: 1 - k, cap: 'round' });
          }
          if (a < 4) g.circle(X, Y, 22 * sc).fill({ color: 0xffffff, alpha: 0.9 * (1 - a / 4) });
          if (deflect || perfect) g.circle(X, Y, R * (0.3 + k)).stroke({ width: 4, color: col, alpha: 0.9 * (1 - k) });
        }
        if ((deflect || perfect) && !seen.has(key)) {
          seen.add(key);
          const f = floatNumber(a);
          if (f) this.label(n++, perfect ? 'PERFECT PARRY' : 'PARRY', X, Y - (40 + f.rise * 0.6) * sc, perfect ? 0xffe14a : 0x9fe1ff, perfect ? 0.95 : 0.8, f.alpha);
        }
      } else if (e.kind === 'block') {
        const k = a / 16;
        if (k < 1) {
          if (a < 6) g.circle(X, Y, 26 * sc).fill({ color: 0xbfeaff, alpha: 0.85 * (1 - a / 6) });
          g.circle(X, Y, 55 * sc * (0.3 + k)).stroke({ width: 4, color: 0x7fd4ff, alpha: 1 - k });
          g.circle(X, Y, 35 * sc * (0.3 + k)).stroke({ width: 2, color: 0xffffff, alpha: 1 - k });
        }
        const f = floatNumber(a);
        if (f) this.label(n++, 'BLOCK', X, Y - (40 + f.rise * 0.6) * sc, 0x7fd4ff, 0.8, f.alpha);
      } else if (e.kind === 'destroyed') {
        const k = a / 20;
        if (k < 1) g.circle(X, Y, 70 * sc * (0.3 + k)).stroke({ width: 5, color: 0xff7a2a, alpha: 1 - k });
        const f = floatNumber(a);
        if (f) this.label(n++, 'PART OFF!', X, Y - (85 + f.rise) * sc, KO_TINT, 0.9, f.alpha);
      } else if (e.kind === 'ko') {
        const k = a / 30;
        if (k < 1) for (const [rr, wd] of [[160, 7], [100, 4]]) g.circle(X, Y, rr * sc * (0.2 + k)).stroke({ width: wd, color: 0xffffff, alpha: 1 - k });
      }
    }
    for (let i = n; i < this.labels.length; i++) this.labels[i].visible = false;
    this.drawBanner(w, h);
  }

  /** Big K.O. banner from the first ko event onwards (pops in, then holds); a white screen flash on the blow. */
  private drawBanner(w: number, h: number): void {
    const sf = this.screenFx; sf.clear();
    const ko = this.fight.events.find((e) => e.kind === 'ko');
    const a = ko ? this.tick - Math.min(ko.t, this.lastTick) + (ko.t > this.lastTick ? 10 : 0) : -1;
    const on = !!ko && a >= 0;
    this.banner.visible = this.bannerSub.visible = on;
    if (!on || !ko) return;
    if (!this.reduceMotion && a < 8) sf.rect(0, 0, w, h).fill({ color: 0xffffff, alpha: 0.5 * (1 - a / 8) });
    const pop = a < 10 ? 1.6 - 0.6 * (a / 10) + Math.sin(a / 10 * Math.PI) * 0.2 : 1, fit = Math.min(1, w / 760) * (h < 700 ? 0.75 : 1);
    this.banner.text = 'K.O.'; this.banner.tint = KO_TINT; this.banner.alpha = Math.min(1, a / 3 + 0.2);
    this.banner.scale.set(pop * fit); this.banner.position.set(w / 2, h * 0.3);
    this.bannerSub.text = `${ko.who === 0 ? 'HERO' : 'MONSTER'} DOWN`; this.bannerSub.tint = 0xffffff; this.bannerSub.alpha = this.banner.alpha;
    this.bannerSub.scale.set(fit); this.bannerSub.position.set(w / 2, h * 0.3 + 78 * fit);
  }
}
