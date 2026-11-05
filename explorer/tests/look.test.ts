/// <reference types="node" />
import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import App from '../src/App.vue';

/** The documentation site's configuration, the one source of its navbar and footer. */
const config = readFileSync(resolve(__dirname, '../../website/docusaurus.config.ts'), 'utf8');

const SITE = 'https://beyond10x.github.io';
const EXPLORER = `${SITE}/rfcspec/explorer/?rfc=6455`;

type Link = { label: string; href: string };

/** Where Docusaurus sends a configured link, with `baseUrl: '/rfcspec/'` and `trailingSlash: false`. */
function built(target: string): string {
  if (target.startsWith('pathname://')) {
    return `${SITE}${target.slice('pathname://'.length)}`;
  }
  if (target.startsWith('/')) {
    const path = `/rfcspec${target}`.replace(/(.)\/$/, '$1');
    return `${SITE}${path}`;
  }
  return target;
}

/** The `{to|href: …, label: …}` and `{label: …, to|href: …}` links of one block of the configuration. */
function links(block: string): Link[] {
  const found: Link[] = [];
  for (const item of block.matchAll(/\{([^{}]*)\}/g)) {
    const body = item[1]!;
    const label = /label: '([^']*)'/.exec(body)?.[1];
    const target = /(?:to|href): '([^']*)'/.exec(body)?.[1];
    if (label !== undefined && target !== undefined) {
      found.push({ label, href: built(target) });
    }
  }
  return found;
}

function between(text: string, start: string, end: string): string {
  const from = text.indexOf(start);
  const to = text.indexOf(end, from + start.length);
  expect(from, start).toBeGreaterThanOrEqual(0);
  expect(to, end).toBeGreaterThan(from);
  return text.slice(from, to);
}

const navbar = between(config, 'navbar: {', 'footer: {');
const footer = between(config, 'footer: {', 'copyright:');
const title = /navbar: \{\s*title: '([^']*)'/.exec(config)?.[1];
const copyright = /copyright: '([^']*)'/.exec(config)?.[1];
const columns = [...footer.matchAll(/title: '([^']*)',\s*items: \[([^\]]*)\]/g)].map((m) => ({
  title: m[1]!,
  items: links(m[2]!),
}));

function shown(anchors: { text(): string; attributes(name: string): string | undefined }[]): Link[] {
  return anchors.map((a) => ({ label: a.text().trim(), href: new URL(a.attributes('href') ?? '', EXPLORER).href }));
}

async function explorer() {
  window.history.replaceState(null, '', '/rfcspec/explorer/?rfc=6455');
  vi.stubGlobal('fetch', async () => new Response('', { status: 404 }));
  const wrapper = mount(App);
  await flushPromises();
  return wrapper;
}

afterEach(() => {
  vi.unstubAllGlobals();
  window.history.replaceState(null, '', '/');
});

describe('the explorer takes the documentation site look', () => {
  it('reads the navbar and footer of the configuration', () => {
    expect(title).toBe('RFCspec');
    expect(links(navbar).length).toBeGreaterThanOrEqual(7);
    expect(columns.map((c) => c.title)).toEqual(['Documentation', 'Project']);
    expect(copyright).toBeDefined();
  });

  it('has the navbar of the documentation site: its title linked to the site root, then its links in order', async () => {
    const wrapper = await explorer();
    const bar = wrapper.find('nav.navbar');
    expect(bar.exists(), wrapper.html()).toBe(true);
    const brand = bar.find('a.navbar__brand');
    expect(brand.find('.navbar__title').text()).toBe(title);
    expect(new URL(brand.attributes('href')!, EXPLORER).href).toBe(`${SITE}/rfcspec/`);
    expect(shown(bar.findAll('a.navbar__link'))).toEqual(links(navbar));
    const active = bar.findAll('a.navbar__link--active');
    expect(active.map((a) => a.text())).toEqual(['RFC explorer']);
    expect(active[0]!.attributes('aria-current')).toBe('page');
  });

  it('has the footer of the documentation site: its columns, their links in order and its copyright', async () => {
    const wrapper = await explorer();
    const foot = wrapper.find('footer.footer');
    expect(foot.exists(), wrapper.html()).toBe(true);
    const shownColumns = foot.findAll('.footer__col').map((col) => ({
      title: col.find('.footer__title').text(),
      items: shown(col.findAll('a.footer__link-item')),
    }));
    expect(shownColumns).toEqual(columns);
    expect(foot.find('.footer__copyright').text()).toBe(copyright);
  });

  it('follows the colour mode the documentation site stored, else the system preference', async () => {
    window.localStorage.setItem('theme-cdd', 'light');
    await explorer();
    expect(document.documentElement.getAttribute('data-theme')).toBe('light');
    window.localStorage.setItem('theme-cdd', 'dark');
    await explorer();
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
    window.localStorage.removeItem('theme-cdd');
    vi.stubGlobal('matchMedia', (query: string) => ({ matches: query === '(prefers-color-scheme: dark)' }));
    await explorer();
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
    vi.stubGlobal('matchMedia', () => ({ matches: false }));
    await explorer();
    expect(document.documentElement.getAttribute('data-theme')).toBe('light');
  });
});
