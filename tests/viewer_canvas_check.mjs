// Crisp canvases (plans/2026-10-20_viewer-redesign-v2.md, "Addendum 4"). Static markup/CSS checks plus
// behaviour via the stub-DOM harness (stubs: devicePixelRatio, clientWidth/clientHeight, setTransform spy,
// matchMedia resolution listeners, ResizeObserver). Derived from requirements, not from current code.
import { html, load, makeRun, tick, runner } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_canvas_check');
const css = ((html.match(/<style[^>]*>([\s\S]*?)<\/style>/) || [, ''])[1]).replace(/\/\*[\s\S]*?\*\//g, '');
const markup = html.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<style[\s\S]*?<\/style>/g, '');

// ---------- static: no hard-coded sizes ----------
{
  const tags = [...markup.matchAll(/<canvas\b[^>]*>/gi)].map(m => m[0]);
  check(tags.length > 0, 'page must contain at least one <canvas>');
  for (const t of tags) {
    check(!/\swidth\s*=/i.test(t) && !/\sheight\s*=/i.test(t), 'canvas must not have width/height attributes in markup: ' + t);
    check(!/style\s*=\s*["'][^"']*(width|height)\s*:\s*\d+(\.\d+)?px/i.test(t), 'canvas must not have inline fixed-px width/height: ' + t);
  }
  const ids = tags.map(t => (t.match(/\bid\s*=\s*["']([^"']+)["']/) || [])[1]).filter(Boolean);
  for (const m of css.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
    const sels = m[1].split(',').map(s => s.trim());
    if (!sels.some(s => /(^|[\s>+~])canvas\b/i.test(s) || ids.some(i => s.endsWith('#' + i)))) continue;
    for (const d of m[2].split(';')) {
      const mm = d.match(/^\s*(width|height)\s*:\s*([^;]+)$/i);
      if (mm) check(!/^\s*\d+(\.\d+)?px\s*$/i.test(mm[2]), `CSS rule "${m[1].trim()}" must not set fixed-px ${mm[1]}: ${mm[2].trim()}`);
    }
  }
}

// ---------- behaviour harness ----------
function boot(dpr, cw, ch) {
  const rec = { ro: [], mq: [], ctx: [], gets: 0 };
  const pre = w => {
    w.devicePixelRatio = dpr;
    w.ResizeObserver = class { constructor(cb) { this.cb = cb; rec.ro.push(this); } observe() {} unobserve() {} disconnect() {} };
    w.matchMedia = q => { const m = { media: q, matches: false, _l: [], addEventListener(t, f) { this._l.push(f); }, removeEventListener() {}, addListener(f) { this._l.push(f); }, removeListener() {} }; rec.mq.push(m); return m; };
  };
  const v = load({ run: makeRun(300), pre });
  const cv = v.$('timeline');
  const mk = () => new Proxy({}, {
    get: (t, k) => k in t ? t[k] : (...a) => { rec.ctx.push([String(k), a]); },
    set: (t, k, val) => { t[k] = val; if (k === 'font') rec.ctx.push(['font', [String(val)]]); return true; },
  });
  if (cv) cv.getContext = () => { rec.gets++; return mk(); };
  const size = (w, h) => { for (const e of [cv, cv && cv.parent]) if (e) { e.clientWidth = w; e.clientHeight = h; } };
  size(cw, ch);
  const flushRO = () => rec.ro.forEach(o => o.cb([], o));
  const setDpr = d => { v.win.devicePixelRatio = d; };
  const resMq = () => rec.mq.filter(m => /resolution/.test(m.media));
  const fireDpr = () => resMq().forEach(m => m._l.slice().forEach(f => f({ type: 'change', matches: false, media: m.media })));
  const lastT = () => { const a = rec.ctx.filter(c => c[0] === 'setTransform').pop(); return a ? a[1] : null; };
  return { v, cv, rec, size, flushRO, setDpr, fireDpr, resMq, lastT };
}
const fitOK = (h, dpr, cw, ch, label) => {
  const { cv } = h, W = Math.round(cw * dpr), H = Math.round(ch * dpr), t = h.lastT();
  check(cv.width === W, `${label}: cv.width ${cv.width} must be round(${cw}*${dpr}) = ${W}`);
  check(cv.height === H, `${label}: cv.height ${cv.height} must be round(${ch}*${dpr}) = ${H}`);
  check(Math.abs(cv.width / cv.height - cw / ch) <= 1 / H + 1 / cw + 1e-9, `${label}: backing aspect ratio must equal CSS box`);
  check(t && t.length === 6 && Math.abs(t[0] - dpr) < 1e-9 && t[1] === 0 && t[2] === 0 && Math.abs(t[3] - dpr) < 1e-9 && t[4] === 0 && t[5] === 0, `${label}: last setTransform must be (${dpr},0,0,${dpr},0,0), got ${JSON.stringify(t)}`);
};
const draw = h => { h.v.$('tab-dashboard').click(); h.v.$('tab-replay').click(); };

// ---------- shared sizing at each dpr ----------
for (const [dpr, cw, ch] of [[1, 640, 140], [1.25, 333, 141], [1.5, 641, 139], [2, 520, 140]]) {
  const h = boot(dpr, cw, ch);
  if (!h.cv) { check(false, 'canvas #timeline missing'); continue; }
  draw(h);
  fitOK(h, dpr, cw, ch, `dpr ${dpr} ${cw}x${ch}`);
  const bad = h.rec.ctx.filter(c => (c[0] === 'scale' && c[1][0] !== c[1][1]) || (c[0] === 'setTransform' && (c[1][0] !== c[1][3] || c[1][1] !== 0 || c[1][2] !== 0)));
  check(bad.length === 0, `dpr ${dpr}: no non-uniform scaling allowed, got ${JSON.stringify(bad.slice(0, 2))}`);
  const fonts = h.rec.ctx.filter(c => c[0] === 'font').map(c => c[1][0]);
  check(fonts.length > 0 && fonts.every(f => /\b\d+(\.\d+)?px\b/.test(f)), `dpr ${dpr}: canvas font size must be in CSS px, got ${JSON.stringify([...new Set(fonts)])}`);
}
