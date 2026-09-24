// Assumed module: src/route.ts  route(search: string): 'landing' | 'run' | 'fight'
import { describe, it, expect } from 'vitest';
import { route, shouldShowRunError } from '../src/route';

describe('route', () => {
  it('empty and bare query -> landing', () => {
    expect(route('')).toBe('landing');
    expect(route('?')).toBe('landing');
    expect(route('?foo=1')).toBe('landing');
  });
  it('?run=x -> run', () => expect(route('?run=x')).toBe('run'));
  it('?fight=y -> fight', () => expect(route('?fight=y')).toBe('fight'));
  it('fight wins when both present', () => expect(route('?run=x&fight=y')).toBe('fight'));
  it('?record&early=.. keeps existing non-landing behaviour', () => {
    expect(route('?record=1&early=a&late=b')).not.toBe('landing');
    expect(route('?record=1&run=x')).toBe('run');
  });
  it('?run= with empty value is still run, not landing', () => expect(route('?run=')).toBe('run'));
});
