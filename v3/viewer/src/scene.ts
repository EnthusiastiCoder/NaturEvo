import { Application, Container, Graphics } from 'pixi.js';
import type { Fight, FighterFrame, Frame, Seg } from './data';
import { cameraFor, lerpCamera, type Camera } from './camera';
import { debrisPose, drawRobot, PART_Z } from './robot-draw';
import { drawGround } from './ground';
import { hitstopTicks, shakeFor, sparkSpec, spawnParticles, type Particle } from './fx';
import { rendererOptions } from './renderer';
