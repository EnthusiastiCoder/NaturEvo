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
