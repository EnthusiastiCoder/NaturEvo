// Theme + accent requirements (plans/2026-10-20_viewer-redesign-v2.md, "Addendum (user decisions):
// themes and accent"). Static CSS/markup checks plus behaviour via the stub-DOM harness. No dependencies.
import { html, load, makeRun, tick, runner } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_theme_check');

const styleRaw = (html.match(/<style[^>]*>([\s\S]*?)<\/style>/) || [, ''])[1];
const css = styleRaw.replace(/\/\*[\s\S]*?\*\//g, '');
const markup = html.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<style[\s\S]*?<\/style>/g, '');
const attr = (tag, name) => { const m = tag.match(new RegExp('\\b' + name + '\\s*=\\s*(?:"([^"]*)"|\'([^\']*)\')', 'i')); return m ? (m[1] ?? m[2]) : null; };
const rules = [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)].map(m => ({ sels: m[1].split(',').map(s => s.trim().replace(/\s+/g, ' ')), body: m[2] }));
const decls = body => body.split(';').map(d => { const k = d.indexOf(':'); return k < 0 ? null : [d.slice(0, k).trim().toLowerCase(), d.slice(k + 1).trim()]; }).filter(Boolean);
const themeSel = t => s => new RegExp('^:root\\[data-theme=["\']?' + t + '["\']?\\]$').test(s);
const tokens = t => Object.fromEntries(rules.filter(r => r.sels.some(themeSel(t))).flatMap(r => decls(r.body)));
const TOK = ['--bg', '--card', '--card-2', '--border', '--fg', '--muted', '--good', '--bad', '--warn', '--ring'];

// colour maths (independent of the page)
const hex = c => { c = String(c || '').trim().toLowerCase(); let m = c.match(/^#([0-9a-f]{6})$/); if (m) return [0, 2, 4].map(i => parseInt(m[1].slice(i, i + 2), 16)); m = c.match(/^#([0-9a-f]{3})$/); if (m) return [...m[1]].map(x => parseInt(x + x, 16)); m = c.match(/^rgba?\(\s*(\d+)[,\s]+(\d+)[,\s]+(\d+)/); return m ? [+m[1], +m[2], +m[3]] : null; };
const lum = c => { const [r, g, b] = hex(c).map(v => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }); return 0.2126 * r + 0.7152 * g + 0.0722 * b; };
const cr = (a, b) => { const x = lum(a), y = lum(b); return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05); };
const same = (a, b) => { const x = hex(a), y = hex(b); return !!x && !!y && x.join() === y.join(); };

const safeLoad = (opts, label) => { try { return load({ run: makeRun(120), ...opts }); } catch (e) { check(false, label + ': page script threw at load: ' + e.message); return null; } };
const theme = v => v.root.getAttribute('data-theme') || v.root.dataset.theme || null;
const cssVar = (v, n) => String(v.win.getComputedStyle(v.root).getPropertyValue(n) || '').trim();

// ---------- (1) two token sets ----------
const dark = tokens('dark'), light = tokens('light');
{
  for (const [n, set] of [['dark', dark], ['light', light]]) {
    const miss = TOK.filter(t => !set[t] || !set[t].trim());
    check(miss.length === 0, `:root[data-theme="${n}"] must define tokens: missing ${miss.join(' ')}`);
    check(hex(set['--bg']) && hex(set['--fg']) && cr(set['--bg'], set['--fg']) >= 4.5, `${n}: --fg on --bg contrast must be >= 4.5`);
    check(hex(set['--bg']) && hex(set['--muted']) && cr(set['--bg'], set['--muted']) >= 3, `${n}: --muted on --bg contrast must be >= 3`);
  }
  const same_ = ['--bg', '--card', '--card-2', '--border', '--fg', '--muted'].filter(t => dark[t] && light[t] && same(dark[t], light[t]));
  check(Object.keys(dark).length > 0 && Object.keys(light).length > 0 && same_.length === 0, 'dark and light must have distinct values for: ' + same_.join(' '));
  check(dark['--bg'] && light['--bg'] && lum(dark['--bg']) < lum(light['--bg']), 'dark --bg must be darker than light --bg');
}

// ---------- (2) no colour literal outside token blocks ----------
{
  const colour = /#[0-9a-fA-F]{3,8}\b|\brgba?\s*\(|\bhsla?\s*\(/;
  const isTokenBlock = r => r.sels.every(s => /^:root(\[data-theme=["']?\w+["']?\])?$/.test(s));
  const bad = [];
  for (const r of rules) if (!isTokenBlock(r)) for (const [k, v] of decls(r.body)) if (colour.test(v)) bad.push(r.sels[0] + ' { ' + k + ': ' + v + ' }');
  check(bad.length === 0, 'colour literals outside :root / :root[data-theme] blocks: ' + bad.slice(0, 5).join(' | '));
  check(!/\bstyle\s*=\s*["'][^"']*(#[0-9a-f]{3,8}|rgb)/i.test(markup), 'no inline colour styles in markup');
}
