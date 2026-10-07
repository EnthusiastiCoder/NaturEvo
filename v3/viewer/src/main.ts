// v3 viewer entry. Routing: #/fight/<run>/<n> opens a duel; landing is wired by WS6.
import './style.css';
import { applyTheme } from './theme';
import { mountFightView } from './fight-view';
import type { FightScene } from './scene';

applyTheme(document.documentElement, 'dark');

let scene: FightScene | null = null;
async function show(): Promise<void> {
  const m = /^#\/fight\/([^/]+)\/(\d+)$/.exec(location.hash);
  const landing = document.getElementById('landing');
  const view = document.getElementById('view');
  if (!view) return;
  scene?.destroy();
  scene = null;
  view.innerHTML = '';
  if (landing) landing.hidden = !!m;
  if (m) scene = await mountFightView(view, `/runs/${m[1]}/fight_gen_${m[2]}.json`, `${decodeURIComponent(m[1])} gen ${m[2]}`);
}
window.addEventListener('hashchange', () => void show());
void show();
