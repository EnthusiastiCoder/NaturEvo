export type Route = 'landing' | 'run' | 'fight';

/** Bare URL = landing. ?fight= wins over ?run=. Record mode never shows the landing. */
export function route(search: string): Route {
  const q = new URLSearchParams(search);
  if (q.has('fight')) return 'fight';
  if (q.has('run') || q.has('record')) return 'run';
  return 'landing';
}

/** A requested run or fight that failed to load must show an error, never silently the landing. */
export function shouldShowRunError(r: Route, loadedOk: boolean): boolean {
  return r !== 'landing' && !loadedOk;
}
