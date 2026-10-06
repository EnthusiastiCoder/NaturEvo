// Vector-only robot drawing: every part is a Graphics capsule. Endpoints come straight from the log.
import { Graphics } from 'pixi.js';
import type { FighterFrame, Seg } from './data';
import { partDamageStyle } from './damage-style';

// order: Head Torso Pelvis UArmL UArmR FArmL FArmR ThighL ThighR ShinL ShinR FootL FootR
const RADIUS_CM = [14, 30, 26, 11, 11, 10, 10, 15, 15, 12, 12, 9, 9];
const BACK = new Set([3, 5, 7, 9, 11]); // left limbs drawn darker (far side)
export const PART_Z = [8, 5, 4, 2, 6, 3, 7, 1, 2, 1, 2, 0, 1];
