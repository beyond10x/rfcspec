/// <reference types="node" />
import { describe, expect, it } from 'vitest';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const config = readFileSync(resolve(__dirname, '../../website/docusaurus.config.ts'), 'utf8');
const app = readFileSync(resolve(__dirname, '../src/App.vue'), 'utf8');

/**
 * Docusaurus names its colour-mode key `theme-<md5(url + baseUrl + '/')[0..3]>` when site storage is
 * namespaced (`@docusaurus/core/lib/server/storage.js`, `automaticNamespace`); `future.v4` turns that on.
 */
describe('the stored colour-mode key follows the documentation site configuration', () => {
  it('is the key Docusaurus derives from url and baseUrl', () => {
    const url = /\burl: '([^']*)'/.exec(config)?.[1];
    const baseUrl = /\bbaseUrl: '([^']*)'/.exec(config)?.[1];
    expect(url).toBeDefined();
    expect(baseUrl).toBeDefined();
    const normalized = `${url!.replace(/\/$/, '')}/${baseUrl!.replace(/^\/|\/$/g, '')}/`;
    const key = `theme-${createHash('md5').update(normalized).digest('hex').slice(0, 3)}`;
    expect(app).toContain(`'${key}'`);
  });
});
