# Viewer layout browser checklist

Run with chrome-devtools MCP (read `prompts/chrome-devtools-mcp.md` first; max 6 tabs, close extras after).
Served: `cd sim && cargo run --release -- serve --port 8099` (PATH must include `/c/Users/enthu/.cargo/bin`), open `http://127.0.0.1:8099/`.
File variant: open `file:///C:/Users/enthu/Projects/NaturEvo/viewer/index.html`.

Every snippet is passed to `evaluate_script` as a function body expression and returns `{pass, details}`. Expected: `pass === true`.
Resize with `resize_page` (width, height) before each run. Wait ~300ms after resizing or opening a drawer (resize handler).

## 0. Helper (paste at the start of each snippet, or define once on `window.__chk` and reuse)

```js
() => {
  window.__chk = {
    tab: t => document.getElementById(t === 'r' ? 'tab-replay' : 'tab-dashboard').click(),
    closeAll: () => document.querySelectorAll('details[open]').forEach(d => d.removeAttribute('open')),
    open: id => { window.__chk.closeAll(); document.getElementById(id).setAttribute('open', ''); document.getElementById(id).dispatchEvent(new Event('toggle')); },
    vis: e => { const r = e.getBoundingClientRect(); const s = getComputedStyle(e); return r.width > 0 && r.height > 0 && s.visibility !== 'hidden' && s.display !== 'none'; },
  };
  return 'ok';
}
```

Note: if the page closes other drawers on toggle (one-open-at-a-time), `open` still works because it closes all first.

## 1. No page scroll (core)

Matrix: viewports 1920x1080, 1366x768, 1280x720, 800x600 x tabs Replay, Dashboard x states {closed, knob-box, log-box, more-box, batch-box}. Batch-box and knob-box exist only while `#run-controls` is visible (served mode); in file:// skip them. Total 4 x 2 x 5 = 40 served runs.
Change `TAB` and `STATE` (`'none'|'knob-box'|'log-box'|'more-box'|'batch-box'`) per run.

```js
() => {
  const TAB = 'r', STATE = 'none';
  const c = window.__chk; c.tab(TAB); c.closeAll();
  if (STATE !== 'none') c.open(STATE);
  const de = document.documentElement, b = document.body;
  const d = {
    vw: innerWidth, vh: innerHeight,
    docScrollH: de.scrollHeight, docScrollW: de.scrollWidth,
    bodyScrollH: b.scrollHeight, bodyScrollW: b.scrollWidth,
    winScrollY: (window.scrollTo(0, 99999), window.scrollY), winScrollX: (window.scrollTo(99999, 0), window.scrollX),
    state: STATE, tab: TAB,
  };
  window.scrollTo(0, 0);
  const pass = de.scrollHeight <= innerHeight && de.scrollWidth <= innerWidth
    && b.scrollHeight <= innerHeight && b.scrollWidth <= innerWidth && d.winScrollY === 0 && d.winScrollX === 0;
  return { pass, details: d };
}
```

Expected: `pass: true` for all 40. Also verify the opened drawer is actually visible and inside the viewport:

```js
() => {
  const STATE = 'log-box';   // or knob-box / more-box / batch-box
  const el = document.getElementById(STATE), r = el.getBoundingClientRect();
  const panel = [...el.children].filter(x => x.tagName !== 'SUMMARY').map(x => x.getBoundingClientRect());
  const inView = panel.every(p => p.height > 0 && p.top >= -1 && p.bottom <= innerHeight + 1 && p.left >= -1 && p.right <= innerWidth + 1);
  return { pass: el.open && inView, details: { open: el.open, panel: panel.map(p => ({ t: p.top, b: p.bottom, l: p.left, r: p.right })), vh: innerHeight, vw: innerWidth } };
}
```

Expected: `pass: true` (panel not cut off by viewport edge).

## 2. Closed by default (reload page first, then run)

```js
() => {
  const ids = ['knob-box', 'log-box', 'more-box', 'batch-box'];
  const served = !document.getElementById('run-controls').classList.contains('hidden');
  const d = {};
  ids.forEach(i => { const e = document.getElementById(i); d[i] = e ? { tag: e.tagName, open: e.open } : null; });
  const required = served ? ids : ['log-box', 'more-box'];
  const pass = required.every(i => d[i] && d[i].tag === 'DETAILS' && d[i].open === false)
    && document.getElementById('knob-box') && document.getElementById('knob-box').tagName === 'DETAILS';
  // visible text/number inputs while all closed: only #seed (served) and none in file://
  const vis = [...document.querySelectorAll('input')].filter(e => ['text', 'number', ''].includes(e.type) && e.getBoundingClientRect().width > 0 && getComputedStyle(e).display !== 'none' && e.offsetParent !== null).map(e => e.id);
  const okInputs = served ? (vis.length === 1 && vis[0] === 'seed') : vis.length === 0;
  return { pass: pass && okInputs, details: { served, drawers: d, visibleTextNumberInputs: vis } };
}
```

Expected: served -> visible text/number inputs exactly `["seed"]`; file:// -> `[]`. The only other default-visible inputs allowed are the `#speed` select, the `#scrub` range and (Dashboard tab only) the "Log scale" checkbox (type checkbox, not counted by the filter above).

