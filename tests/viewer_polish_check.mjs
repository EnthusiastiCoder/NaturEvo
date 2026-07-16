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
