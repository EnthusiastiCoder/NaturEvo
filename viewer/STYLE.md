# Viewer style system

One file, `viewer/index.html`, no build, no CDN. Look: shadcn/ui-style analytics dashboard (zinc neutrals,
1px borders, muted foreground, 8px radius, 32px controls). Everything below lives in the single `<style>` block.

## Tokens

All values come from `:root` custom properties. Outside the token blocks use `var(--x)` only: no hex/rgb
literals, no `px` font sizes, no inline `style=` in markup (JS may set dynamic widths/heights/positions only).

- Colours (per theme): `--bg --card --card-2 --border --fg --muted --good --bad --warn --ring`
- Map colours (per theme, `#rrggbb` so contrast can be computed): `--tile-floor --tile-grass --tile-wall --tile-camp` and the zone tints `--biome-0..--biome-8`. The hero marker (`--accent`) keeps at least 3:1 contrast against floor and grass in both themes.
- Accent (user-settable): `--accent`, `--accent-fg` (text on accent), `--accent-text` (accent used as text/border)
- Spacing: `--s1..--s6` = 4, 8, 12, 16, 24, 32 px
- Radii: `--r-sm`, `--r-md` (8px)
- Font sizes: `--fs-2xs` 11 (chart axis text), `--fs-xs` 12, `--fs-sm` 13, `--fs-md` 14, `--fs-lg` 18, `--fs-kpi` 28
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
| `.anchor` | positioning wrapper for a toolbar button plus its popover |
| `.sr` | visually hidden text for screen readers |
| `.hidden` | `display:none` helper (also a legacy-id helper for `#run-controls`, `#replay`, `#dashboard`) |
| `.dash-grid` | dashboard two-column area (chart left, table right) |
| `.chart-card`, `.chart-head` | dashboard chart card and its title/toggle row |
| `.hsvg`, `.hbar`, `.hgrid`, `.htext` | histogram SVG, bars, gridlines, labels (token-coloured) |
| `.summary`, `.sitem`, `.sval` | replay summary strip, one item, its value |
| `.selectwrap` | wrapper placing the chevron icon over a native select |

Legacy ids (`#seed`, `#log`, `#hist`, ...) stay; they are hooks for tests and scripts, not styling vocabulary.

## Icons

Lucide (ISC licence, notice kept in the sprite comment). One inline sprite `<svg class="hidden" aria-hidden="true">` at the top
of `<body>` holds `<symbol id="i-NAME" viewBox="0 0 24 24">` (stroke `currentColor`, width 2, round caps/joins, no fill).
Use `<svg class="icon" aria-hidden="true"><use href="#i-NAME"/></svg>`. No emoji or text glyphs as UI chrome; the only
emoji are the sprite arrays `SP` and `ZN` in the script.

To add an icon: copy its SVG child elements from lucide.dev into a new `<symbol id="i-NAME" viewBox="0 0 24 24">` in the
sprite, then reference it with `<use>`.

## Tooltips and info buttons

Static: `<span class="tipwrap"><button type="button" class="btn-icon info" aria-label="About X" aria-describedby="tip-x"><svg class="icon" aria-hidden="true"><use href="#i-info"/></svg></button><span role="tooltip" id="tip-x" class="tip">Text</span></span>`.
Generated (knobs, KPIs): JS `tipHtml(id, name, text)` builds the same markup; texts live in the `INFO` map. Knob tooltips add
"Range min to max, default d" from `/api/config`. Every icon-only button needs `aria-label` and a tooltip.
The tooltip is `position:fixed`, placed by a tiny script so scroll containers never clip it; Escape hides it.

## Theming

Two token sets, `:root[data-theme="dark"]` and `:root[data-theme="light"]`, each define every colour token. Add a colour by
adding the token to BOTH blocks, then use `var(--name)`. Charts read tokens with `getComputedStyle` and redraw on the
`jv-themechange` event (fired on theme or accent change).

