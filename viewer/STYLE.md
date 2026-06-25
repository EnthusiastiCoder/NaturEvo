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
