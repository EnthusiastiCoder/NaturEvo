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

// ---------- (3) toggle button + sprite ----------
{
  const m = markup.match(/<button\b([^>]*\bid\s*=\s*["']theme-toggle["'][^>]*)>([\s\S]*?)<\/button>/);
  check(!!m, '#theme-toggle button must exist');
  if (m) {
    check(!!(attr(m[1], 'aria-label') || '').trim(), '#theme-toggle needs aria-label');
    const d = attr(m[1], 'aria-describedby');
    check(!!d && new RegExp('role\\s*=\\s*["\']tooltip["\'][^>]*>|id\\s*=\\s*["\']' + d + '["\'][^>]*role\\s*=\\s*["\']tooltip["\']').test(markup) && new RegExp('id\\s*=\\s*["\']' + d + '["\']').test(markup), '#theme-toggle aria-describedby must reference an existing role=tooltip element, got ' + d);
    check(/type\s*=\s*["']button["']/.test(m[1]), '#theme-toggle must be type="button"');
    check(/<use\b[^>]*href\s*=\s*["']#i-(sun|moon)["']/.test(m[2]), '#theme-toggle must use #i-sun or #i-moon');
  }
  for (const s of ['i-sun', 'i-moon', 'i-palette']) check(new RegExp('<symbol\\b[^>]*\\bid\\s*=\\s*["\']' + s + '["\']').test(html), 'sprite must define symbol ' + s);
}

// ---------- (4) initial theme precedence: localStorage > prefers-color-scheme > dark ----------
{
  const cases = [
    ['no storage, matchMedia dark', {}, 'dark', 'dark'],
    ['no storage, matchMedia light', {}, 'light', 'light'],
    ['no storage, no matchMedia', {}, 'absent', 'dark'],
    ['no storage, matchMedia throws', {}, 'throws', 'dark'],
    ['storage light beats matchMedia dark', { 'jv-theme': 'light' }, 'dark', 'light'],
    ['storage dark beats matchMedia light', { 'jv-theme': 'dark' }, 'light', 'dark'],
    ['invalid storage ignored -> matchMedia light', { 'jv-theme': 'purple' }, 'light', 'light'],
    ['invalid storage ignored -> default dark', { 'jv-theme': '' }, 'absent', 'dark'],
  ];
  for (const [name, storage, mm, want] of cases) {
    const v = safeLoad({ storage, matchMedia: mm }, name); if (!v) continue;
    check(theme(v) === want, `initial theme (${name}) must be ${want}, got ${theme(v)}`);
    check(v.win.__jv && v.win.__jv.theme && typeof v.win.__jv.theme.get === 'function' && v.win.__jv.theme.get() === want, `__jv.theme.get() (${name}) must be ${want}`);
  }
}

// ---------- (5) toggle, persistence, storage failure ----------
{
  const v = safeLoad({ storage: {}, matchMedia: 'dark' }, 'toggle');
  if (v) {
    const t = v.$('theme-toggle');
    check(!!t, '#theme-toggle element must exist in DOM');
    if (t) {
      t.click(); check(theme(v) === 'light', 'click toggles dark -> light, got ' + theme(v));
      check(v.store.get('jv-theme') === 'light', 'toggle persists localStorage jv-theme=light, got ' + v.store.get('jv-theme'));
      t.click(); check(theme(v) === 'dark', 'second click toggles light -> dark, got ' + theme(v));
      check(v.store.get('jv-theme') === 'dark', 'toggle persists jv-theme=dark');
    }
    if (v.win.__jv && v.win.__jv.theme) { v.win.__jv.theme.set('light'); check(theme(v) === 'light' && v.win.__jv.theme.get() === 'light', '__jv.theme.set("light") applies'); }
    else check(false, 'window.__jv.theme must exist');
  }
  for (const mode of ['throw-access', 'throw-ops']) {
    const w = safeLoad({ storage: mode, matchMedia: 'light' }, 'storage ' + mode); if (!w) continue;
    check(theme(w) === 'light', `${mode}: initial theme falls back to matchMedia light, got ${theme(w)}`);
    const t = w.$('theme-toggle');
    check(!!t, `${mode}: #theme-toggle must exist`);
    if (t) {
      let err = null; try { t.click(); } catch (e) { err = e; }
      check(!err, `${mode}: toggle click must not throw (${err && err.message})`);
      check(theme(w) === 'dark', `${mode}: toggle still works for the session, got ${theme(w)}`);
      try { t.click(); } catch (e) { err = e; }
      check(theme(w) === 'light', `${mode}: second toggle works`);
    }
    let e2 = null; try { w.win.__jv.theme.accent.set('#7b3fa0'); } catch (e) { e2 = e; }
    check(!e2 && cssVar(w, '--accent').toLowerCase() === '#7b3fa0', `${mode}: accent.set still applies without storage (${e2 && e2.message})`);
  }
}

// ---------- (6) accent presets, custom, reset, persistence ----------
{
  const box = (markup.match(/<details\b[^>]*\bid\s*=\s*["']accent-box["'][\s\S]*?<\/details>/) || [''])[0];
  check(!!box, 'details#accent-box must exist');
  check(/<button\b[^>]*\bid\s*=\s*["']accent-btn["'][^>]*aria-label|<button\b[^>]*aria-label[^>]*\bid\s*=\s*["']accent-btn["']/.test(markup), '#accent-btn needs aria-label');
  check(/id\s*=\s*["']accent-btn["'][\s\S]{0,400}#i-palette/.test(markup), '#accent-btn must use #i-palette');
  check(/<input\b[^>]*\bid\s*=\s*["']accent-custom["'][^>]*>/.test(markup) && /type\s*=\s*["']color["']/.test((markup.match(/<input\b[^>]*\bid\s*=\s*["']accent-custom["'][^>]*>/) || [''])[0]), '#accent-custom must be <input type="color">');
  check(/\bid\s*=\s*["']accent-reset["']/.test(markup), '#accent-reset must exist');
  const sw = [...box.matchAll(/<button\b([^>]*)>/g)].map(m => m[1]).filter(a => attr(a, 'id') !== 'accent-reset');
  check(sw.length >= 6, 'accent-box needs >= 6 preset swatch buttons, found ' + sw.length);
  check(sw.every(a => (attr(a, 'aria-label') || '').trim().length > 0), 'every swatch needs an aria-label');
  const colours = sw.map(a => (a.match(/#[0-9a-fA-F]{6}\b/) || [''])[0].toLowerCase()).filter(Boolean);
  check(new Set(colours).size >= 6, 'swatches must carry 6 distinct #rrggbb values (e.g. data-accent), found ' + colours.join(','));
  check(colours.includes('#2e7d4f'), 'a swatch must be jungle green #2e7d4f');

  const v = safeLoad({ storage: {}, matchMedia: 'dark' }, 'accent');
  if (v) {
    const T = v.win.__jv && v.win.__jv.theme;
    check(!!T && !!T.accent, '__jv.theme.accent must exist');
    if (T && T.accent) {
      check(String(T.accent.get()).toLowerCase() === '#2e7d4f', 'default accent must be #2e7d4f, got ' + T.accent.get());
      check(same(cssVar(v, '--accent'), '#2e7d4f'), '--accent at load must be #2e7d4f, got ' + cssVar(v, '--accent'));
      T.accent.set('#7b3fa0');
      check(same(cssVar(v, '--accent'), '#7b3fa0'), 'accent.set applies --accent live, got ' + cssVar(v, '--accent'));
      check(String(v.store.get('jv-accent') || '').toLowerCase() === '#7b3fa0', 'accent persisted as jv-accent, got ' + v.store.get('jv-accent'));
      const c = v.$('accent-custom');
      if (c) { c.value = '#336699'; c.fire('input'); c.fire('change'); check(same(cssVar(v, '--accent'), '#336699') && String(T.accent.get()).toLowerCase() === '#336699', 'custom colour input sets accent, got ' + cssVar(v, '--accent')); }
      else check(false, '#accent-custom element must exist in DOM');
      const r = v.$('accent-reset');
      if (r) { r.click(); check(same(cssVar(v, '--accent'), '#2e7d4f') && String(T.accent.get()).toLowerCase() === '#2e7d4f', 'reset restores #2e7d4f, got ' + cssVar(v, '--accent')); const sv = v.store.get('jv-accent'); check(!sv || same(sv, '#2e7d4f'), 'reset clears or resets persisted jv-accent, got ' + sv); }
      else check(false, '#accent-reset element must exist in DOM');
      T.accent.set('not-a-colour');
      check(same(cssVar(v, '--accent'), '#2e7d4f'), 'invalid accent value is ignored');
    }
  }
  const p = safeLoad({ storage: { 'jv-accent': '#7b3fa0', 'jv-theme': 'light' }, matchMedia: 'dark' }, 'persisted');
  if (p) { check(same(cssVar(p, '--accent'), '#7b3fa0'), 'persisted jv-accent applied on load, got ' + cssVar(p, '--accent')); check(theme(p) === 'light', 'persisted theme applied on load'); }
  const q = safeLoad({ storage: { 'jv-accent': 'javascript:1' } }, 'bad persisted');
  if (q) check(same(cssVar(q, '--accent'), '#2e7d4f'), 'invalid persisted accent ignored, got ' + cssVar(q, '--accent'));
}

// ---------- (7) accent-fg by luminance, accent-text fallback ----------
{
  const v = safeLoad({ storage: {}, matchMedia: 'dark' }, 'contrast');
  const T = v && v.win.__jv && v.win.__jv.theme;
  if (T && T.contrast) {
    check(Math.abs(T.contrast('#000000', '#ffffff') - 21) < 0.01, 'contrast(black, white) must be 21, got ' + T.contrast('#000000', '#ffffff'));
    check(Math.abs(T.contrast('#777777', '#ffffff') - cr('#777777', '#ffffff')) < 0.02, 'contrast must follow WCAG relative luminance');
  } else check(false, '__jv.theme.contrast(a,b) must exist');
  if (v && T && T.accent) {
    const presets = ['#2e7d4f', '#7b3fa0', '#2563eb', '#c2410c', '#be185d', '#0e7490', '#b45309', '#ffff00', '#000080', '#ffffff', '#000000', '#808080', '#ffd700', '#00ff00'];
    for (const a of presets) {
      T.accent.set(a);
      const fg = cssVar(v, '--accent-fg');
      check(!!hex(fg), `--accent-fg must be set for ${a}, got "${fg}"`);
      if (!hex(fg)) continue;
      const best = cr(a, '#000000') >= cr(a, '#ffffff') ? '#000000' : '#ffffff';
      check(same(fg, best), `--accent-fg for ${a} must be ${best} (higher contrast), got ${fg}`);
      if (['#2e7d4f', '#7b3fa0', '#2563eb', '#ffff00', '#000080', '#c2410c', '#be185d', '#0e7490'].includes(a)) check(cr(a, fg) >= 4.5, `on-accent contrast for ${a} must be >= 4.5, got ${cr(a, fg).toFixed(2)}`);
    }
    // accent-text fallback: accent vs --bg < 3 => derived --accent-text >= 3 against --bg, per theme
    for (const [th, bad] of [['dark', '#000080'], ['dark', '#101010'], ['light', '#ffff00'], ['light', '#f5f5dc']]) {
      T.set(th); T.accent.set(bad);
      const bg = cssVar(v, '--bg'), at = cssVar(v, '--accent-text');
      check(!!hex(bg), `${th}: --bg must resolve to a colour, got "${bg}"`);
      if (!hex(bg)) continue;
      check(cr(bad, bg) < 3, `test setup: ${bad} on ${th} --bg should be < 3:1`);
      check(!!hex(at) && cr(at, bg) >= 3, `${th}: --accent-text for low-contrast ${bad} must reach >= 3:1 on --bg, got "${at}" (${hex(at) ? cr(at, bg).toFixed(2) : 'n/a'})`);
      if (hex(at)) check(th === 'dark' ? lum(at) > lum(bad) : lum(at) < lum(bad), `${th}: --accent-text must be ${th === 'dark' ? 'lighter' : 'darker'} than ${bad}`);
    }
    T.set('dark'); T.accent.set('#2e7d4f');
    const at = cssVar(v, '--accent-text'), bg = cssVar(v, '--bg');
    check(!!hex(at) && hex(bg) && cr(at, bg) >= 3, 'accent-text is always defined and >= 3:1 on --bg (jungle green, dark)');
    check(/--accent-text/.test(css) && /var\(\s*--accent-text\s*\)/.test(css), 'CSS must use var(--accent-text) for accent-coloured text/borders');
    check(/var\(\s*--accent-fg\s*\)/.test(css), 'CSS must use var(--accent-fg) for text on accent backgrounds');
  }
}

// ---------- (8) charts redraw on theme / accent change ----------
{
  const v = safeLoad({ storage: {}, matchMedia: 'dark' }, 'charts');
  if (v && v.win.__jv && v.win.__jv.theme) {
    const T = v.win.__jv.theme;
    check(v.paints.length > 0, 'timeline must paint with fillStyle/strokeStyle at load');
    const cols = th => new Set(TOK.concat(['--accent', '--accent-text']).map(t => (tokens(th)[t] || '').toLowerCase()).filter(Boolean));
    const painted = () => new Set(v.paints.map(p => p.toLowerCase()));
    // timeline must use token colours (not hard-coded) : something painted equals a dark token
    check([...painted()].some(p => [...cols('dark')].some(c => same(p, c))), 'initial paints must use dark token colours; painted: ' + [...painted()].slice(0, 6).join(','));
    v.paints.length = 0; v.events.length = 0;
    T.set('light');
    check(v.events.includes('jv-themechange'), 'theme change must dispatch jv-themechange, events: ' + v.events.join(','));
    await tick();
    check(v.paints.length > 0, 'charts must redraw (canvas paint) after theme change');
    const lightOnly = [...cols('light')].filter(c => ![...cols('dark')].some(d => same(c, d)));
    check([...painted()].some(p => lightOnly.some(c => same(p, c))), 'redraw after theme change must use light token colours; painted: ' + [...painted()].slice(0, 6).join(','));
    check(![...painted()].some(p => [...cols('dark')].filter(d => !lightOnly.some(c => same(c, d))).some(d => same(p, d) && !same(p, '#2e7d4f'))), 'redraw after theme change must not use stale dark token colours');
    v.paints.length = 0; v.events.length = 0;
    T.accent.set('#7b3fa0');
    check(v.events.includes('jv-themechange'), 'accent change must dispatch jv-themechange, events: ' + v.events.join(','));
    await tick();
    check(v.paints.length > 0, 'charts must redraw after accent change');
    check([...painted()].some(p => same(p, '#7b3fa0') || /rgba?\(\s*123\s*,\s*63\s*,\s*160/.test(p)) || [...painted()].some(p => same(p, cssVar(v, '--accent-text'))), 'redraw after accent change must paint with the new accent; painted: ' + [...painted()].slice(0, 6).join(','));
  } else check(false, 'charts: __jv.theme missing');
  // dashboard histogram is DOM/CSS based: must not hard-code colours in JS
  const js = [...html.matchAll(/<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/g)].map(m => m[1]).join('\n').replace(/\b(SP|ZN)\s*=\s*\[\[[\s\S]*?\]\]\s*;/g, '');
  check(/getComputedStyle/.test(js) && /jv-themechange/.test(js), 'inline JS must read tokens via getComputedStyle and handle jv-themechange');
}

// ---------- (9) file:// makes no fetch, even when theming ----------
{
  const v = safeLoad({ protocol: 'file:', storage: 'throw-access', matchMedia: 'light' }, 'file');
  if (v) {
    try { v.$('theme-toggle') && v.$('theme-toggle').click(); (v.win.__jv && v.win.__jv.theme ? v.win.__jv.theme.accent.set('#7b3fa0') : check(false, 'file://: __jv.theme missing')); } catch (e) { check(false, 'file:// theming threw: ' + e.message); }
    await tick();
    check(v.fetchCalls.length === 0, 'file:// must not fetch anything, got ' + v.fetchCalls.join(','));
  }
}