Initial theme: `localStorage jv-theme`, else `prefers-color-scheme`, else dark. The toolbar toggle `#theme-toggle` switches
and persists. Accent: `#accent-btn` opens `#accent-box` (presets, `#accent-custom` colour input, `#accent-reset` back to
jungle green `#2e7d4f`); persisted as `jv-accent`. All storage access is try/catch (file:// may throw).

Accent derivation (set together with `--accent`): `--accent-fg` is black or white, whichever has the higher WCAG contrast
with the accent. `--accent-text` equals the accent when its contrast against `--bg` is at least 3:1, otherwise the accent
is mixed toward white (dark theme) or black (light theme) until it reaches 3:1. Use `--accent-text` for accent-coloured
text/borders/lines and `--accent` + `--accent-fg` for filled accent surfaces. Helpers: `window.__jv.theme = {get, set,
accent: {get, set, reset}, contrast(a, b)}`.

## Battle card and drawers

- `.fighter` is one combatant (emoji, `.fname` name, `.bar` HP bar, `.fhp` HP text); `.mon` mirrors it to the right side.
- `.center` is the middle column of the battle card (zone and last action).
- `.drawer-body` opens in the layout (not as an overlay) with a bounded height and scrolls inside itself; `.short` sizes
  it to its content (used by Hero details).
- Number inputs are right-aligned.

## Chart axes

Axes use round ticks: the max is rounded up to 1/2/5 x 10^n and 3 to 5 evenly spaced ticks are drawn (`niceTicks`). Log
scale uses 0, 1, 10, 100, ... Histogram x labels are every Nth bin, anchored so the last bin is always labelled
(e.g. `39-40`) and each label is centred under its bar.

## Dashboard and replay additions

- `.caption` is a short muted explanation line. `.dash-side` stacks the right-hand dashboard cards.
- Outcomes bar: `.obar` with `.owin` / `.olose` segments, `.legend` with `.dot` (`.dot-win`, `.dot-lose`) markers.
- `.swatch.on` marks the selected accent. Scrollbars and range sliders are styled with tokens (thin, `--border`).
- Tooltips show on hover or keyboard focus (`:focus-visible`) only, and info buttons blur after a click.
- Drawers (`.drawer`) sit side by side; the open one grows and its body shows in the layout. The timeline card keeps a
  minimum plot height (`#timeline-wrap`).

## Event log modal, hero chips, speed

- `#log-modal` is a `<dialog>` opened with `showModal()` from `#log-open`. `.modal-in` is its padded flex column,
  `.modal-head` its title row, `.mtools` the search / group / count row. `#log` is the only scroller and is windowed
  (fixed `--s5` rows; `.lrow` event rows, `.lhead` collapsible group headers with `aria-expanded`, `.lspace` spacers).
- `#hero-details` holds `.chip` stat chips (`.chip-k` label, `.chip-v` value) inside the battle card.
- Speed is a logarithmic range `#speed` (0.25 to 4096) plus a numeric `#speed-num`; base rate is 10 events per second.
- `#seed` is a wide text input (digits only, 24ch).
- Canvases are sized to round(CSS size x devicePixelRatio) with a matching transform (`fitCanvas`).

## Post-game

- `#stage` is a `.badge` in the monster header showing "Stage N" (hidden at stage 0). `.post-stats` is the dashboard row of
  post-game averages (`.pstat` items). `.lev` marks evolve events in the log. Runs longer than the loaded window are
  fetched in windows (`from`, `count`); the seed is kept as a string so 20-digit seeds are never rounded.

## Map (replay)

- `#map-card` (a `.card`) sits above the battle card in the Replay view. `.map-head` is its title row with the `#mapzoom` toggle,
  `#mapwrap` is the sized box, `#map` the canvas (fills the box by percentage, no fixed px, no width/height attributes),
  `.map-tip` the hover tooltip (`#maptip`, `.hidden` when off) and `.map-empty` the empty state (`#mapempty`).
- The map is fetched once per seed and size from `GET /api/map?seed=&map_size=` (an optional `data/map.js` may set `window.MAP` for file://).
  Without a map or without hero x,y in the run the empty state shows and nothing throws.
- Canvas is DPR aware: backing size = round(css size x dpr), drawn in device pixels, `imageSmoothingEnabled = false`, integer tile size.
  Bird's-eye fits the whole map. Zoom (`#mapzoom` or double-click, stored as `localStorage jv-mapzoom` "1"/"0") shows a 16 x 16 tile
  window with the hero at the canvas centre and emoji on NPCs.
- Markers: hero `--accent`, elite NPC `--warn`, boss `--bad`, the fought monster `--fg`; the last walk path is a `--accent-text` trail.
  Tiles use `--tile-*` plus a translucent `--biome-N` overlay. Hover shows tile x,y, zone, kind and any NPC.
- Replay: every cursor change redraws from the current event (only events in the loaded window; outside it the last frame is held).
  A walk event animates the hero along `path`. All attacks (attack, skill, potion from the hero, hit_taken from the NPC) use the single
  `attackAnim` function: the attacker lunges 40% toward the target and back (250 ms at 1x) with a flash ring on the target.
  Animations are skipped when the scaled duration is under 40 ms (high speed). `requestAnimationFrame` runs only while one is active.
  State for tests: `window.__mapAnim = {active, kind, from, to, t, duration, pos, mpos}`.
- Log rows for walk, encounter, elite and boss events end with "(x,y)".
