// CHANGED DELIBERATELY (plans/2026-10-20_viewer-redesign-v2.md, Addendum 3): the event log is now a modal
// (#log-open -> #log-modal) that lists ALL events (search/grouping need the full set), windowed to <=300
// rows, with the current row highlighted (.cur) and scrolled into view. The old behaviour "log lists only
// 0..cur" (and its "append on step" window checks) is retired. Kept: #log is position:relative and the
// scroller, __jv.logIndices() returns numbers (the rendered indices), the .cur highlight + scroll-into-view,
// stepping forward does not rebuild the list. Grouping is set to "none" so rows are in event order.
import { html, load, makeRun, runner } from './_viewer_harness.mjs';

const { check, done } = runner('viewer_log_check');

check(/#log\s*\{[^}]*position\s*:\s*relative/.test(html), 'CSS #log{...position:relative} missing');

let v;
try { v = load({ protocol: 'file:', run: makeRun(300) }); } catch (e) { check(false, 'script load threw: ' + e.message); done(); }
const jv = v.win.__jv;
check(jv && typeof jv.setRun === 'function' && typeof jv.goto === 'function' && typeof jv.logIndices === 'function',
  'window.__jv hook (setRun/goto/logIndices) missing');
if (!jv || typeof jv.logIndices !== 'function') done();

const eq = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const idx = () => jv.logIndices();
const L = v.$('log');
check(!!v.$('log-open') && !!v.$('log-group'), '#log-open / #log-group missing (log is a modal now)');
if (!v.$('log-open') || !v.$('log-group')) done();
v.$('log-open').click();
v.$('log-group').value = 'none'; v.$('log-group').fire('change'); v.$('log-group').fire('input');

check(idx().every(n => typeof n === 'number'), 'logIndices must return numbers');
check(eq(idx(), Array.from({ length: 300 }, (_, i) => i)), 'modal lists ALL events 0..299 (300 fit in the window), got ' + idx().length);
const curOf = () => L._all().find(c => c.classList.contains('cur'));

jv.goto(50);
check(curOf() && curOf().dataset.i === '50', 'cur highlight on event 50');
check(idx().includes(51) && idx().includes(299), 'events after the current one are listed too');
check(L.scrollTop > 0, 'cur row scrolled into view: scrollTop must move for event 50, got ' + L.scrollTop);

jv.goto(250);
check(curOf() && curOf().dataset.i === '250' && L._all().filter(c => c.classList.contains('cur')).length === 1, 'cur highlight moves to 250 (single .cur)');
const s250 = L.scrollTop;
jv.goto(0);
check(curOf() && curOf().dataset.i === '0', 'cur highlight back at 0');
check(L.scrollTop === 0 && s250 > 0, 'scrollTop returns to 0 for event 0 (and was >0 at 250), got ' + L.scrollTop + ' / ' + s250);
check(idx().length === 300, 'goto(0) still lists all events, got ' + idx().length);

// stepping forward moves the highlight without rebuilding the list
v.$('play').click();
check(v.intervals.length > 0, 'play must start an interval');
const first = L.children[0], before = idx().length;
v.intervals[v.intervals.length - 1]();
check(curOf() && curOf().dataset.i === '1', 'step forward moves .cur to 1');
check(L.children[0] === first && idx().length === before, 'stepping forward must not rebuild the list');
