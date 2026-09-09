import { Application, Container, Graphics, Sprite, Texture } from 'pixi.js';
import { type Fight, type Run } from './data';
import { fitStage } from './layout';
import { hash2, paintGround, paintRock, paintTree } from './ground';
import { sampleFight, type FighterFrame } from './interp';
import { ANIM, Atlas, HERO_MODEL, MONSTER_MODELS, directionFromAngle, loadAtlas } from './sprites';

export const STAGE_W = 1536, STAGE_H = 1024;
const PITCH = 0.8;
const ATTACK_TICKS = { Windup: 0, Active: 6, Recovery: 10 } as Record<string, number>;
const ATTACK_TOTAL = 16;
