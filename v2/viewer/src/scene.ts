import { Application, Container, Graphics, Sprite, Text, Texture } from 'pixi.js';
import { type Fight, type Run } from './data';
import { STAGE_H, STAGE_W, fightCamera, fitStage, mapCamera, separate, stateLabel } from './layout';
import { sceneSize } from './record';
import { hash2, paintGround, paintRock, paintTree } from './ground';
import { sampleFight, type FighterFrame } from './interp';
import { ANIM, Atlas, HERO_MODEL, MONSTER_MODELS, directionFromAngle, loadAtlas } from './sprites';

export { STAGE_W, STAGE_H };
export interface SceneOpts { record?: boolean; w?: number; h?: number }
const PITCH = 0.8;
const ATTACK_TICKS = { Windup: 0, Active: 6, Recovery: 10 } as Record<string, number>;
const ATTACK_TOTAL = 16;

export interface FightOpts { rays: boolean }
interface Rig { sprite: Sprite; label: Text }

export class Scene {
  app!: Application;
  private atlases = new Map<string, Atlas>();
  private world = new Container();
  private layer = new Container();
  private overlay = new Graphics();
  private fx = new Graphics();
  private ground: Sprite | null = null;
  private sw = STAGE_W;
  private sh = STAGE_H;
  private res = 1;
  private arenaPx = { w: 1, h: 1 };
  private clip: Graphics | null = null;
  private rigs: Rig[] = [];
  private mode: 'fight' | 'map' | null = null;
  private ts = 32;
  private blocked: ((x: number, y: number) => boolean) | null = null;
  private trees = [0, 1, 2].map((v) => Texture.from(paintTree(v)));
  private rocks = [0, 1, 2].map((v) => Texture.from(paintRock(v)));
  private heroKey = HERO_MODEL;
  private monsterKey = MONSTER_MODELS[0];

