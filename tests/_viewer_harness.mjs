// Shared helper for viewer node tests: loads the inline script of viewer/index.html
// in node:vm with a small stub DOM, stub fetch and stub timers. No dependencies.
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import vm from 'node:vm';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
export const htmlPath = join(root, 'viewer', 'index.html');
export const html = readFileSync(htmlPath, 'utf8');

export function makeRun(n, seed = 1) {
  const events = [];
  for (let i = 0; i < n; i++) {
    events.push({
      idx: i, kind: 'travel', text: 'event ' + i,
      hero: { level: 1 + Math.floor(i / 20), hp: 10, max_hp: 10, xp: 0, gold: 0, potions: 0, zone: 0, weapon: 0, armor: 0 },
      mon: null,
    });
  }
  return { seed, events, won: false, deaths: 0, final_level: 1 };
}

export const tick = async () => { for (let i = 0; i < 20; i++) await new Promise(r => setImmediate(r)); };

// Additive options (theme tests): storage = object (initial localStorage contents) | 'throw-access'
// (reading window.localStorage throws, like file:// SecurityError) | 'throw-ops' (getItem/setItem throw);
// matchMedia = 'dark' | 'light' | 'absent' | 'throws' (prefers-color-scheme answer).
// Returns extra: paints (canvas fillStyle/strokeStyle assignments), events (dispatched event types),
// store (backing Map), root (documentElement stub with style.setProperty / getPropertyValue).
export function load({ protocol = 'file:', run = makeRun(300), fetchImpl = null, storage = {}, matchMedia = 'dark', pre = null } = {}) {
  const paints = [], events = [], store = new Map(Object.entries(typeof storage === 'object' ? storage : {}));
  const listeners = {};
  const addL = (t, f) => { (listeners[t] = listeners[t] || []).push(f); };
  const dispatch = ev => { events.push(ev.type); (listeners[ev.type] || []).forEach(f => f.call(null, ev)); return true; };
  const byId = new Map();
  const fetchCalls = [];
  const intervals = [];

  const docRef = { activeElement: null };
  class El {
    constructor(tag) {
      this.tagName = (tag || 'div').toUpperCase();
      this.children = []; this.parent = null;
      this.dataset = {}; this.style = { cssText: '' };
      this._cls = ''; this._text = ''; this._html = '';
      this.value = ''; this.disabled = false; this.max = '0'; this.min = '0'; this.type = '';
      this.clientHeight = 260; this.scrollTop = 0; this.handlers = {};
      this.onclick = null; this.oninput = null; this.onchange = null;
      this._id = ''; this._doc = docRef;
      this.classList = {
        add: (...c) => { const s = new Set(this._cls.split(/\s+/).filter(Boolean)); c.forEach(x => s.add(x)); this._cls = [...s].join(' '); },
        remove: (...c) => { this._cls = this._cls.split(/\s+/).filter(x => x && !c.includes(x)).join(' '); },
        contains: c => this._cls.split(/\s+/).includes(c),
        toggle: (c, f) => { const has = this.classList.contains(c); const want = f === undefined ? !has : f; want ? this.classList.add(c) : this.classList.remove(c); return want; },
      };
    }
    get id() { return this._id; }
    set id(v) { this._id = v; byId.set(v, this); }
    get className() { return this._cls; }
    set className(v) { this._cls = String(v); }
    get textContent() { return this._text + this.children.map(c => c.textContent).join(''); }
    set textContent(v) { this._text = String(v); this.children = []; }
    get innerHTML() { return this._html; }
    set innerHTML(v) {
      this._html = String(v); this._text = ''; this.children = [];
      for (const m of this._html.matchAll(/<(\w+)([^>]*\bid\s*=\s*["']([^"']+)["'][^>]*)>/g)) {
        const e = new El(m[1]); e.id = m[3];
        // additive: copy simple static attributes so tests can check type/min/max/step/class
        for (const a of ['type', 'min', 'max', 'step', 'value']) { const am = m[2].match(new RegExp('\\b' + a + '\\s*=\\s*["\']([^"\']*)["\']')); if (am) e[a] = am[1]; }
        const cm = m[2].match(/\bclass\s*=\s*["']([^"']*)["']/); if (cm) e.className = cm[1];
        this.appendChild(e);
      }
    }
    get offsetTop() { return this.parent ? this.parent.children.indexOf(this) * 20 : 0; }
    appendChild(c) { c.parent = this; this.children.push(c); return c; }
    removeChild(c) { this.children = this.children.filter(x => x !== c); return c; }
    setAttribute(k, v) { if (/^data-/.test(k)) this.dataset[k.slice(5).replace(/-(\w)/g, (_, c) => c.toUpperCase())] = String(v); if (k === 'id') this.id = v; else if (k === 'class') this.className = v; else this[k] = v; }
    getAttribute(k) { return k === 'id' ? this._id : k === 'class' ? this._cls : this[k]; }
    addEventListener(t, f) { (this.handlers[t] = this.handlers[t] || []).push(f); }
    removeEventListener() {}
    _all() { return this.children.flatMap(c => [c, ...c._all()]); }
    querySelector(sel) { return this.querySelectorAll(sel)[0] || null; }
    querySelectorAll(sel) {
      const m = sel.match(/^([.#]?)([\w-]+)$/);
      if (!m) return [];
      return this._all().filter(e => m[1] === '.' ? e.classList.contains(m[2]) : m[1] === '#' ? e._id === m[2] : e.tagName === m[2].toUpperCase());
    }
    getContext() {
      return new Proxy({}, { get: (t, k) => (k in t ? t[k] : () => {}), set: (t, k, v) => { if (k === 'fillStyle' || k === 'strokeStyle') paints.push(String(v)); t[k] = v; return true; } });
    }
    fire(type, extra) {
      const ev = Object.assign({ type, target: this, defaultPrevented: false, preventDefault() { this.defaultPrevented = true; }, stopPropagation() {} }, extra || {});
      const f = this['on' + type]; if (typeof f === 'function') f.call(this, ev);
      (this.handlers[type] || []).forEach(h => h.call(this, ev));
      // additive: native <dialog> emulation (Escape -> cancel -> close unless prevented)
      if (type === 'cancel' && this.tagName === 'DIALOG' && !ev.defaultPrevented) this.close();
      return ev;
    }
    // additive: focus + dialog stubs (polish tests)
    focus() { this._doc.activeElement = this; this.fire('focus'); }
    blur() { if (this._doc.activeElement === this) this._doc.activeElement = this._doc.body; }
    showModal() { this.open = true; this.modalCalls = (this.modalCalls || 0) + 1; }
    show() { this.open = true; }
    close() { if (!this.open) return; this.open = false; this.fire('close'); }
    click() { if (!this.disabled) this.fire('click'); }
  }

  // Create stub elements for every id found in the static HTML markup.
  const body = new El('body');
  for (const m of html.replace(/<script[\s\S]*?<\/script>/g, '').matchAll(/<(\w+)([^>]*)>/g)) {
    const idm = m[2].match(/\bid\s*=\s*["']([^"']+)["']/);
    if (!idm) continue;
    const e = new El(m[1]); e.id = idm[1];
    const cm = m[2].match(/\bclass\s*=\s*["']([^"']*)["']/); if (cm) e.className = cm[1];
    for (const a of ['type', 'min', 'max', 'step', 'pattern', 'inputmode']) { const am = m[2].match(new RegExp('\\b' + a + '\\s*=\\s*["\']([^"\']*)["\']')); if (am && !e[a]) e[a] = am[1]; }
    const vm_ = m[2].match(/\bvalue\s*=\s*["']([^"']*)["']/); if (vm_) e.value = vm_[1];
    body.appendChild(e);
  }
  if (byId.has('speed')) byId.get('speed').value = '1';

  const doc = {
    body, activeElement: body,
    getElementById: i => byId.get(i) || null,
    createElement: t => new El(t),
    querySelector: s => body.querySelector(s),
    querySelectorAll: s => body.querySelectorAll(s),
    addEventListener: addL,
    dispatchEvent: dispatch,
  };
  Object.defineProperty(docRef, 'activeElement', { get: () => doc.activeElement, set: v => { doc.activeElement = v; } });
  const props = new Map();
  const root = new El('html');
  root.style = { cssText: '', setProperty: (k, v) => props.set(k, String(v)), getPropertyValue: k => props.get(k) || '', removeProperty: k => props.delete(k) };
  root._props = props;
  doc.documentElement = root;
  const defaultFetch = async () => { throw new Error('fetch not stubbed'); };
  const sandbox = {
    document: doc,
    location: { protocol, href: protocol + '//x/', hostname: '127.0.0.1', origin: protocol + '//127.0.0.1' },
    fetch: (url, opts) => { fetchCalls.push(String(url)); return (fetchImpl || defaultFetch)(String(url), opts); },
    setInterval: (f, ms) => { intervals.push(f); return intervals.length; },
    clearInterval: () => {},
    setTimeout: (f, ms) => setTimeout(f, ms),
    clearTimeout,
    console,
    addEventListener: addL,
    dispatchEvent: dispatch,
    CustomEvent: class { constructor(type, init) { this.type = type; this.detail = init && init.detail; } },
    Event: class { constructor(type) { this.type = type; } },
    getComputedStyle: el => ({ getPropertyValue: n => resolveToken(el, n) }),
  };
  const cssAll = (html.match(/<style[^>]*>([\s\S]*?)<\/style>/) || [, ''])[1].replace(/\/\*[\s\S]*?\*\//g, '');
  const tokenOf = (sel, n) => {
    for (const m of cssAll.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
      if (m[1].split(',').map(x => x.trim()).includes(sel)) { const d = m[2].match(new RegExp('(?:^|[;\s])' + n + '\s*:\s*([^;]+)')); if (d) return d[1].trim(); }
    }
    return null;
  };
  const resolveToken = (el, n) => {
    if (props.has(n)) return props.get(n);
    const th = root.dataset.theme || root['data-theme'];
    return (th && tokenOf(':root[data-theme="' + th + '"]', n)) || (th && tokenOf(":root[data-theme='" + th + "']", n)) || tokenOf(':root', n) || '';
  };
  if (storage === 'throw-access') Object.defineProperty(sandbox, 'localStorage', { get() { throw new Error('SecurityError'); } });
  else sandbox.localStorage = {
    getItem: k => { if (storage === 'throw-ops') throw new Error('denied'); return store.has(k) ? store.get(k) : null; },
    setItem: (k, v) => { if (storage === 'throw-ops') throw new Error('denied'); store.set(k, String(v)); },
    removeItem: k => { if (storage === 'throw-ops') throw new Error('denied'); store.delete(k); },
  };
  if (matchMedia === 'throws') sandbox.matchMedia = () => { throw new Error('nope'); };
  else if (matchMedia !== 'absent') sandbox.matchMedia = q => ({ media: q, matches: /prefers-color-scheme:\s*(\w+)/.test(q) ? RegExp.$1 === matchMedia : false, addEventListener() {}, addListener() {} });
  sandbox.window = sandbox;
  sandbox.self = sandbox;
  if (run) sandbox.RUN = run;
  if (pre) pre(sandbox, { byId, El });
  vm.createContext(sandbox);
  const inline = [...html.matchAll(/<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/g)].map(m => m[1]);
  for (const code of inline) vm.runInContext(code, sandbox, { filename: 'index.html' });
  return { win: sandbox, doc, $: i => byId.get(i), fetchCalls, intervals, El, byId, paints, events, store, root };
}
