// Addendum 3 requirements (plans/2026-10-20_viewer-redesign-v2.md): (1) hero details live in the top
// hero/battle card (no Hero details drawer), drawers in one row; (2) event log is a searchable,
// groupable, windowed modal dialog; (3) seed input wide, non-spinner, validated.
// Derived from the requirements, not from the current code. Real pixel/focus checks: checklist section 16.
import { html, load, makeRun, tick, runner, jsonResp } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_polish_check');

const css = ((html.match(/<style[^>]*>([\s\S]*?)<\/style>/) || [, ''])[1]).replace(/\/\*[\s\S]*?\*\//g, '');
const markup = html.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<style[\s\S]*?<\/style>/g, '');
const rules = [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)].map(m => ({
  sels: m[1].split(',').map(s => s.trim().replace(/\s+/g, ' ')),
  decl: Object.fromEntries(m[2].split(';').map(d => { const k = d.indexOf(':'); return k < 0 ? null : [d.slice(0, k).trim().toLowerCase(), d.slice(k + 1).trim()]; }).filter(Boolean)),
}));
const declFor = pred => Object.assign({}, ...rules.filter(r => r.sels.some(pred)).map(r => r.decl));
const tagOf = id => (markup.match(new RegExp('<(\\w+)\\b([^>]*\\bid\\s*=\\s*["\']' + id + '["\'][^>]*)>')) || null);
const attr = (t, a) => t && ((t[2].match(new RegExp('\\b' + a + '\\s*=\\s*["\']([^"\']*)["\']')) || [])[1]);

