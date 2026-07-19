// Style-system / icon / tooltip / dashboard requirements for the viewer redesign v2
// (plans/2026-10-20_viewer-redesign-v2.md, "Test cases"). Derived from requirements, not
// from the current layout. Static checks parse viewer/index.html; behaviour checks use the
// shared stub-DOM harness. No dependencies.
import { readFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { html, load, makeRun, tick, runner, jsonResp } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_style_check');
const root = join(dirname(fileURLToPath(import.meta.url)), '..');

// ---------- static helpers ----------
const scripts = [...html.matchAll(/<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/g)].map(m => m[1]);
const styleRaw = (html.match(/<style[^>]*>([\s\S]*?)<\/style>/) || [, ''])[1];
const css = styleRaw.replace(/\/\*[\s\S]*?\*\//g, '');
const markup = html.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<style[\s\S]*?<\/style>/g, '');
const attr = (tag, name) => { const m = tag.match(new RegExp('\\b' + name + '\\s*=\\s*(?:"([^"]*)"|\'([^\']*)\')', 'i')); return m ? (m[1] ?? m[2]) : null; };
const stripTags = s => s.replace(/<[^>]*>/g, '').replace(/&nbsp;/g, ' ').trim();

// Flat list of CSS rules (also those nested in @media) -> { sels, body, root }.
const rules = [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)].map(m => {
  const sels = m[1].split(',').map(s => s.trim().replace(/\s+/g, ' '));
  return { sels, body: m[2], root: sels.length >= 1 && sels.every(s => /^:root(\[data-theme=["']?(light|dark)["']?\])?$/.test(s.trim())) };
});
const decls = body => body.split(';').map(d => { const k = d.indexOf(':'); return k < 0 ? null : [d.slice(0, k).trim().toLowerCase(), d.slice(k + 1).trim()]; }).filter(Boolean);
const rootCss = rules.filter(r => r.root).map(r => r.body).join(';');
const nonRoot = rules.filter(r => !r.root);

// Tooltip elements present in static markup: id -> text.
const tipsStatic = new Map();
for (const m of markup.matchAll(/<(\w+)\b([^>]*\brole\s*=\s*["']tooltip["'][^>]*)>([\s\S]*?)<\/\1>/g)) {
  const id = attr(m[2], 'id'); if (id) tipsStatic.set(id, stripTags(m[3]));
}

// ---------- (1) no emoji / symbol glyphs outside SP and ZN ----------
{
  const stripSprites = s => s.replace(/\b(SP|ZN)\s*=\s*\[\[[\s\S]*?\]\]\s*;/g, '');
  const text = markup + '\n' + scripts.map(stripSprites).join('\n');
  const glyph = /[\p{Extended_Pictographic}\u2190-\u21FF\u25A0-\u25FF\u2600-\u27BF\u2B00-\u2BFF\u2022\u00D7\u2026\uFE0F]/gu;
  const found = [...new Set(text.match(glyph) || [])].map(c => 'U+' + c.codePointAt(0).toString(16).toUpperCase());
  check(found.length === 0, 'emoji/symbol glyph chars outside SP/ZN sprite arrays: ' + found.join(' '));
  const ents = [...new Set([...text.matchAll(/&(?:#(\d+)|#x([0-9a-f]+)|(rarr|larr|uarr|darr|harr|bull|times|hellip|check|cross|starf|star|phone));/gi)]
    .filter(m => m[3] || ((c => (c >= 0x2190 && c <= 0x21ff) || (c >= 0x25a0 && c <= 0x25ff) || (c >= 0x2600 && c <= 0x27bf) || (c >= 0x2b00 && c <= 0x2bff) || c === 0x2022 || c >= 0x1f000)(m[1] ? +m[1] : parseInt(m[2], 16))))
    .map(m => m[0]))];
  check(ents.length === 0, 'symbol HTML entities used as UI chrome: ' + ents.join(' '));
  const esc = [...new Set([...text.matchAll(/\\u(2[1-9a-b][0-9a-f]{2}|2022|d83[cd])/gi)].map(m => m[0]))];
  check(esc.length === 0, 'symbol \\u escapes used as UI chrome: ' + esc.join(' '));
}

// ---------- (2) one inline Lucide sprite, no external assets ----------
const symbols = new Set([...html.matchAll(/<symbol\b[^>]*\bid\s*=\s*["'](i-[\w-]+)["']/g)].map(m => m[1]));
{
  const svgs = [...markup.matchAll(/<svg\b[\s\S]*?<\/svg>/g)].map(m => m[0]).filter(s => /<symbol\b/.test(s));
  check(svgs.length === 1, 'exactly one inline sprite <svg> containing <symbol> required, found ' + svgs.length);
  check(symbols.size >= 10, 'sprite must define >= 10 <symbol id="i-..."> icons, found ' + symbols.size);
  check(/ISC/.test(html) && /lucide/i.test(html), 'Lucide ISC licence notice (mentions "ISC" and "Lucide") must be present in the html');
  const uses = [...html.matchAll(/<use\b[^>]*\bhref\s*=\s*["']#(i-[\w-]+)["']/g)].map(m => m[1]);
  check(uses.length >= 5, 'icons must be used via <use href="#i-name">, found ' + uses.length + ' uses');
  const missing = [...new Set(uses.filter(u => !symbols.has(u)))];
  check(missing.length === 0, '<use> references missing symbols: ' + missing.join(', '));
  check(!/<img\b/i.test(html), 'no <img> allowed');
  check(!/\b(src|href)\s*=\s*["']\s*(https?:)?\/\//i.test(html), 'no external src/href (http(s):// or //) allowed');
  check(!/<link\b/i.test(html), 'no <link> allowed');
}
