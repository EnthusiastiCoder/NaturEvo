// Assumed module: src/route.ts  route(search: string): 'landing' | 'run' | 'fight'
import { describe, it, expect } from 'vitest';
import { route, shouldShowRunError } from '../src/route';
