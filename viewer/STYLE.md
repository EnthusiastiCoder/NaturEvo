# Viewer style system

One file, `viewer/index.html`, no build, no CDN. Look: shadcn/ui-style analytics dashboard (zinc neutrals,
1px borders, muted foreground, 8px radius, 32px controls). Everything below lives in the single `<style>` block.

## Tokens

All values come from `:root` custom properties. Outside the token blocks use `var(--x)` only: no hex/rgb
literals, no `px` font sizes, no inline `style=` in markup (JS may set dynamic widths/heights/positions only).
