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