## 3. No clipped inputs / labels (each viewport, each tab, drawers closed and each open)

```js
() => {
  const bad = [];
  document.querySelectorAll('input, select, button, label, summary, #outcome, #sum-level, #sum-zone, #sum-progress, .stat').forEach(e => {
    const r = e.getBoundingClientRect(), s = getComputedStyle(e);
    if (r.width === 0 || r.height === 0 || s.display === 'none' || s.visibility === 'hidden') return;
    if (e.scrollWidth > e.clientWidth + 1 && e.tagName !== 'SELECT') bad.push({ id: e.id || e.tagName, sw: e.scrollWidth, cw: e.clientWidth });
    if (r.right > innerWidth + 1 || r.left < -1) bad.push({ id: e.id || e.tagName, offscreen: [r.left, r.right] });
    if (s.textOverflow === 'ellipsis' && e.scrollWidth > e.clientWidth) bad.push({ id: e.id || e.tagName, ellipsis: true });
  });
  // seed shows a full 10 digit value
  const seed = document.getElementById('seed');
  let seedOk = true, seedD = null;
  if (seed && seed.offsetParent !== null) {
    const old = seed.value; seed.value = '4294967295';
    seedOk = seed.scrollWidth <= seed.clientWidth; seedD = { sw: seed.scrollWidth, cw: seed.clientWidth, widthCh: seed.getBoundingClientRect().width };
    seed.value = old;
  }
  // knob inputs >= 8ch (approx 8 * 7px = 56px)
  const knobs = [...document.querySelectorAll('#knobs input')].filter(e => e.getBoundingClientRect().width > 0);
  const narrow = knobs.filter(e => e.getBoundingClientRect().width < 56).map(e => e.id);
  return { pass: bad.length === 0 && seedOk && narrow.length === 0, details: { bad, seedOk, seedD, narrowKnobs: narrow } };
}
```

Run with `knob-box` open to cover knob inputs and labels (served mode). Expected: `bad: []`, `narrowKnobs: []`.

## 4. Inner scrolling (log, knobs, hist)

Log, after a run is loaded and navigated to event 150 (log-box open):

```js
() => {
  window.__jv.goto(150);
  const c = window.__chk; c.tab('r'); c.open('log-box');
  const L = document.getElementById('log');
  const before = { sh: L.scrollHeight, ch: L.clientHeight, st: L.scrollTop };
  L.scrollTop = 0; const t0 = L.scrollTop; L.scrollTop = 40; const t1 = L.scrollTop;
  const r = L.getBoundingClientRect();
  const de = document.documentElement;
  const pass = before.sh > before.ch && before.ch > 0 && t1 !== t0 && r.bottom <= innerHeight + 1 && de.scrollHeight <= innerHeight
    && getComputedStyle(L).overflowY !== 'visible';
  return { pass, details: { before, t0, t1, bottom: r.bottom, vh: innerHeight, pageScrollH: de.scrollHeight } };
}
```

Expected: `scrollHeight > clientHeight`, scrollTop changes, log bottom inside viewport, page does not grow. Also the current row is visible after opening (`.cur` centered):

```js
() => {
  const L = document.getElementById('log'), cur = L.querySelector('.cur');
  if (!cur) return { pass: false, details: 'no .cur row' };
  const a = L.getBoundingClientRect(), b = cur.getBoundingClientRect();
  return { pass: b.top >= a.top - 1 && b.bottom <= a.bottom + 1, details: { log: [a.top, a.bottom], cur: [b.top, b.bottom] } };
}
```

Knobs (served, knob-box open, at 800x600 and 1280x720):

```js
() => {
  const c = window.__chk; c.tab('r'); c.open('knob-box');
  const K = document.getElementById('knobs'), box = document.getElementById('knob-box');
  const panel = [...box.children].find(x => x.tagName !== 'SUMMARY');
  const rects = [...box.querySelectorAll('input')].map(i => i.getBoundingClientRect());
  const scroller = [K, panel, box].find(e => e && e.scrollHeight > e.clientHeight + 1 && ['auto', 'scroll'].includes(getComputedStyle(e).overflowY));
  const pb = (panel || K).getBoundingClientRect();
  const reset = document.getElementById('knobs-reset').getBoundingClientRect();
  const pass = pb.bottom <= innerHeight + 1 && document.documentElement.scrollHeight <= innerHeight
    && reset.top >= 0 && reset.bottom <= innerHeight + 1 && (scroller ? true : rects.every(r => r.bottom <= innerHeight + 1));
  // grid of 2-3 columns at >=1280
  const cols = getComputedStyle(K).gridTemplateColumns.split(' ').length;
  return { pass, details: { panelBottom: pb.bottom, vh: innerHeight, scrollerId: scroller && (scroller.id || scroller.tagName), resetTop: reset.top, cols, inputs: rects.length } };
}
```

Expected: `pass: true`; if content taller than the panel, a scroller exists (inner scroll), `knobs-reset` visible without scrolling (pinned at top); `cols` is 2-3 at >=1280 wide.

