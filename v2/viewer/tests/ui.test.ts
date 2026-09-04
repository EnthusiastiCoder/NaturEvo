// Static checks on v2/viewer/index.html and src/style.css (no DOM dependency; only pixi/vite/vitest allowed).
// Assumed: src/theme.ts exports DEFAULT_ACCENT, nextTheme(t), applyTheme(root,t), setAccent(root,hex),
// src/layout.ts exports fitStage(winW, winH, stageW, stageH) -> {w,h,scale}.
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { DEFAULT_ACCENT, nextTheme, applyTheme, setAccent } from '../src/theme';
import { fitStage } from '../src/layout';

const root = resolve(__dirname, '..');
const html = () => readFileSync(resolve(root, 'index.html'), 'utf8');
const css = () => readFileSync(resolve(root, 'src/style.css'), 'utf8');
