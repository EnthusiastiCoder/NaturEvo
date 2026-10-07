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

const esc = (s: string) => s.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]!);

export async function mountFightView(view: HTMLElement, url: string, title: string): Promise<FightScene | null> {
  let fight: Fight;
  try {
    const res = await fetch(url);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    fight = parseFight(await res.json());
  } catch (e) {
    view.innerHTML = `<p role="alert" style="padding:16px">Could not load fight (${esc(String(e))}). <a href="#/">Back to start</a></p>`;
    return null;
  }
  view.innerHTML = `
  <div class="fv">
    <div class="fv-bar bar">
      <a href="#/" class="btn" title="Back to the run list" aria-label="Back">${svg(ICON.home)}</a>
      <h1>${esc(title)}</h1><span class="spacer"></span>
      <button id="fv-play" title="Play or pause (Space)" aria-label="Play or pause">${svg(ICON.pause)}</button>
      <button id="fv-restart" title="Restart the fight (R)" aria-label="Restart">${svg(ICON.rotate)}</button>
      <select id="fv-speed" title="Playback speed" aria-label="Speed">${SPEEDS.map((s) => `<option value="${s}"${s === 1 ? ' selected' : ''}>${s}x</option>`).join('')}</select>
      <input id="fv-accent" type="color" value="${DEFAULT_ACCENT}" title="Accent colour" aria-label="Accent colour">
      <button id="fv-theme" title="Toggle light or dark theme (T)" aria-label="Toggle theme"></button>
      <button id="fv-info" title="What the HUD shows" aria-label="Info">${svg(ICON.info)}</button>
    </div>
    <div class="fv-stage"><div id="fv-canvas"></div>
      <div class="fv-hud" id="fv-hud"></div>
      <div id="fv-help" class="fv-help" hidden>Bars: core HP (green), energy (blue), posture (yellow; full means staggered). Silhouette: each of 13 parts coloured by health; dark red is disabled. Keys: Space pause, R restart, T theme, arrows step.</div>
      <div id="fv-result" class="fv-result" hidden></div></div>
  </div>`;
  const $ = <T extends HTMLElement>(id: string) => view.querySelector<T>('#' + id)!;
  const scene = new FightScene();
  await scene.mount($('fv-canvas'), fight);

  const hud = $('fv-hud');
  const sideHtml = (i: number) => `<div class="fv-side" id="fv-side${i}">
    <div class="fv-name">${i === 0 ? 'Hero' : 'Monster'} <span class="fv-state"></span></div>
    <div class="fv-bar-row" title="Core (torso) health"><i class="b hp"></i></div>
    <div class="fv-bar-row" title="Energy: spent by movement and guarding"><i class="b en"></i></div>
    <div class="fv-bar-row" title="Posture: full means staggered"><i class="b po"></i></div>
    <svg class="fv-sil" viewBox="0 0 40 72">${SIL.map(([x, y, w, h], k) => `<rect x="${x - w}" y="${y - h / 2}" width="${w * 2}" height="${h}" rx="2"><title>${PART_NAMES[k]}</title></rect>`).join('')}</svg>
    <div class="fv-tally"></div></div>`;
  hud.innerHTML = sideHtml(0) + sideHtml(1);
  const sides = [0, 1].map((i) => hud.querySelector<HTMLElement>(`#fv-side${i}`)!);
  const setBar = (el: HTMLElement, cls: string, v: number) => { el.querySelector<HTMLElement>('.' + cls)!.style.width = `${Math.max(0, Math.min(100, v))}%`; };
  let lastKey = '';
  scene.onFrame = (fr, tick) => {
    const key = `${fr.t}|${Math.round(tick)}`;
    if (key === lastKey) return;
    lastKey = key;
    const m = hudModel(fr);
    m.fighters.forEach((f, i) => {
      setBar(sides[i], 'hp', f.coreHp); setBar(sides[i], 'en', f.energy); setBar(sides[i], 'po', f.posture);
      sides[i].querySelectorAll<SVGRectElement>('rect').forEach((r, k) => { r.style.fill = f.parts[k].color; });
      sides[i].querySelector('.fv-state')!.textContent = fr.f[i].state === 'idle' ? '' : fr.f[i].state;
      const items = defenceTallyModel(tallyAt(fight, i, tick)).items;
      sides[i].querySelector('.fv-tally')!.innerHTML = items.map((x) => `<span title="${esc(x.tooltip)}">${x.key} ${x.count}</span>`).join('');
    });
  };
  const res = $('fv-result');
  scene.onEnd = () => { res.hidden = false; res.textContent = `Result: ${fight.result.outcome.replace('_', ' ')} after ${(fight.result.ticks / 60).toFixed(1)} s`; };

  const root = document.documentElement;
  let theme: Theme = (root.dataset.theme as Theme) || 'dark';
  const syncTheme = () => { applyTheme(root, theme); $('fv-theme').innerHTML = svg(themeIconPath(theme)); };
  syncTheme();
  const play = $('fv-play');
  const syncPlay = () => { play.innerHTML = svg(scene.playing ? ICON.pause : ICON.play); };
  const toggle = () => { scene.playing = !scene.playing; syncPlay(); };
  const restart = () => { scene.seek(0); res.hidden = true; scene.playing = true; syncPlay(); };
  play.onclick = toggle;
  $('fv-restart').onclick = restart;
  $<HTMLSelectElement>('fv-speed').onchange = (e) => { scene.speed = Number((e.target as HTMLSelectElement).value); };
  $<HTMLInputElement>('fv-accent').oninput = (e) => setAccent(root, (e.target as HTMLInputElement).value);
  $('fv-theme').onclick = () => { theme = nextTheme(theme); syncTheme(); };
  $('fv-info').onclick = () => { const h = $('fv-help'); h.hidden = !h.hidden; };
  window.addEventListener('keydown', (e) => {
    if ((e.target as HTMLElement).matches?.('input,select')) return;
    if (e.key === ' ') { e.preventDefault(); toggle(); }
    else if (e.key === 'r' || e.key === 'R') restart();
    else if (e.key === 't' || e.key === 'T') { theme = nextTheme(theme); syncTheme(); }
    else if (e.key === 'ArrowRight') { scene.playing = false; syncPlay(); scene.seek(scene.tick + 2); }
    else if (e.key === 'ArrowLeft') { scene.playing = false; syncPlay(); scene.seek(scene.tick - 2); }
  });
  return scene;
}