Dashboard hist (Dashboard tab, stats loaded: `window.STATS` set via file:// data or batch):

```js
() => {
  window.__chk.tab('d');
  const H = document.getElementById('hist'), D = document.getElementById('dstats');
  const hr = H.getBoundingClientRect(), cards = D.children.length;
  const s = getComputedStyle(H);
  const pass = hr.bottom <= innerHeight + 1 && hr.height > 40 && ['auto', 'scroll'].includes(s.overflowY) && cards === 4
    && document.documentElement.scrollHeight <= innerHeight && document.documentElement.scrollWidth <= innerWidth;
  return { pass, details: { histBottom: hr.bottom, histH: hr.height, overflowY: s.overflowY, cards, scrollH: H.scrollHeight, clientH: H.clientHeight } };
}
```

Expected: 4 stat cards in one row, `#hist` bounded and scrolls inside (`scrollHeight > clientHeight` at 800x600 when the histogram has many levels).

## 5. Timeline canvas fills its container (Replay, each viewport, plus after resizing)

```js
() => {
  window.__chk.tab('r'); window.__chk.closeAll();
  const cv = document.getElementById('timeline'), r = cv.getBoundingClientRect(), p = cv.parentElement.getBoundingClientRect();
  const cs = getComputedStyle(cv.parentElement);
  const inner = p.width - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight);
  const d = { canvasCssW: r.width, canvasCssH: r.height, backingW: cv.width, backingH: cv.height, parentInnerW: inner, parentH: p.height, vw: innerWidth, left: r.left, right: r.right };
  const pass = Math.abs(r.width - inner) <= 2 && Math.abs(cv.width - Math.round(cv.clientWidth)) <= 1
    && Math.abs(cv.height - Math.round(cv.clientHeight)) <= 1 && r.height >= 80 && r.bottom <= innerHeight + 1
    && innerWidth - r.right <= 40;   // no large gap on the right
  return { pass, details: d };
}
```

Expected: CSS width equals container inner width (no gap on the right), backing size equals CSS size (no blur), height >= 80, and the timeline takes the leftover height (no big empty area: `r.height >= 80`; at 1920x1080 expect it taller than 200).

Resize test (backing store follows `resize`), run twice with `resize_page` between (e.g. 1280x720 then 800x600) and compare:

```js
() => {
  const cv = document.getElementById('timeline');
  return { pass: Math.abs(cv.width - cv.clientWidth) <= 1 && Math.abs(cv.height - cv.clientHeight) <= 1, details: { w: cv.width, cw: cv.clientWidth, h: cv.height, ch: cv.clientHeight, vw: innerWidth } };
}
```

## 6. Summary strip and landing view (Replay, drawers closed, run loaded)

```js
() => {
  window.__chk.tab('r'); window.__chk.closeAll(); window.__jv.goto(60);
  const ids = ['summary', 'outcome', 'sum-level', 'sum-hp', 'sum-zone', 'sum-progress'];
  const t = {}; ids.forEach(i => { const e = document.getElementById(i); t[i] = e ? { vis: window.__chk.vis(e), text: e.textContent.trim().slice(0, 60) } : null; });
  const sr = document.getElementById('summary').getBoundingClientRect();
  const pass = ids.every(i => t[i] && t[i].vis && (i === 'summary' || t[i].text.length > 0))
    && /in progress|WON|LOST/.test(t.outcome.text) && /60/.test(t['sum-progress'].text) && sr.top < 120;
  return { pass, details: { t, summaryTop: sr.top } };
}
```

Expected: summary visible without scrolling, near the top (below the ~44px top bar).

## 7. file:// variant

Open the file URL, then at each of the 4 viewports x 2 tabs, run sections 1 (states `none`, `log-box`, `more-box`), 2, 3, 5, 6, plus:

```js
() => {
  const rc = document.getElementById('run-controls');
  const hidden = getComputedStyle(rc).display === 'none';
  const visInputs = [...document.querySelectorAll('input,select')].filter(e => e.offsetParent !== null).map(e => e.id);
  const pass = hidden && location.protocol === 'file:' && visInputs.filter(i => !/log/i.test(i)).sort().join() === 'scrub,speed';  // v2: dashboard log-scale checkbox allowed
  return { pass, details: { protocol: location.protocol, hidden, visInputs, runLoaded: !!window.RUN, statsLoaded: !!window.STATS } };
}
```

Expected: `visInputs` is exactly `["scrub","speed"]` (plus the log-scale checkbox while on Dashboard); replay renders (summary populated) and dashboard shows 4 cards from data/stats.js; no network requests (check `list_network_requests`: no `/api/` calls).

## 8. Served-mode behaviour unchanged (spot check, 1280x720)

1. Reload; `#run-controls` becomes visible; knob-box shows 'Tuning' toggle; type `4294967295` in `#seed`; click `#run-btn`; summary updates seed/outcome; no page scroll.
2. Open `#batch-box`, set `#batch-runs` to 200, click `#batch-btn`; Dashboard tab becomes active, 4 cards; `#run-status` text shows then clears.
3. Open `#knob-box`, change one knob, click `#knobs-reset` -> value returns to default.
4. Click `#random-seed` -> `#seed` gets a number 0..4294967295.
5. Console has no errors (`list_console_messages`).

## 9. Tooltips and info buttons (v2; each viewport, both tabs; served and file://)

Hover the info button (MCP `hover`), then:

```js
() => {
  const b = document.querySelector('.info'); if (!b) return { pass: false, details: 'no .info button' };
  const tip = document.getElementById(b.getAttribute('aria-describedby'));
  const r = tip.getBoundingClientRect(), s = getComputedStyle(tip);
  const shown = r.width > 0 && r.height > 0 && s.visibility !== 'hidden' && s.display !== 'none' && +s.opacity > 0;
  const inView = r.left >= -1 && r.top >= -1 && r.right <= innerWidth + 1 && r.bottom <= innerHeight + 1;
  return { pass: shown && tip.textContent.trim().length > 0 && inView, details: { shown, text: tip.textContent.trim().slice(0, 80), rect: [r.left, r.top, r.right, r.bottom], vw: innerWidth, vh: innerHeight } };
}
```

Expected: `pass: true` while hovering. Repeat for one info button in each container: a KPI (Dashboard), a toolbar control, a replay summary item, a knob (knob-box open, served), and the first and last `.info` in DOM order (viewport edges).

Keyboard: move the mouse away, press `Tab` until an `.info` button is focused, run the same snippet (tooltip visible by focus alone). Then press `Escape` and run:

```js
() => {
  const a = document.activeElement, tip = a && a.getAttribute && document.getElementById(a.getAttribute('aria-describedby') || '-');
  if (!tip) return { pass: false, details: 'active element is not an info button' };
  const s = getComputedStyle(tip), r = tip.getBoundingClientRect();
  return { pass: s.visibility === 'hidden' || s.display === 'none' || +s.opacity === 0 || r.width === 0, details: { vis: s.visibility, display: s.display, opacity: s.opacity } };
}
```

Expected: hidden after Escape (button may stay focused).

Not clipped by overflow (run on both tabs and with `knob-box` open):

```js
() => {
  const bad = [];
  document.querySelectorAll('.info').forEach(b => {
    const tip = document.getElementById(b.getAttribute('aria-describedby'));
    if (!tip) { bad.push({ id: b.getAttribute('aria-label'), missingTip: true }); return; }
    if (b.getBoundingClientRect().width === 0) return;
    b.focus();
    const r = tip.getBoundingClientRect();
    for (let p = tip.parentElement; p && p !== document.documentElement; p = p.parentElement) {
      const s = getComputedStyle(p);
      if (/(hidden|auto|scroll|clip)/.test(s.overflowX + s.overflowY)) {
        const pr = p.getBoundingClientRect();
        if (r.width > 0 && (r.left < pr.left - 1 || r.right > pr.right + 1 || r.top < pr.top - 1 || r.bottom > pr.bottom + 1)) bad.push({ id: b.getAttribute('aria-label'), clippedBy: p.id || p.className });
      }
    }
    b.blur();
  });
  return { pass: bad.length === 0, details: bad.slice(0, 10) };
}
```

Expected: `bad: []`. Every icon-only button has aria-label and a tooltip:

```js
() => {
  const bad = [...document.querySelectorAll('button')].filter(b => !b.textContent.trim() && (!b.getAttribute('aria-label') || !document.getElementById(b.getAttribute('aria-describedby') || '-'))).map(b => b.id || b.className);
  return { pass: bad.length === 0, details: bad };
}
```

## 10. Consistent control height and icon size (v2; each tab, each popover open)

All visible `.btn`, `.input`, `select` have equal `offsetHeight` == `--control-h` (+-1px):

```js
() => {
  const h = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--control-h'));
  const els = [...document.querySelectorAll('.btn,.input,select')].filter(e => e.offsetParent !== null && e.getBoundingClientRect().width > 0);
  const bad = els.filter(e => Math.abs(e.offsetHeight - h) > 1).map(e => ({ id: e.id || e.className, h: e.offsetHeight }));
  return { pass: els.length > 0 && bad.length === 0, details: { controlH: h, count: els.length, bad } };
}
```

Expected: `bad: []`, `controlH` 32. All icons equal `--icon`:

```js
() => {
  const ic = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--icon'));
  const els = [...document.querySelectorAll('svg.icon')].filter(e => e.getBoundingClientRect().width > 0);
  const bad = els.filter(e => { const r = e.getBoundingClientRect(); return Math.abs(r.width - ic) > 0.5 || Math.abs(r.height - ic) > 0.5; }).map(e => ({ use: e.querySelector('use') && e.querySelector('use').getAttribute('href'), w: e.getBoundingClientRect().width }));
  return { pass: els.length > 0 && bad.length === 0, details: { iconPx: ic, count: els.length, bad } };
}
```

Expected: `bad: []` (any exception must be documented in viewer/STYLE.md). Eyeball: consistent Lucide stroke style, no text glyph or emoji outside the small hero/monster/zone sprites.

## 11. Dashboard log-scale toggle (v2; Dashboard tab, skewed stats such as data/stats.js)

```js
() => {
  window.__chk.tab('d');
  const cb = document.querySelector('#dashboard input[type=checkbox]');
  if (!cb) return { pass: false, details: 'no checkbox' };
  const hs = () => [...document.querySelectorAll('#hist .bar, #hist rect')].map(e => Math.round(e.getBoundingClientRect().height));
  const minPos = a => Math.min(...a.filter(x => x > 0));
  const first = { checked: cb.checked, h: hs() };
  cb.click();
  const second = { checked: cb.checked, h: hs() };
  cb.click();
  const log = first.checked ? first : second, lin = first.checked ? second : first;
  const pass = first.checked !== second.checked && first.h.length >= 2 && first.h.length <= 20
    && JSON.stringify(first.h) !== JSON.stringify(second.h) && minPos(log.h) > minPos(lin.h);
  return { pass, details: { first, second } };
}
```

Expected: toggling changes bar heights; log mode makes the smallest non-zero bar taller than in linear mode; default OFF unless max/median > 20, then ON (data/stats.js is skewed: expect ON); at most 20 bars; hover/focus a bar shows its value tooltip; no page scroll after toggling.

## 12. Screenshots (save under `research/screenshots/viewer-redesign/`, do not commit)

Use `take_screenshot` with `filePath`:

| File | Viewport | Tab | State |
|---|---|---|---|
| 1920x1080-replay-closed.png | 1920x1080 | Replay | all closed |
| 1920x1080-dashboard.png | 1920x1080 | Dashboard | all closed |
| 1366x768-replay-closed.png | 1366x768 | Replay | all closed |
| 1366x768-replay-log.png | 1366x768 | Replay | log-box open |
| 1366x768-replay-more.png | 1366x768 | Replay | more-box open |
| 1280x720-replay-knobs.png | 1280x720 | Replay | knob-box open |
| 1280x720-replay-batch.png | 1280x720 | Replay | batch-box open |
| 1280x720-dashboard.png | 1280x720 | Dashboard | all closed |
| 800x600-replay-closed.png | 800x600 | Replay | all closed |
| 800x600-replay-knobs.png | 800x600 | Replay | knob-box open |
| 800x600-replay-log.png | 800x600 | Replay | log-box open |
| 800x600-dashboard.png | 800x600 | Dashboard | all closed |
| file-1280x720-replay.png | 1280x720 | Replay | file:// (no run controls) |
| file-1280x720-dashboard.png | 1280x720 | Dashboard | file:// |
| 1920x1080-dashboard-kpi.png | 1920x1080 | Dashboard | KPI row close-up (element screenshot of `#dstats`) |
| 1280x720-dashboard-log.png | 1280x720 | Dashboard | log scale toggled away from its default |
| 1280x720-tooltip-hover.png | 1280x720 | Dashboard | KPI info tooltip open via hover |
| 1280x720-tooltip-focus.png | 1280x720 | Replay | summary/toolbar info tooltip open via keyboard focus (focus ring visible) |
| 1280x720-knobs-tooltip.png | 1280x720 | Replay | knob-box open with one knob info tooltip open |
| 1366x768-dashboard.png | 1366x768 | Dashboard | all closed |
| 800x600-dashboard-kpi.png | 800x600 | Dashboard | all closed, KPI row wrapping check |

Visually confirm in each: no page scrollbar, nothing clipped, no big empty area, timeline reaches the right edge, active tab clear, focus ring visible on Tab key press (check once).

## 13. Independent visual review gate (v2, final step, after every check above passes)

Spawn a FRESH reviewer agent (sonnet, no access to the implementer's reasoning or source files). Give it ONLY the screenshots from section 12 and the verbatim requirement text from the plan ("cognitive load ... bird eye view ... without scrolling", "the layout is ugly", "info buttons and tool tip texts", "proper lucide icons", "good ui, not the game ui but the dashboard"). It returns a scored review (1-5) of: visual hierarchy, density/cognitive load, consistency (spacing, control height, icon size, colours), readability of the skewed histogram, game-like styling, clipped/overlapping/empty areas, tooltip legibility, plus PASS/FAIL and concrete fixes. Any FAIL or any score below 4: the implementer fixes, re-shoots section 12, and a NEW fresh reviewer re-reviews. Done only on PASS. Keep the review text with the run notes; do not commit screenshots.

## 14. Themes and accent (v2 addendum: light + dark, customisable accent)

Snippets reuse `window.__chk` (section 0). Set a theme/accent with the UI (`#theme-toggle`, `#accent-btn` -> `#accent-box`) or `window.__jv.theme.set('light'|'dark')` / `window.__jv.theme.accent.set('#rrggbb')`.

### 14.1 Repeat in BOTH themes

Run each in dark, click `#theme-toggle` (`document.documentElement.dataset.theme` flips), run again in light. All must pass in both:

- Section 1 (no page scroll): all 40 combinations in dark and in light = 80 served runs. The theme must not change layout size.
- Section 9 (tooltips): hover and focus tooltip visible, inside viewport, not clipped, Escape hides; include the tooltips on `#theme-toggle` and `#accent-btn`.
- Section 10 (control height and icon size): `#theme-toggle` and `#accent-btn` have `offsetHeight` == `--control-h` (32) and their `svg.icon` == `--icon`; repeat with `#accent-box` open (swatches, `#accent-custom`, `#accent-reset` included).
- Section 3 (no clipped inputs) with `#accent-box` open; the popover is fully inside the viewport at 800x600 and causes no page scroll.

Theme state snippet (run after each toggle):

```js
() => {
  const r = document.documentElement, t = r.dataset.theme, cs = getComputedStyle(r);
  const names = ['--bg','--card','--card-2','--border','--fg','--muted','--good','--bad','--warn','--ring','--accent','--accent-fg','--accent-text'];
  const vals = Object.fromEntries(names.map(n => [n, cs.getPropertyValue(n).trim()]));
  let stored; try { stored = localStorage.getItem('jv-theme'); } catch (e) { stored = 'storage-error'; }
  const pass = (t === 'dark' || t === 'light') && names.every(n => vals[n]);
  return { pass, details: { theme: t, stored, vals } };
}
```

Expected: `pass: true`; `stored` equals `theme` (or `'storage-error'` on file://); toggling twice returns to the starting theme.

### 14.2 Contrast, both themes

```js
() => {
  const rgb = s => (s.match(/[\d.]+/g) || []).slice(0, 3).map(Number);
  const lum = c => { const [r, g, b] = c.map(v => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }); return 0.2126 * r + 0.7152 * g + 0.0722 * b; };
  const cr = (a, b) => { const x = lum(a), y = lum(b); return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05); };
  const tok = n => { const p = document.createElement('i'); p.style.color = 'var(' + n + ')'; document.body.appendChild(p); const c = rgb(getComputedStyle(p).color); p.remove(); return c; };
  const d = { fgOnBg: cr(tok('--fg'), tok('--bg')), fgOnCard: cr(tok('--fg'), tok('--card')), mutedOnCard: cr(tok('--muted'), tok('--card')), accentTextOnBg: cr(tok('--accent-text'), tok('--bg')), fgOnAccent: cr(tok('--accent-fg'), tok('--accent')) };
  return { pass: d.fgOnBg >= 4.5 && d.fgOnCard >= 4.5 && d.mutedOnCard >= 3 && d.accentTextOnBg >= 3 && d.fgOnAccent >= 4.5, details: d };
}
```

Expected: `pass: true` in dark and light, for the default accent and for each accent in 14.3.

### 14.3 Accent change live (no reload), both themes

For each accent in `#2e7d4f` (default), `#7b3fa0` (purple), `#ffff00` (very light yellow), `#000080` (navy), in dark then light: set it (one via a preset swatch click, the rest via `#accent-custom`), run 14.2, then (edit `A` per accent):

```js
() => {
  const A = '#ffff00', T = window.__jv.theme;
  T.accent.set(A);
  const cs = getComputedStyle(document.documentElement);
  const hexNow = cs.getPropertyValue('--accent').trim().toLowerCase();
  const fg = cs.getPropertyValue('--accent-fg').trim().toLowerCase();
  let stored; try { stored = localStorage.getItem('jv-accent'); } catch (e) { stored = null; }
  const pass = hexNow === A && T.accent.get().toLowerCase() === A && /^(#000000|#000|#fff|#ffffff|rgb\(0, 0, 0\)|rgb\(255, 255, 255\))$/.test(fg);
  return { pass, details: { hexNow, fg, stored } };
}
```

Expected: `--accent-fg` is black for yellow, white for navy. Primary button, active tab, focus ring, timeline marker and histogram bars change colour immediately (compare screenshots before/after, no reload). Yellow on light: accent-coloured text stays readable (darker `--accent-text`). Reload: accent and theme persist. Click `#accent-reset`: accent returns to `#2e7d4f`. Timeline and histogram repaint on theme or accent change (no stale colours from the previous theme).

### 14.4 file:// and storage failure

Open the file URL: toggle theme and change accent. Both work for the session, no console errors (storage may throw), `list_network_requests` shows no `/api/` and no fetch. Repeat with site data blocked.

### 14.5 Screenshots (save under `research/screenshots/viewer-redesign/`, do not commit)

| File | Viewport | Tab | State |
|---|---|---|---|
| dark-1920x1080-replay.png | 1920x1080 | Replay | dark, default accent |
| dark-1920x1080-dashboard.png | 1920x1080 | Dashboard | dark, default accent |
| light-1920x1080-replay.png | 1920x1080 | Replay | light, default accent |
| light-1920x1080-dashboard.png | 1920x1080 | Dashboard | light, default accent |
| dark-1280x720-replay-accent-box.png | 1280x720 | Replay | dark, `#accent-box` open |
| light-1280x720-replay-accent-box.png | 1280x720 | Replay | light, `#accent-box` open |
| dark-1280x720-replay-purple.png | 1280x720 | Replay | dark, accent `#7b3fa0` |
| dark-1280x720-dashboard-purple.png | 1280x720 | Dashboard | dark, accent `#7b3fa0` |
| light-1280x720-replay-yellow.png | 1280x720 | Replay | light, accent `#ffff00` (contrast stress) |
| light-1280x720-dashboard-yellow.png | 1280x720 | Dashboard | light, accent `#ffff00` |
| dark-1280x720-replay-navy.png | 1280x720 | Replay | dark, accent `#000080` (contrast stress) |
| dark-800x600-dashboard.png | 800x600 | Dashboard | dark |
| light-800x600-replay-log.png | 800x600 | Replay | light, log-box open |
| light-1280x720-tooltip-theme-toggle.png | 1280x720 | Replay | light, `#theme-toggle` tooltip via focus |
| dark-1280x720-tooltip-theme-toggle.png | 1280x720 | Replay | dark, `#theme-toggle` tooltip via hover |

Also repeat the section 12 subset (1920x1080 replay and dashboard, 1280x720 knobs, 800x600 dashboard) in light with the default accent.

### 14.6 Review gate additions (extends section 13)

Give the fresh reviewer the dark, light and non-default accent screenshots from 14.5 in addition to section 12. Extra rubric: both themes equally polished, text and chart contrast in each theme, accent applied consistently (buttons, tabs, focus ring, chart marks), readability with yellow and navy accents, popover legibility. Same pass rule: any FAIL or score below 4 is fixed, re-shot in BOTH themes and re-reviewed by a NEW reviewer.

## 15. No clipping, select alignment, knob sliders (v2 addendum 2: user lost the bottom of the page)

Viewports (resize with `resize_page`, wait ~300ms): 1920x1080, 1366x768, 1280x720, 800x600, 1920x900, 1280x640. Run 15.1 for each viewport x tab {Replay, Dashboard} x state {closed, log-box, more-box} (served mode also knob-box, batch-box). Define the helper from section 0 first. 6 x 2 x 3 = 36 runs (60 served).

### 15.1 Nothing clipped (edit `TAB` and `STATE`)

```js
() => {
  const TAB = 'r', STATE = 'none'; // TAB 'r'|'d'; STATE 'none'|'log-box'|'more-box'|'knob-box'|'batch-box'
  const c = window.__chk; c.tab(TAB);
  if (STATE === 'none') c.closeAll(); else c.open(STATE);
  const W = innerWidth, H = innerHeight, T = 1, out = [];
  const view = document.getElementById(TAB === 'r' ? 'replay' : 'dashboard');
  const all = [...document.body.querySelectorAll('*')].filter(e => !e.closest('svg') && !e.closest('.sr') && c.vis(e) && !e.closest('[hidden],.hidden'));
  // 1. every visible element rect inside the viewport (hidden tooltips are skipped)
  for (const e of all) {
    const r = e.getBoundingClientRect();
    if (e.getAttribute('role') === 'tooltip' && getComputedStyle(e).opacity === '0') continue;
    if (r.left < -T || r.top < -T || r.right > W + T || r.bottom > H + T) out.push({ kind: 'outside-viewport', el: e.id || e.className || e.tagName, r: [r.left, r.top, r.right, r.bottom].map(Math.round) });
  }
  // 2. bottom-most interactive element and the last card fully visible
  const inter = all.filter(e => e.matches('button,summary,input,select,a,[tabindex]'));
  const lowest = inter.sort((a, b) => b.getBoundingClientRect().bottom - a.getBoundingClientRect().bottom)[0];
  const lr = lowest && lowest.getBoundingClientRect();
  if (!lr || lr.bottom > H + T) out.push({ kind: 'bottom-interactive-clipped', el: lowest && (lowest.id || lowest.textContent.trim().slice(0, 20)), bottom: lr && Math.round(lr.bottom) });
  const lastCard = [...view.querySelectorAll('.card')].filter(c.vis).sort((a, b) => b.getBoundingClientRect().bottom - a.getBoundingClientRect().bottom)[0];
  if (lastCard && lastCard.getBoundingClientRect().bottom > H + T) out.push({ kind: 'last-card-clipped', el: lastCard.id || lastCard.className });
  // 3. page itself must not scroll or exceed the viewport
  const d = document.documentElement;
  if (d.scrollHeight > H + T || d.scrollWidth > W + T || document.body.getBoundingClientRect().bottom > H + T) out.push({ kind: 'page-exceeds-viewport', sh: d.scrollHeight, H });
  // 4. no element hides overflowing content (overflow hidden/clip with scrollHeight > clientHeight), except ellipsis text
  for (const e of all) {
    const s = getComputedStyle(e);
    const hid = v => v === 'hidden' || v === 'clip';
    if ((hid(s.overflowY) && e.scrollHeight > e.clientHeight + T) || (hid(s.overflowX) && e.scrollWidth > e.clientWidth + T && s.textOverflow !== 'ellipsis'))
      out.push({ kind: 'hidden-overflow-clips', el: e.id || e.className || e.tagName, sh: e.scrollHeight, ch: e.clientHeight, sw: e.scrollWidth, cw: e.clientWidth });
  }
  return { pass: out.length === 0, details: { vp: W + 'x' + H, TAB, STATE, out: out.slice(0, 12) } };
}
```

Expected: `pass: true` for all runs. The layout reallocates height (the timeline/hist area shrinks) instead of pushing the drawer buttons below the fold. Screenshot 1280x640 with each drawer open and confirm the drawer panel is fully visible (not cut by the top or bottom edge).

### 15.2 Select alignment and one consistent style (each tab; 1280x720 and 800x600)

```js
() => {
  const sels = [...document.querySelectorAll('select')];
  const rows = sels.map(s => { const cs = getComputedStyle(s); return { id: s.id, cls: s.className, ta: cs.textAlign, tal: cs.textAlignLast, pr: cs.paddingRight, pl: cs.paddingLeft, h: cs.height, bg: cs.backgroundImage !== 'none' || !!s.parentElement.querySelector('use[href="#i-chevron-down"]') }; });
  const sp = document.getElementById('speed'), r = sp.getBoundingClientRect(), cs = getComputedStyle(sp);
  const m = document.createElement('span');
  m.style.cssText = 'position:absolute;visibility:hidden;white-space:nowrap;font:' + cs.font; m.textContent = sp.selectedOptions[0].text; document.body.appendChild(m);
  const tw = m.getBoundingClientRect().width; m.remove();
  const same = k => new Set(rows.map(x => x[k])).size === 1;
  const pass = rows.length >= 1 && rows.every(x => /\bselect\b/.test(x.cls) && (x.ta === 'right' || x.ta === 'end') && (x.tal === 'right' || x.tal === 'end') && x.bg) && same('pr') && same('pl') && same('h') && parseFloat(rows[0].pr) >= 20 && tw < r.width;
  return { pass, details: { rows, textW: Math.round(tw) } };
}
```

Expected: `pass: true`. Visually (crop of the Speed select, both themes): the selected value is flush right next to the chevron, with the chevron at the same inset as every other select. Open the select: options are readable in both themes.

### 15.3 Knobs are sliders in a non-scrolling grid (served mode, each viewport incl. 1280x640 and 800x600)

```js
() => {
  const c = window.__chk; c.open('knob-box');
  const K = document.getElementById('knobs'), cs = getComputedStyle(K), kb = document.getElementById('knob-box');
  const knobs = [...K.querySelectorAll('.knob')];
  const rows = knobs.map(k => { const r = k.querySelector('input[type=range]'), n = k.querySelector('input[type=number]'); return { id: r && r.id, ok: !!r && !!n && n.id === r.id + '-num' && r.min === n.min && r.max === n.max && r.step === n.step && r.value === n.value && !!k.querySelector('.info') }; });
  const cols = cs.gridTemplateColumns.split(' ').length;
  const pr = kb.querySelector('.pop-body').getBoundingClientRect();
  let p = K; const scrollers = [];
  while (p && p !== document.body) { const s = getComputedStyle(p); if (/auto|scroll/.test(s.overflowY + s.overflowX) && (p.scrollHeight > p.clientHeight || p.scrollWidth > p.clientWidth)) scrollers.push(p.id || p.className); p = p.parentElement; }
  const inside = pr.left >= -1 && pr.top >= -1 && pr.right <= innerWidth + 1 && pr.bottom <= innerHeight + 1;
  const k0 = knobs[0].querySelector('input[type=range]'), n0 = knobs[0].querySelector('input[type=number]');
  k0.value = k0.max; k0.dispatchEvent(new Event('input', { bubbles: true }));
  const s1 = n0.value === k0.value;
  n0.value = String(+k0.max + 1000); n0.dispatchEvent(new Event('input', { bubbles: true })); n0.dispatchEvent(new Event('change', { bubbles: true }));
  const clamped = +n0.value === +k0.max && +k0.value === +k0.max;
  document.getElementById('knobs-reset').click();
  const reset = knobs.every(k => k.querySelector('input[type=range]').value === k.querySelector('input[type=number]').value);
  const pass = knobs.length === 13 && rows.every(x => x.ok) && /grid/.test(cs.display) && cols >= 2 && !/auto|scroll/.test(cs.overflowY + cs.overflowX) && scrollers.length === 0 && inside && s1 && clamped && reset;
  return { pass, details: { knobs: knobs.length, cols, overflow: cs.overflowY, scrollers, inside, pop: [pr.left, pr.top, pr.right, pr.bottom].map(Math.round), bad: rows.filter(x => !x.ok).map(x => x.id), s1, clamped, reset } };
}
```

Expected: `pass: true` at every viewport: 13 knobs, 2+ columns (more at wide viewports), no scrollbar anywhere in the panel, panel fully inside the viewport. Screenshots (do not commit): `research/screenshots/viewer-redesign/knobs-sliders-1920x1080.png`, `knobs-sliders-1280x640.png`, `knobs-sliders-800x600.png`; confirm slider thumbs use the accent, the number input shows the live value while dragging, and the info tooltip still opens beside each knob.

## 16. Space use, event log modal, seed field (v2 addendum 3)

Browser checks (real layout and focus; node stubs cannot check these). Served mode, run loaded, Replay tab, each viewport 1920x1080, 1366x768, 1280x720, 1280x640, 800x600, both themes. Node-checkable parts: `node tests/viewer_polish_check.mjs`.

### 16.1 Hero details in the top card; no dead gaps; one-row bottom bar
