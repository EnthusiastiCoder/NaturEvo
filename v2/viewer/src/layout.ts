/** Largest stage with the given aspect that fits the window. */
export function fitStage(winW: number, winH: number, stageW: number, stageH: number) {
  const scale = Math.max(0.01, Math.min(winW / stageW, winH / stageH));
  return { w: Math.floor(stageW * scale), h: Math.floor(stageH * scale), scale };
}
