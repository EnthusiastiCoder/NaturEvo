// Crisp canvases (plans/2026-10-20_viewer-redesign-v2.md, "Addendum 4"). Static markup/CSS checks plus
// behaviour via the stub-DOM harness (stubs: devicePixelRatio, clientWidth/clientHeight, setTransform spy,
// matchMedia resolution listeners, ResizeObserver). Derived from requirements, not from current code.
import { html, load, makeRun, tick, runner } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_canvas_check');
const css = ((html.match(/<style[^>]*>([\s\S]*?)<\/style>/) || [, ''])[1]).replace(/\/\*[\s\S]*?\*\//g, '');
const markup = html.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<style[\s\S]*?<\/style>/g, '');
