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
