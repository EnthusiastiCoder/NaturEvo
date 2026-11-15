// v3 viewer entry. Routing: #/fight/<run>/<n> opens a duel; landing is wired by WS6.
import './style.css';
import { applyTheme } from './theme';
import { mountFightView } from './fight-view';
import type { FightScene } from './scene';
import { loadIndex } from './runs-index';
import { mountLanding } from './landing';
import { fightRoute } from './fight-route';
import { applyRecordMode, isRecordMode } from './record';

applyTheme(document.documentElement, 'dark');
applyRecordMode(document.documentElement, isRecordMode(location.search));

let scene: FightScene | null = null;
async function show(): Promise<void> {
  const m = fightRoute(location.hash);
  const landing = document.getElementById('landing');
  const view = document.getElementById('view');
  if (!view) return;
  scene?.destroy();
  scene = null;
  view.innerHTML = '';
  document.documentElement.dataset.view = m ? 'fight' : 'landing';
  if (landing) landing.hidden = !!m;
  if (!m && landing) { await mountLanding(landing, await loadIndex()); return; }
  if (m) scene = await mountFightView(view, m.url, m.title);
}
window.addEventListener('hashchange', () => void show());
void show();
