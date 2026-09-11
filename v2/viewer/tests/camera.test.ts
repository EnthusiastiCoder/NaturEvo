import { describe, it, expect } from 'vitest';
import { fightCamera, mapCamera, separate, stateLabel } from '../src/layout';

const ARENA = { w: 1536, h: 819 };
describe('fight camera', () => {