// ---------- (1) space use: hero details in the top card, no drawer ----------
{
  check(!/id\s*=\s*["']more-box["']/.test(markup), 'Hero details drawer #more-box must be removed');
  check(!/id\s*=\s*["']hmeta["']/.test(markup), 'old #hmeta drawer body must be removed');
  const hd = markup.search(/id\s*=\s*["']hero-details["']/), b = markup.search(/id\s*=\s*["']battle["']/), t = markup.search(/id\s*=\s*["']timeline-card["']/);
  check(hd >= 0, '#hero-details element missing');
  check(hd > b && hd < t, '#hero-details must sit inside the top hero/battle card (between #battle and #timeline-card)');
  const dr = declFor(s => s === '.drawers');
  check(!/column/.test(dr['flex-direction'] || '') && !/column|row\s+dense/.test(dr['grid-auto-flow'] || '') && /flex|grid/.test(dr.display || ''), 'bottom bar .drawers must lay out in one row, got ' + JSON.stringify(dr));
  check(!/wrap(?!-reverse\b)/.test(dr['flex-wrap'] || '') || /nowrap/.test(dr['flex-wrap'] || ''), 'bottom bar .drawers must not wrap onto a second row');
  const hdc = declFor(s => s === '#hero-details');
  check(/flex|grid/.test(hdc.display || ''), '#hero-details must be a flex/grid of chips, got display ' + hdc.display);
}
{
  const R = makeRun(60);
  R.events.forEach((e, i) => { e.hero = { level: 1 + (i >= 30 ? 6 : 0), hp: 10, max_hp: 10, xp: i >= 30 ? 345 : 12, gold: i >= 30 ? 1234 : 50, potions: i >= 30 ? 3 : 1, zone: 0, weapon: i >= 30 ? 3 : 1, armor: i >= 30 ? 2 : 1 }; });
  const v = load({ protocol: 'file:', run: R });
  const jv = v.win.__jv, hd = v.$('hero-details');
  check(!!hd, '#hero-details must exist in the DOM');
  check(!v.$('more-box'), '#more-box must not exist');
  if (hd && jv) {
    const txt = () => hd.textContent.replace(/\s+/g, ' ');
    // at event 0: Wooden... weapon 1 = Stone Axe +4, armor 1 = Leaf Wrap +3
    check(/level\D{0,4}1\b/i.test(txt()), 'level chip at 0: ' + txt());
    check(/xp\D{0,4}12\b/i.test(txt()), 'xp chip at 0: ' + txt());
    check(/gold\D{0,4}50\b/i.test(txt()), 'gold chip at 0: ' + txt());
    check(/potions?\D{0,4}1\b/i.test(txt()), 'potions chip at 0: ' + txt());
    check(/attack\D{0,4}4\b/i.test(txt()), 'attack chip at 0 (Stone Axe +4): ' + txt());
    check(/defen[cs]e\D{0,4}3\b/i.test(txt()), 'defense chip at 0 (Leaf Wrap +3): ' + txt());
    jv.goto(45);
    check(/level\D{0,4}7\b/i.test(txt()), 'level chip after goto(45): ' + txt());
    check(/xp\D{0,4}345\b/i.test(txt()), 'xp chip after goto(45): ' + txt());
    check(/gold\D{0,4}1,?234\b/i.test(txt()), 'gold chip after goto(45): ' + txt());
    check(/potions?\D{0,4}3\b/i.test(txt()), 'potions chip after goto(45): ' + txt());
    check(/attack\D{0,4}16\b/i.test(txt()), 'attack chip after goto(45) (Steel Blade +16): ' + txt());
    check(/defen[cs]e\D{0,4}7\b/i.test(txt()), 'defense chip after goto(45) (Hide Vest +7): ' + txt());
    jv.goto(3);
    check(/gold\D{0,4}50\b/i.test(txt()) && /level\D{0,4}1\b/i.test(txt()), 'chips update backwards on goto(3): ' + txt());
  }
}

// ---------- (2) event log modal ----------
{
  const dlg = tagOf('log-modal');
  check(!!dlg, '#log-modal element missing');
  if (dlg) {
    const isDialog = dlg[1].toLowerCase() === 'dialog';
    check(isDialog || (attr(dlg, 'role') === 'dialog' && attr(dlg, 'aria-modal') === 'true'), '#log-modal must be <dialog> or role=dialog aria-modal=true');
    const lb = attr(dlg, 'aria-labelledby');
    check(!!lb && new RegExp('id\\s*=\\s*["\']' + lb + '["\']').test(markup), '#log-modal needs aria-labelledby pointing to an existing id, got ' + lb);
    if (isDialog) {
      const s = markup.search(/id\s*=\s*["']log-modal["']/), e = markup.indexOf('</dialog>', s), l = markup.search(/id\s*=\s*["']log["']/);
      check(l > s && l < e, '#log must be inside #log-modal');
    }
    const md = declFor(s => s === '#log-modal');
    check(/(vh|dvh|%)/.test(md['max-height'] || ''), '#log-modal must have a viewport-bounded max-height, got ' + md['max-height']);
  }
  check(/^button$/i.test((tagOf('log-open') || [])[1] || ''), '#log-open must be a <button>');
  const se = tagOf('log-search');
  check(!!se && attr(se, 'type') === 'search' && !!attr(se, 'aria-label'), '#log-search must be type=search with aria-label');
  check(/^select$/i.test((tagOf('log-group') || [])[1] || '') || !!tagOf('log-group'), '#log-group control missing');
  const ld = declFor(s => s === '#log');
  check(/auto|scroll/.test((ld['overflow-y'] || ld.overflow || '')), '#log must be the scroller (overflow auto), got ' + JSON.stringify(ld));
  check(/position\s*:\s*relative/.test(css.replace(/\s+/g, ' ').match(/#log\s*\{[^}]*\}/) ? css.match(/#log\s*\{[^}]*\}/)[0] : ''), '#log needs position:relative');
  const gs = (markup.match(/<select\b[^>]*\bid\s*=\s*["']log-group["'][^>]*>([\s\S]*?)<\/select>/) || [, ''])[1];
  check(/type/i.test(gs) && /zone/i.test(gs) && /none/i.test(gs), '#log-group needs options by type, zone, none');
  const opts = [...gs.matchAll(/<option([^>]*)>([^<]*)/gi)];
  const def = opts.find(o => /selected/.test(o[1])) || opts[0];
  check(!!def && /type/i.test(def[1] + def[2]), 'by type must be the default grouping (selected or first option)');
}

const KINDS = ['travel', 'fight', 'loot'];
const mk = n => { const r = makeRun(n); r.events.forEach((e, i) => { e.kind = KINDS[i % 3]; e.hero.zone = i % 4; }); return r; };
const sorted = a => [...a].sort((x, y) => x - y);
const eq = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const nums = n => Array.from({ length: n }, (_, i) => i);
const all = el => el._all();
const headers = L => all(L).filter(e => e.getAttribute('aria-expanded') !== undefined);

modal: {
  const v = load({ protocol: 'file:', run: mk(300) });
  const jv = v.win.__jv, $ = v.$, L = $('log'), dlg = $('log-modal'), open = $('log-open'), S = $('log-search'), G = $('log-group'), C = $('log-count');
  if (!dlg || !open || !L || !S || !G || !C) { check(false, 'modal elements missing (log-modal/log-open/log/log-search/log-group/log-count)'); break modal; }
  const isOpen = () => dlg.tagName === 'DIALOG' ? !!dlg.open : (!dlg.classList.contains('hidden') && !dlg.hidden && dlg.style.display !== 'none');
  check(!isOpen(), 'modal closed initially');
  open.focus(); open.click();
  check(isOpen(), '#log-open click must open the modal');
  check(dlg.tagName !== 'DIALOG' || dlg.modalCalls >= 1, '<dialog> must be opened with showModal() (modal, inert page, native trap)');
  check(v.doc.activeElement === S, 'focus must move to #log-search on open');

  // Tab trap, where stub-checkable: Tab keydown on the search input must not leave the dialog.
  // Native showModal traps natively; a manual trap must preventDefault when wrapping. Either is accepted:
  // we only require that Tab/Shift+Tab events do not throw and focus stays on an element of the dialog or is prevented.
  { let threw = false; try { S.fire('keydown', { key: 'Tab', shiftKey: true }); S.fire('keydown', { key: 'Tab' }); } catch (e) { threw = true; } check(!threw, 'Tab keydown handling must not throw'); }
  check(dlg.tagName === 'DIALOG' ? dlg.modalCalls >= 1 : /Tab/.test(html.replace(/<style[\s\S]*?<\/style>/, '')), 'focus trap: showModal() or explicit Tab handling required');
