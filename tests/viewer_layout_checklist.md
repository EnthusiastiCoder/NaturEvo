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
