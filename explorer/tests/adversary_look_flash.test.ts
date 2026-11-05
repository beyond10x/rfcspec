/// <reference types="node" />
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const html = readFileSync(resolve(__dirname, '../index.html'), 'utf8');
const css = readFileSync(resolve(__dirname, '../src/style.css'), 'utf8');

/** Inline scripts (no `src`) in the document head, which run before the first paint. */
function headInlineScripts(page: string): string[] {
  const head = /<head>([\s\S]*?)<\/head>/.exec(page)?.[1] ?? '';
  return [...head.matchAll(/<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/g)].map((m) => m[1]!);
}

describe('the explorer paints the documentation site colour mode before its app runs', () => {
  it('sets data-theme in the head, as the documentation site does, or has a dark fallback without it', () => {
    const headSetsTheme = headInlineScripts(html).some((s) => s.includes('theme-cdd') && s.includes('data-theme'));
    const cssFallsBackToSystemDark = /prefers-color-scheme:\s*dark/.test(css);
    expect(
      headSetsTheme || cssFallsBackToSystemDark,
      'data-theme is only set when App.vue runs from the deferred module script; until then :root paints the light ' +
        'canvas, so a visitor whose stored or system mode is dark sees a light page first',
    ).toBe(true);
  });
});
