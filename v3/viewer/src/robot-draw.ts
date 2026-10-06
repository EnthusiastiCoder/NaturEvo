// Vector-only robot drawing: every part is a Graphics capsule. Endpoints come straight from the log.
import { Graphics } from 'pixi.js';
import type { FighterFrame, Seg } from './data';
import { partDamageStyle } from './damage-style';
