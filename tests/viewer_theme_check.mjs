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
