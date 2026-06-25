# Viewer style system

One file, `viewer/index.html`, no build, no CDN. Look: shadcn/ui-style analytics dashboard (zinc neutrals,
1px borders, muted foreground, 8px radius, 32px controls). Everything below lives in the single `<style>` block.

## Tokens

All values come from `:root` custom properties. Outside the token blocks use `var(--x)` only: no hex/rgb
literals, no `px` font sizes, no inline `style=` in markup (JS may set dynamic widths/heights/positions only).

- Colours (per theme): `--bg --card --card-2 --border --fg --muted --good --bad --warn --ring`
- Accent (user-settable): `--accent`, `--accent-fg` (text on accent), `--accent-text` (accent used as text/border)
- Spacing: `--s1..--s6` = 4, 8, 12, 16, 24, 32 px
- Radii: `--r-sm`, `--r-md` (8px)
- Font sizes: `--fs-xs` 12, `--fs-sm` 13, `--fs-md` 14, `--fs-lg` 18, `--fs-kpi` 28
- Sizes: `--icon` 16px, `--control-h` 32px; elevation `--shadow-pop`

## Components

| Class | Use |
|---|---|
| `.app` | page shell grid (toolbar row + content row), never scrolls |
| `.toolbar` | top bar, one row, wraps on narrow screens |
| `.brand` | logo mark + title |
| `.tool-group` | inline cluster of toolbar controls |
| `.spacer` | flexible gap that pushes following items right |
| `.main` | content area under the toolbar, `min-height:0` |
| `.view` | one tab body (`#replay`, `#dashboard`), fills `.main` |
| `.btn` | button, height `var(--control-h)`; variants `.btn-primary` (accent fill), `.btn-outline`, `.btn-ghost` |
| `.btn-icon` | square icon-only button, needs `aria-label` plus a tooltip |
| `.icon` | Lucide `<svg>` sized `var(--icon)`, uses `<use href="#i-name"/>` |
| `.input` | text/number/select control, height `var(--control-h)` |
| `.label` | form label, small muted text |
| `.field` | label + control row |
| `.range` | styled range input (scrub) |
| `.card` | bordered surface; `.card-title` is its heading row |
| `.kpi` | KPI card; `.kpi-label`, `.kpi-value` inside |
| `.tabs` | segmented tab group; `.tab` is a tab button, `.on` marks the selected one |
| `.badge` | small status pill; `.badge-good`, `.badge-bad` |
| `.tipwrap` | wrapper of a trigger button and its `.tip` |
| `.tip` | tooltip (`role="tooltip"`), shown on hover and focus of the trigger, Escape hides |
| `.info` | info button, `class="btn-icon info"`, `aria-label="About X"`, `aria-describedby="tip-x"` |
| `.expander` | `<details>` wrapper; `.popover` opens a floating `.pop-body` card under a toolbar button; `.pop-head` is its title row |
| `.drawers` | bottom bar of log/more drawers; `.drawer` is one `<details>`, `.drawer-body` its panel (opens upward, bounded) |
| `.table` | compact data table |
| `.scroll` | bounded inner scroll area (`overflow:auto`, `min-height:0`) |
| `.swatches`, `.swatch` | accent colour presets (`data-accent` carries the colour) |
| `.knob` | one tuning knob row (label, info, input) |
| `.fighter`, `.emo`, `.bar` | battle panel parts: sprite box, emoji sprite, HP bar |
| `.playback` | playback card layout |
| `.sr` | visually hidden text for screen readers |
| `.hidden` | `display:none` helper (also a legacy-id helper for `#run-controls`, `#replay`, `#dashboard`) |
| `.hrow`, `.hfill` | simple histogram row and bar (dashboard) |

Legacy ids (`#seed`, `#log`, `#hist`, ...) stay; they are hooks for tests and scripts, not styling vocabulary.

## Icons

Lucide (ISC licence, notice kept in the sprite comment). One inline sprite `<svg class="hidden" aria-hidden="true">` at the top
of `<body>` holds `<symbol id="i-NAME" viewBox="0 0 24 24">` (stroke `currentColor`, width 2, round caps/joins, no fill).
Use `<svg class="icon" aria-hidden="true"><use href="#i-NAME"/></svg>`. No emoji or text glyphs as UI chrome; the only
emoji are the sprite arrays `SP` and `ZN` in the script.