  static async create(host: HTMLElement, o: SceneOpts = {}): Promise<Scene> {
    const s = new Scene();
    s.app = new Application();
    const size = sceneSize(!!o.record, window.devicePixelRatio || 1);
    s.sw = o.w ?? size.w; s.sh = o.h ?? size.h; s.res = size.resolution;
    // transparent outside record so the themed stage shows; record frames are opaque
    await s.app.init({ width: s.sw, height: s.sh, antialias: true, ...(o.record ? { background: 0x0c120b } : { backgroundAlpha: 0 }), resolution: size.resolution, autoDensity: !o.record });
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
    const f = fitStage(hostW - 2, hostH - 2, this.sw, this.sh);
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
    g.scale.set(1 / this.res); // texture is painted at device resolution
    g.zIndex = -1e9;
    this.layer.addChild(g);
    this.ground = g;
    // clip props (border trees) to the map rectangle
    this.clip?.destroy(); this.clip = new Graphics();
    this.clip.rect(0, 0, canvas.width / this.res, canvas.height / this.res).fill(0xffffff);
    this.world.addChild(this.clip); this.layer.mask = this.clip;
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
      const label = new Text({ text: '', style: { fontFamily: 'system-ui, sans-serif', fontSize: 15, fontWeight: '800', fill: 0xffffff, stroke: { color: 0x000000, width: 4 } }, resolution: 3 });
      label.anchor.set(0.5, 1); label.visible = false; label.zIndex = 1e8 + 1;
      this.layer.addChild(sprite, label);
      this.rigs.push({ sprite, label });
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

  private pose(fr: FighterFrame, prevPos: [number, number], model: string, f: Fight, i: number, t: number, hitRecent: boolean, role: 0 | 1): void {
    const atlas = this.atlases.get(model)!;
    const rig = this.rigs[role];
    const frames = f.frames;
    const idx = Math.min(frames.length - 1, Math.floor(t * f.tickHz));
    const key = role === 0 ? 'hero' : 'monster';
    let ticks = 0;
    while (ticks < 24 && idx - ticks > 0 && frames[idx - ticks - 1][key].state === fr.state) ticks++;
    const frac = t * f.tickHz - Math.floor(t * f.tickHz);
    const moving = Math.hypot(fr.pos[0] - prevPos[0], fr.pos[1] - prevPos[1]) > 0.004;
    let anim = moving ? ANIM.walk : ANIM.idle, n = atlas.count(anim), fi = Math.floor(t * 12);
    if (fr.state in ATTACK_TICKS) {
      anim = ANIM.attack; n = atlas.count(anim);
      fi = Math.floor(((ATTACK_TICKS[fr.state] + ticks + frac) / ATTACK_TOTAL) * n);
    } else if (fr.state === 'Dead') {
      anim = ANIM.death; n = atlas.count(anim);
      fi = Math.min(n - 1, Math.floor((ticks + frac) / 2));
    } else if (hitRecent) { anim = ANIM.hit; n = atlas.count(anim); fi = Math.floor(t * 20) % n; }
    const pick = atlas.get(anim, directionFromAngle(fr.facing), Math.min(fi, 1e6));
    const sp = rig.sprite;
    if (pick) { sp.texture = pick.texture; sp.anchor.set(pick.ax, pick.ay); }
    const sc = this.mode === 'map' ? 0.5 : 0.62;
    sp.scale.set(sc);
    sp.position.set(fr.pos[0] * this.ts, fr.pos[1] * this.ts * PITCH);
    sp.zIndex = sp.y;
    sp.alpha = fr.state === 'Dodging' ? 0.55 : 1;
    sp.tint = hitRecent && fr.state !== 'Dead' ? 0xffb0b0 : 0xffffff;
    void i;
  }

  drawFight(f: Fight, t: number, o: FightOpts): { hero: FighterFrame; monster: FighterFrame } {
    const hz = f.tickHz;
    const s = sampleFight(f.frames, t, hz);
    const p = sampleFight(f.frames, Math.max(0, t - 1 / hz), hz);
    const back = sampleFight(f.frames, Math.max(0, t - 0.2), hz);
    this.pose(s.hero, p.hero.pos, this.heroKey, f, 0, t, back.hero.hp - s.hero.hp > 0.5, 0);
    this.pose(s.monster, p.monster.pos, this.monsterKey, f, 1, t, back.monster.hp - s.monster.hp > 0.5, 1);
    const g = this.overlay; g.clear();
    const TS = this.ts, P = PITCH;
    const fighters: [FighterFrame, FighterFrame, number][] = [[s.hero, s.monster, 0x4cc9f0], [s.monster, s.hero, 0xf4845f]];
    // ground rings + telegraphs
    for (const [a, , col] of fighters) {
      const cx = a.pos[0] * TS, cy = a.pos[1] * TS * P;
      g.ellipse(cx, cy, 0.5 * TS, 0.5 * TS * P).stroke({ width: 2, color: col, alpha: 0.9 });
      if (a.state === 'Windup' || a.state === 'Active') {
        g.moveTo(cx, cy);
        for (let k = 0; k <= 12; k++) {
          const ang = a.facing + (k / 12 - 0.5) * 1.3;
          g.lineTo(cx + Math.cos(ang) * 1.7 * TS, cy + Math.sin(ang) * 1.7 * TS * P);
        }
        g.closePath().fill({ color: 0xff3b30, alpha: a.state === 'Active' ? 0.45 : 0.22 });
      }
      if (a.state === 'Blocking') {
        g.ellipse(cx + Math.cos(a.facing) * 0.7 * TS, cy + Math.sin(a.facing) * 0.7 * TS * P, 0.35 * TS, 0.5 * TS * P).stroke({ width: 4, color: 0x9ad1ff, alpha: 0.95 });
      }
    }
    if (o.rays) for (const [a, b, col] of fighters) this.drawRays(g, a, b, col);
    // bars
    for (const [a, , col] of fighters) {
      const cx = a.pos[0] * TS, cy = a.pos[1] * TS * P - 62;
      const w = 54;
      g.roundRect(cx - w / 2 - 1, cy - 1, w + 2, 12, 3).fill({ color: 0x000000, alpha: 0.6 });
      g.rect(cx - w / 2, cy, w * Math.max(0, a.hp) / 100, 6).fill({ color: a.hp > 35 ? 0x6bd968 : 0xe5484d });
      g.rect(cx - w / 2, cy + 7, w * Math.max(0, a.stamina) / 100, 3).fill({ color: 0xf5d90a });
      g.circle(cx - w / 2 - 6, cy + 4, 3).fill({ color: col });
    }
    this.particles(t, 0, 0, STAGE_W / this.world.scale.x, STAGE_H / this.world.scale.y, true);
    this.app.render();
    return s;
  }

  private drawRays(g: Graphics, a: FighterFrame, b: FighterFrame, col: number): void {
    const N = 15, FOV = 2.094, RANGE = 12;
    const TS = this.ts, P = PITCH;
    const pts: number[] = [a.pos[0] * TS, a.pos[1] * TS * P];
    const hits: [number, number, boolean][] = [];
    for (let i = 0; i < N; i++) {
      const ang = a.facing + (i / (N - 1) - 0.5) * FOV;
      const dx = Math.cos(ang), dy = Math.sin(ang);
      let d = 0, seen = false;
      for (; d < RANGE; d += 0.25) {
        const x = a.pos[0] + dx * d, y = a.pos[1] + dy * d;
        if (this.blocked?.(x, y)) break;
        if (Math.hypot(x - b.pos[0], y - b.pos[1]) < 0.45) { seen = true; break; }
      }
      const ex = (a.pos[0] + dx * d) * TS, ey = (a.pos[1] + dy * d) * TS * P;
      pts.push(ex, ey);
      hits.push([ex, ey, seen]);
    }
    g.poly(pts).fill({ color: col, alpha: 0.09 });
    for (const [ex, ey, seen] of hits) {
      g.moveTo(pts[0], pts[1]).lineTo(ex, ey).stroke({ width: seen ? 2 : 1, color: seen ? 0xff4d4d : col, alpha: seen ? 0.95 : 0.35 });
    }
  }

  /** Ambient fireflies/pollen: pure function of t so every frame is reproducible. */
  private particles(t: number, x0: number, y0: number, w: number, h: number, screenless: boolean): void {
    void screenless;
    const g = this.fx; g.clear();
    const sc = this.world.scale.x, ox = this.world.x, oy = this.world.y;
    for (let i = 0; i < 36; i++) {
      const bx = hash2(i, 1) * w, by = hash2(i, 2) * h;
      const x = x0 + bx + Math.sin(t * 0.4 + i) * 18, y = y0 + by + Math.cos(t * 0.33 + i * 1.7) * 14 - ((t * 6 + i * 13) % 40);
      const a = 0.25 + 0.35 * (0.5 + 0.5 * Math.sin(t * 2 + i * 3));
      g.circle(ox + x * sc, oy + y * sc, 1.6).fill({ color: i % 3 ? 0xd7ff8a : 0xfff2a8, alpha: a });
    }
  }

  // ---------- map ----------
  setMap(run: Run): void {
    const ts = 48;
    this.reset('map', ts);
    const m = run.map;
    const tileAt = (x: number, y: number) => m.tiles[Math.max(0, Math.min(m.h - 1, y))][Math.max(0, Math.min(m.w - 1, x))];
    this.blocked = null;
    this.addGround(paintGround({ w: m.w, h: m.h, seed: 7, tile: tileAt, zone: (x, y) => Number(m.zones[Math.max(0, Math.min(m.h - 1, y))][Math.max(0, Math.min(m.w - 1, x))]) }, ts, PITCH));
    const gx = this.overlay;
    void gx;
    for (let y = 0; y < m.h; y++) for (let x = 0; x < m.w; x++) {
      const t = m.tiles[y][x];
      if (t === 'T') this.addProp('T', x, y, 0.95);
      else if (t === 'R') this.addProp('R', x, y, 0.9);
      else if (t === '#' || t === 'F') {
        const b = new Graphics();
        const px = x * ts, py = y * ts * PITCH;
        b.rect(px, py + ts * PITCH * 0.3, ts, ts * PITCH * 0.7).fill(t === '#' ? 0x6f7078 : 0x7a5c3a);
        b.rect(px, py + ts * PITCH * 0.3, ts, 5).fill(t === '#' ? 0x9a9ba3 : 0xa58058);
        b.zIndex = py + ts * PITCH;
        this.layer.addChild(b);
      } else if (t === 'C') {
        const b = new Graphics();
        const px = (x + 0.5) * ts, py = (y + 0.6) * ts * PITCH;
        b.poly([px - 20, py + 8, px, py - 24, px + 20, py + 8]).fill(0xb4552d);
        b.poly([px - 5, py + 8, px, py - 4, px + 5, py + 8]).fill(0x2a1a10);
        b.circle(px + 26, py + 8, 4).fill(0xffb347);
        b.zIndex = py;
        this.layer.addChild(b);
      }
    }
    this.addFighters();
    this.monsterKey = MONSTER_MODELS[0];
  }

  drawMap(run: Run, pathPos: number, t: number, activeEnc: number): FighterFrame {
    const m = run.map, ts = this.ts;
    const n = run.path.length - 1;
    const i = Math.max(0, Math.min(n, Math.floor(pathPos))), j = Math.min(n, i + 1), f = Math.min(1, pathPos - i);
    const a = run.path[i], b = run.path[j];
    const x = a[0] + (b[0] - a[0]) * f + 0.5, y = a[1] + (b[1] - a[1]) * f + 0.5;
    const facing = Math.atan2(b[1] - a[1], b[0] - a[0]);
    const hero: FighterFrame = { pos: [x, y], facing: i === j ? 0 : facing, hp: 100, stamina: 100, action: 'none', state: 'Idle' };
    const moving = i !== j && pathPos < n;
    const at = this.atlases.get(HERO_MODEL)!;
    const pick = at.get(moving ? ANIM.walk : ANIM.idle, directionFromAngle(hero.facing), Math.floor(t * 12));
    const sp = this.rigs[0].sprite;
    if (pick) { sp.texture = pick.texture; sp.anchor.set(pick.ax, pick.ay); }
    sp.scale.set(0.5); sp.position.set(x * ts, y * ts * PITCH); sp.zIndex = sp.y;
    this.rigs[1].sprite.visible = false;
    // camera follows the hero, clamped to the map
    const zoom = 1;
    const wpx = m.w * ts, hpx = m.h * ts * PITCH;
    const cx = Math.max(STAGE_W / 2, Math.min(wpx - STAGE_W / 2, x * ts)), cy = Math.max(STAGE_H / 2, Math.min(hpx - STAGE_H / 2, y * ts * PITCH));
    this.world.scale.set(zoom);
    this.world.position.set(STAGE_W / 2 - cx * zoom, STAGE_H / 2 - cy * zoom);
    const g = this.overlay; g.clear();
    const px = (p: [number, number]) => (p[0] + 0.5) * ts, py = (p: [number, number]) => (p[1] + 0.5) * ts * PITCH;
    // route: travelled part solid, ahead dotted
    g.moveTo(px(run.path[0]), py(run.path[0]));
    for (let k = 1; k <= i; k++) g.lineTo(px(run.path[k]), py(run.path[k]));
    g.lineTo(x * ts, y * ts * PITCH).stroke({ width: 4, color: 0xfff2a8, alpha: 0.9 });
    for (let k = i; k < n; k += 1) {
      if (k % 2) continue;
      g.moveTo(px(run.path[k]), py(run.path[k])).lineTo(px(run.path[Math.min(n, k + 1)]), py(run.path[Math.min(n, k + 1)])).stroke({ width: 3, color: 0xffffff, alpha: 0.35 });
    }
    run.encounters.forEach((e, k) => {
      const col = e.kind === 'boss' ? 0xe5484d : e.kind === 'elite' ? 0xf5a623 : 0x6bd968;
      const cx2 = px(e.tile), cy2 = py(e.tile) - 6 + Math.sin(t * 3 + k) * 2;
      g.circle(cx2, cy2, k === activeEnc ? 13 : 9).fill({ color: col, alpha: 0.95 }).stroke({ width: 2, color: 0x000000, alpha: 0.6 });
      g.moveTo(cx2 - 4, cy2 - 4).lineTo(cx2 + 4, cy2 + 4).moveTo(cx2 + 4, cy2 - 4).lineTo(cx2 - 4, cy2 + 4).stroke({ width: 2, color: 0x000000, alpha: 0.8 });
    });
    this.particles(t, cx - STAGE_W / 2, cy - STAGE_H / 2, STAGE_W, STAGE_H, true);
    this.app.render();
    return hero;
  }

  showMonster(): void { this.rigs[1].sprite.visible = true; }
}
