import { Application, Container, Graphics } from 'pixi.js';
import type { Fight, FighterFrame, Frame, Seg } from './data';
import { cameraFor, lerpCamera, type Camera } from './camera';
import { debrisPose, drawRobot, PART_Z } from './robot-draw';
import { drawGround } from './ground';
import { hitstopTicks, shakeFor, sparkSpec, spawnParticles, type Particle } from './fx';
import { rendererOptions } from './renderer';

const hexNum = (css: string, fb: number) => { const m = /^#?([0-9a-f]{6})$/i.exec(css.trim()); return m ? parseInt(m[1], 16) : fb; };
const lerp = (a: number, b: number, k: number) => a + (b - a) * k;
const lerpSeg = (a: Seg, b: Seg, k: number): Seg => [lerp(a[0], b[0], k), lerp(a[1], b[1], k), lerp(a[2], b[2], k), lerp(a[3], b[3], k)];
