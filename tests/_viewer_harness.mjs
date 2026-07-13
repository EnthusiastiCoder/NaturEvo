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
