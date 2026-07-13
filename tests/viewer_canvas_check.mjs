// Crisp canvases (plans/2026-10-20_viewer-redesign-v2.md, "Addendum 4"). Static markup/CSS checks plus
// behaviour via the stub-DOM harness (stubs: devicePixelRatio, clientWidth/clientHeight, setTransform spy,
// matchMedia resolution listeners, ResizeObserver). Derived from requirements, not from current code.
import { html, load, makeRun, tick, runner } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_canvas_check');
const css = ((html.match(/<style[^>]*>([\s\S]*?)<\/style>/) || [, ''])[1]).replace(/\/\*[\s\S]*?\*\//g, '');
const markup = html.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<style[\s\S]*?<\/style>/g, '');

// ---------- static: no hard-coded sizes ----------
{
  const tags = [...markup.matchAll(/<canvas\b[^>]*>/gi)].map(m => m[0]);
  check(tags.length > 0, 'page must contain at least one <canvas>');
  for (const t of tags) {
    check(!/\swidth\s*=/i.test(t) && !/\sheight\s*=/i.test(t), 'canvas must not have width/height attributes in markup: ' + t);
    check(!/style\s*=\s*["'][^"']*(width|height)\s*:\s*\d+(\.\d+)?px/i.test(t), 'canvas must not have inline fixed-px width/height: ' + t);
  }
  const ids = tags.map(t => (t.match(/\bid\s*=\s*["']([^"']+)["']/) || [])[1]).filter(Boolean);
  for (const m of css.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
    const sels = m[1].split(',').map(s => s.trim());
    if (!sels.some(s => /(^|[\s>+~])canvas\b/i.test(s) || ids.some(i => s.endsWith('#' + i)))) continue;
    for (const d of m[2].split(';')) {
      const mm = d.match(/^\s*(width|height)\s*:\s*([^;]+)$/i);
      if (mm) check(!/^\s*\d+(\.\d+)?px\s*$/i.test(mm[2]), `CSS rule "${m[1].trim()}" must not set fixed-px ${mm[1]}: ${mm[2].trim()}`);
    }
  }
}
