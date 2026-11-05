/// <reference types="node" />
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const css = readFileSync(resolve(__dirname, '../src/style.css'), 'utf8');

function tokens(selector: RegExp): Record<string, string> {
  const block = selector.exec(css)?.[1];
  expect(block, String(selector)).toBeDefined();
  return Object.fromEntries([...block!.matchAll(/--b10x-([a-z-]+):\s*(#[0-9a-f]{3,6})/gi)].map((m) => [m[1]!, m[2]!]));
}

function luminance(hex: string): number {
  let h = hex.slice(1);
  if (h.length === 3) h = [...h].map((c) => c + c).join('');
  const [r, g, b] = [0, 2, 4].map((i) => {
    const c = parseInt(h.slice(i, i + 2), 16) / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r! + 0.7152 * g! + 0.0722 * b!;
}

function contrast(a: string, b: string): number {
  const [x, y] = [luminance(a), luminance(b)].sort((p, q) => q - p);
  return (x! + 0.05) / (y! + 0.05);
}

const light = tokens(/html\[data-theme='light'\],\s*:root\s*\{([^}]*)\}/);
const dark = { ...light, ...tokens(/html\[data-theme='dark'\]\s*\{([^}]*)\}/) };

/** Foreground and background pairs the stylesheet actually paints. */
const PAIRS: [string, string][] = [
  ['color-text', 'color-canvas'],
  ['color-heading', 'color-canvas'],
  ['color-muted', 'color-canvas'],
  ['color-link', 'color-canvas'],
  ['color-danger', 'color-canvas'],
  ['color-text', 'color-surface'],
  ['color-on-accent', 'color-accent'],
  ['color-on-accent', 'color-accent-strong'],
  ['color-heading', 'inline-code'],
];

describe('text on the new colours meets WCAG AA (4.5:1)', () => {
  for (const [name, theme] of [
    ['light', light],
    ['dark', dark],
  ] as const) {
    it(`${name} theme`, () => {
      const low = PAIRS.map(([fg, bg]) => ({ fg, bg, ratio: contrast(theme[fg]!, theme[bg]!) })).filter(
        (p) => p.ratio < 4.5,
      );
      expect(low).toEqual([]);
    });
  }
});
